use llvm_sys::core::{LLVMConstNamedStruct, LLVMConstStruct, LLVMStructType, LLVMCountStructElementTypes, LLVMGetStructElementTypes, LLVMGetStructName, LLVMIsPackedStruct, LLVMIsOpaqueStruct, LLVMStructSetBody, LLVMConstArray};
#[llvm_versions(3.7 => latest)]
use llvm_sys::core::LLVMStructGetTypeAtIndex;
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use std::ffi::CStr;
use std::mem::forget;

use AddressSpace;
use context::ContextRef;
use support::LLVMString;
use types::traits::AsTypeRef;
use types::{Type, BasicTypeEnum, ArrayType, PointerType, FunctionType, VectorType};
use values::{ArrayValue, BasicValueEnum, StructValue, IntValue, AsValueRef};

/// A `StructType` is the type of a heterogeneous container of types.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct StructType {
    struct_type: Type,
}

impl StructType {
    pub(crate) fn new(struct_type: LLVMTypeRef) -> Self {
        assert!(!struct_type.is_null());

        StructType {
            struct_type: Type::new(struct_type),
        }
    }

    /// Gets the type of a field belonging to this `StructType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into()], false);
    ///
    /// assert_eq!(struct_type.get_field_type_at_index(0).unwrap().into_float_type(), f32_type);
    /// ```
    #[llvm_versions(3.7 => latest)]
    pub fn get_field_type_at_index(&self, index: u32) -> Option<BasicTypeEnum> {
        // LLVM doesn't seem to just return null if opaque.
        // TODO: One day, with SubTypes (& maybe specialization?) we could just
        // impl this method for non opaque structs only
        if self.is_opaque() {
            return None;
        }

        // OoB indexing seems to be unchecked and therefore is UB
        if index >= self.count_fields() {
            return None;
        }

        let type_ = unsafe {
            LLVMStructGetTypeAtIndex(self.as_type_ref(), index)
        };

        Some(BasicTypeEnum::new(type_))
    }


    /// Creates a `StructValue` based on this `StructType`'s definition.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_zero = f32_type.const_float(0.);
    /// let struct_type = context.struct_type(&[f32_type.into()], false);
    /// let struct_val = struct_type.const_named_struct(&[f32_zero.into()]);
    /// ```
    pub fn const_named_struct(&self, values: &[BasicValueEnum]) -> StructValue {
        let mut args: Vec<LLVMValueRef> = values.iter()
                                                .map(|val| val.as_value_ref())
                                                .collect();
        let value = unsafe {
            LLVMConstNamedStruct(self.as_type_ref(), args.as_mut_ptr(), args.len() as u32)
        };

        StructValue::new(value)
    }

    /// Creates a `StructValue` based on the input values rather than an existing `StructType`.
    /// It will be assigned the global `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::types::{FloatType, StructType};
    ///
    /// let f32_type = FloatType::f32_type();
    /// let f32_zero = f32_type.const_float(0.);
    /// let struct_val = StructType::const_struct(&[f32_zero.into()], false);
    /// ```
    pub fn const_struct(values: &[BasicValueEnum], packed: bool) -> StructValue {
        let mut args: Vec<LLVMValueRef> = values.iter()
                                                .map(|val| val.as_value_ref())
                                                .collect();
        let value = unsafe {
            LLVMConstStruct(args.as_mut_ptr(), args.len() as u32, packed as i32)
        };

        StructValue::new(value)
    }

    /// Creates a null `StructValue` of this `StructType`.
    /// It will be automatically assigned this `StructType`'s `Context`.
    ///
    /// # Example
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::types::{FloatType, StructType};
    ///
    /// // Global Context
    /// let f32_type = FloatType::f32_type();
    /// let struct_type = StructType::struct_type(&[f32_type.into(), f32_type.into()], false);
    /// let struct_null = struct_type.const_null();
    ///
    /// assert!(struct_null.is_null());
    ///
    /// // Custom Context
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), f32_type.into()], false);
    /// let struct_null = struct_type.const_null();
    ///
    /// assert!(struct_null.is_null());
    /// ```
    pub fn const_null(&self) -> StructValue {
        StructValue::new(self.struct_type.const_null())
    }

    /// Creates a constant zero value of this `StructType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), f32_type.into()], false);
    /// let struct_zero = struct_type.const_zero();
    /// ```
    pub fn const_zero(&self) -> StructValue {
        StructValue::new(self.struct_type.const_zero())
    }

    // REVIEW: Can be false if opaque. To make a const fn, we'd have to have
    // have separate impls for Struct<Opaque> and StructType<T*>
    /// Gets whether or not this `StructType` is sized or not.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_struct_type = context.struct_type(&[f32_type.into()], false);
    ///
    /// assert!(f32_struct_type.is_sized());
    /// ```
    pub fn is_sized(&self) -> bool {
        self.struct_type.is_sized()
    }

    // TODO: impl it only for StructType<T*>?
    /// Gets the size of this `StructType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_struct_type = context.struct_type(&[f32_type.into()], false);
    /// let f32_struct_type_size = f32_struct_type.size_of();
    /// ```
    pub fn size_of(&self) -> Option<IntValue> {
        if self.is_sized() {
            return Some(self.struct_type.size_of());
        }

        None
    }

    /// Gets the alignment of this `StructType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), f32_type.into()], false);
    /// let struct_type_alignment = struct_type.get_alignment();
    /// ```
    pub fn get_alignment(&self) -> IntValue {
        self.struct_type.get_alignment()
    }

    /// Gets a reference to the `Context` this `StructType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), f32_type.into()], false);
    ///
    /// assert_eq!(*struct_type.get_context(), context);
    /// ```
    pub fn get_context(&self) -> ContextRef {
        self.struct_type.get_context()
    }

    /// Determines whether or not a `StructType` is opaque.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.opaque_struct_type("opaque_struct");
    ///
    /// assert_eq!(struct_type.get_name().unwrap().to_str().unwrap(), "opaque_struct");
    /// ```
    pub fn get_name(&self) -> Option<&CStr> {
        let name = unsafe {
            LLVMGetStructName(self.as_type_ref())
        };

        if name.is_null() {
            return None;
        }

        let c_str = unsafe {
            CStr::from_ptr(name)
        };

        Some(c_str)
    }

    /// Creates a `PointerType` with this `StructType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), f32_type.into()], false);
    /// let struct_ptr_type = struct_type.ptr_type(AddressSpace::Generic);
    ///
    /// assert_eq!(struct_ptr_type.get_element_type().into_struct_type(), struct_type);
    /// ```
    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.struct_type.ptr_type(address_space)
    }

    /// Creates a `FunctionType` with this `StructType` for its return type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), f32_type.into()], false);
    /// let fn_type = struct_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(&self, param_types: &[BasicTypeEnum], is_var_args: bool) -> FunctionType {
        self.struct_type.fn_type(param_types, is_var_args)
    }

    /// Creates an `ArrayType` with this `StructType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), f32_type.into()], false);
    /// let struct_array_type = struct_type.array_type(3);
    ///
    /// assert_eq!(struct_array_type.len(), 3);
    /// assert_eq!(struct_array_type.get_element_type().into_struct_type(), struct_type);
    /// ```
    pub fn array_type(&self, size: u32) -> ArrayType {
        self.struct_type.array_type(size)
    }

    /// Determines whether or not a `StructType` is packed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), f32_type.into()], false);
    ///
    /// assert!(struct_type.is_packed());
    /// ```
    pub fn is_packed(&self) -> bool {
        unsafe {
            LLVMIsPackedStruct(self.as_type_ref()) == 1
        }
    }

    /// Determines whether or not a `StructType` is opaque.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.opaque_struct_type("opaque_struct");
    ///
    /// assert!(struct_type.is_opaque());
    /// ```
    pub fn is_opaque(&self) -> bool {
        unsafe {
            LLVMIsOpaqueStruct(self.as_type_ref()) == 1
        }
    }

    /// Creates a `StructType` definiton from heterogeneous types in the global `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::{FloatType, IntType, StructType};
    ///
    /// let f32_type = FloatType::f32_type();
    /// let i8_type = IntType::i8_type();
    /// let struct_type = StructType::struct_type(&[f32_type.into(), i8_type.into()], false);
    ///
    /// assert_eq!(struct_type.get_context(), Context::get_global());
    /// ```
    pub fn struct_type(field_types: &[BasicTypeEnum], packed: bool) -> Self {
        let mut field_types: Vec<LLVMTypeRef> = field_types.iter()
                                                           .map(|val| val.as_type_ref())
                                                           .collect();
        let struct_type = unsafe {
            LLVMStructType(field_types.as_mut_ptr(), field_types.len() as u32, packed as i32)
        };

        StructType::new(struct_type)
    }

    /// Counts the number of field types.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let i8_type = context.i8_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), i8_type.into()], false);
    ///
    /// assert_eq!(struct_type.count_fields(), 2);
    /// ```
    pub fn count_fields(&self) -> u32 {
        unsafe {
            LLVMCountStructElementTypes(self.as_type_ref())
        }
    }

    /// Gets this `StructType`'s field types.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let i8_type = context.i8_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), i8_type.into()], false);
    ///
    /// assert_eq!(struct_type.get_field_types(), &[f32_type.into(), i8_type.into()]);
    /// ```
    pub fn get_field_types(&self) -> Vec<BasicTypeEnum> {
        let count = self.count_fields();
        let mut raw_vec: Vec<LLVMTypeRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetStructElementTypes(self.as_type_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| BasicTypeEnum::new(*val)).collect()
    }

    /// Prints the definition of a `VectorType` to a `LLVMString`.
    pub fn print_to_string(&self) -> LLVMString {
        self.struct_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of an `StructType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7 => 4.0)]
    pub fn print_to_stderr(&self) {
        self.struct_type.print_to_stderr()
    }

    /// Creates an undefined instance of a `StructType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let i8_type = context.i8_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), i8_type.into()], false);
    /// let struct_type_undef = struct_type.get_undef();
    ///
    /// assert!(struct_type_undef.is_undef());
    /// ```
    pub fn get_undef(&self) -> StructValue {
        StructValue::new(self.struct_type.get_undef())
    }

    // REVIEW: SubTypes should allow this to only be implemented for StructType<Opaque> one day
    // but would have to return StructType<Tys>. Maybe this is valid for non opaques, though
    // it might just override types?
    // REVIEW: What happens if called with &[] on an opaque? Should that still be opaque? Does the call break?
    /// Determines whether or not a `StructType` is opaque.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let opaque_struct_type = context.opaque_struct_type("opaque_struct");
    ///
    /// opaque_struct_type.set_body(&[f32_type.into()], false);
    ///
    /// assert!(!opaque_struct_type.is_opaque());
    /// ```
    pub fn set_body(&self, field_types: &[BasicTypeEnum], packed: bool) -> bool {
        let is_opaque = self.is_opaque();
        let mut field_types: Vec<LLVMTypeRef> = field_types.iter()
                                                           .map(|val| val.as_type_ref())
                                                           .collect();
        if is_opaque {
            unsafe {
                LLVMStructSetBody(self.as_type_ref(), field_types.as_mut_ptr(), field_types.len() as u32, packed as i32);
            }
        }

        is_opaque
    }

    /// Creates a `VectorType` with this `StructType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), f32_type.into()], false);
    /// let struct_vec_type = struct_type.vec_type(3);
    ///
    /// assert_eq!(struct_vec_type.get_size(), 3);
    /// assert_eq!(struct_vec_type.get_element_type().into_struct_type(), struct_type);
    /// ```
    pub fn vec_type(&self, size: u32) -> VectorType {
        self.struct_type.vec_type(size)
    }

    /// Creates a constant `ArrayValue`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), f32_type.into()], false);
    /// let struct_val = struct_type.const_named_struct(&[]);
    /// let struct_array = struct_type.const_array(&[struct_val, struct_val]);
    ///
    /// assert!(struct_array.is_const());
    /// ```
    pub fn const_array(&self, values: &[StructValue]) -> ArrayValue {
        let mut values: Vec<LLVMValueRef> = values.iter()
                                                  .map(|val| val.as_value_ref())
                                                  .collect();
        let value = unsafe {
            LLVMConstArray(self.as_type_ref(), values.as_mut_ptr(), values.len() as u32)
        };

        ArrayValue::new(value)
    }
}

impl AsTypeRef for StructType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.struct_type.type_
    }
}
