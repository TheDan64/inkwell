use llvm_sys::core::{LLVMConstNamedStruct, LLVMCountStructElementTypes, LLVMGetStructElementTypes, LLVMGetStructName, LLVMIsPackedStruct, LLVMIsOpaqueStruct, LLVMStructSetBody, LLVMConstArray};
#[llvm_versions(3.7..=latest)]
use llvm_sys::core::LLVMStructGetTypeAtIndex;
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use std::ffi::CStr;
use std::mem::forget;

use crate::AddressSpace;
use crate::context::ContextRef;
use crate::types::traits::AsTypeRef;
use crate::types::{ArrayType, BasicTypeEnum, PointerType, FunctionType, Type};
use crate::values::{ArrayValue, BasicValueEnum, StructValue, IntValue, AsValueRef};
use crate::types::enums::BasicMetadataTypeEnum;

/// A `StructType` is the type of a heterogeneous container of types.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct StructType<'ctx> {
    struct_type: Type<'ctx>,
}

impl<'ctx> StructType<'ctx> {
    pub(crate) unsafe fn new(struct_type: LLVMTypeRef) -> Self {
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
    #[llvm_versions(3.7..=latest)]
    pub fn get_field_type_at_index(self, index: u32) -> Option<BasicTypeEnum<'ctx>> {
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

        unsafe {
            Some(BasicTypeEnum::new(LLVMStructGetTypeAtIndex(self.as_type_ref(), index)))
        }
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
    pub fn const_named_struct(self, values: &[BasicValueEnum<'ctx>]) -> StructValue<'ctx> {
        let mut args: Vec<LLVMValueRef> = values.iter()
                                                .map(|val| val.as_value_ref())
                                                .collect();
        unsafe {
            StructValue::new(LLVMConstNamedStruct(self.as_type_ref(), args.as_mut_ptr(), args.len() as u32))
        }
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
    pub fn const_zero(self) -> StructValue<'ctx> {
        unsafe {
            StructValue::new(self.struct_type.const_zero())
        }
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
    pub fn size_of(self) -> Option<IntValue<'ctx>> {
        self.struct_type.size_of()
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
    pub fn get_alignment(self) -> IntValue<'ctx> {
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
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.struct_type.get_context()
    }

    /// Gets this `StructType`'s name.
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
    pub fn ptr_type(self, address_space: AddressSpace) -> PointerType<'ctx> {
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
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
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
    pub fn array_type(self, size: u32) -> ArrayType<'ctx> {
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
    pub fn is_packed(self) -> bool {
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
    pub fn is_opaque(self) -> bool {
        unsafe {
            LLVMIsOpaqueStruct(self.as_type_ref()) == 1
        }
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
    pub fn count_fields(self) -> u32 {
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
    pub fn get_field_types(self) -> Vec<BasicTypeEnum<'ctx>> {
        let count = self.count_fields();
        let mut raw_vec: Vec<LLVMTypeRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetStructElementTypes(self.as_type_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| unsafe { BasicTypeEnum::new(*val) }).collect()
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of an `StructType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7..=4.0)]
    pub fn print_to_stderr(self) {
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
    pub fn get_undef(self) -> StructValue<'ctx> {
        unsafe {
            StructValue::new(self.struct_type.get_undef())
        }
    }

    // REVIEW: SubTypes should allow this to only be implemented for StructType<Opaque> one day
    // but would have to return StructType<Tys>. Maybe this is valid for non opaques, though
    // it might just override types?
    // REVIEW: What happens if called with &[] on an opaque? Should that still be opaque? Does the call break?
    /// Defines the body of an opaue `StructType`.
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
    pub fn set_body(self, field_types: &[BasicTypeEnum<'ctx>], packed: bool) -> bool {
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
    pub fn const_array(self, values: &[StructValue<'ctx>]) -> ArrayValue<'ctx> {
        let mut values: Vec<LLVMValueRef> = values.iter()
                                                  .map(|val| val.as_value_ref())
                                                  .collect();
        unsafe {
            ArrayValue::new(LLVMConstArray(self.as_type_ref(), values.as_mut_ptr(), values.len() as u32))
        }
    }
}

impl AsTypeRef for StructType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.struct_type.ty
    }
}
