use llvm_sys::core::{
    LLVMConstNamedStruct, LLVMCountStructElementTypes, LLVMGetStructElementTypes, LLVMGetStructName,
    LLVMIsOpaqueStruct, LLVMIsPackedStruct, LLVMStructGetTypeAtIndex, LLVMStructSetBody,
};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use std::ffi::CStr;
use std::fmt::{self, Display};
use std::mem::forget;

use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::types::enums::BasicMetadataTypeEnum;
use crate::types::traits::AsTypeRef;
use crate::types::{ArrayType, BasicTypeEnum, FunctionType, PointerType, Type};
use crate::values::{ArrayValue, AsValueRef, BasicValueEnum, IntValue, StructValue};
use crate::AddressSpace;

/// A `StructType` is the type of a heterogeneous container of types.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct StructType<'ctx> {
    struct_type: Type<'ctx>,
}

impl<'ctx> StructType<'ctx> {
    /// Create `StructType` from [`LLVMTypeRef`]
    ///
    /// # Safety
    /// Undefined behavior, if referenced type isn't struct type
    pub unsafe fn new(struct_type: LLVMTypeRef) -> Self {
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

        Some(unsafe { self.get_field_type_at_index_unchecked(index) })
    }

    /// Gets the type of a field belonging to this `StructType`.
    ///
    /// # Safety
    ///
    /// The index must be less than [StructType::count_fields] and the struct must not be opaque.
    pub unsafe fn get_field_type_at_index_unchecked(self, index: u32) -> BasicTypeEnum<'ctx> {
        unsafe { BasicTypeEnum::new(LLVMStructGetTypeAtIndex(self.as_type_ref(), index)) }
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
        let mut args: Vec<LLVMValueRef> = values.iter().map(|val| val.as_value_ref()).collect();
        unsafe {
            StructValue::new(LLVMConstNamedStruct(
                self.as_type_ref(),
                args.as_mut_ptr(),
                args.len() as u32,
            ))
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
        unsafe { StructValue::new(self.struct_type.const_zero()) }
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
    /// assert_eq!(struct_type.get_context(), context);
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
        let name = unsafe { LLVMGetStructName(self.as_type_ref()) };

        if name.is_null() {
            return None;
        }

        let c_str = unsafe { CStr::from_ptr(name) };

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
    /// let struct_ptr_type = struct_type.ptr_type(AddressSpace::default());
    ///
    /// #[cfg(feature = "typed-pointers")]
    /// assert_eq!(struct_ptr_type.get_element_type().into_struct_type(), struct_type);
    /// ```
    #[cfg_attr(
        any(
            all(feature = "llvm15-0", not(feature = "typed-pointers")),
            all(feature = "llvm16-0", not(feature = "typed-pointers")),
            feature = "llvm17-0",
            feature = "llvm18-1"
        ),
        deprecated(
            note = "Starting from version 15.0, LLVM doesn't differentiate between pointer types. Use Context::ptr_type instead."
        )
    )]
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
        unsafe { LLVMIsPackedStruct(self.as_type_ref()) == 1 }
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
        unsafe { LLVMIsOpaqueStruct(self.as_type_ref()) == 1 }
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
        unsafe { LLVMCountStructElementTypes(self.as_type_ref()) }
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

    /// Get a struct field iterator.
    pub fn get_field_types_iter(self) -> FieldTypesIter<'ctx> {
        FieldTypesIter {
            st: self,
            i: 0,
            count: if self.is_opaque() { 0 } else { self.count_fields() },
        }
    }

    /// Print the definition of a `StructType` to `LLVMString`.
    pub fn print_to_string(self) -> LLVMString {
        self.struct_type.print_to_string()
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
        unsafe { StructValue::new(self.struct_type.get_undef()) }
    }

    /// Creates a poison instance of a `StructType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::values::AnyValue;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let i8_type = context.i8_type();
    /// let struct_type = context.struct_type(&[f32_type.into(), i8_type.into()], false);
    /// let struct_type_poison = struct_type.get_poison();
    ///
    /// assert!(struct_type_poison.is_poison());
    /// ```
    #[llvm_versions(12..)]
    pub fn get_poison(self) -> StructValue<'ctx> {
        unsafe { StructValue::new(self.struct_type.get_poison()) }
    }

    /// Defines the body of a `StructType`.
    ///
    /// If the struct is an opaque type, it will no longer be after this call.
    ///
    /// Resetting the `packed` state of a non-opaque struct type may not work.
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
        let mut field_types: Vec<LLVMTypeRef> = field_types.iter().map(|val| val.as_type_ref()).collect();
        unsafe {
            LLVMStructSetBody(
                self.as_type_ref(),
                field_types.as_mut_ptr(),
                field_types.len() as u32,
                packed as i32,
            );
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
        unsafe { ArrayValue::new_const_array(&self, values) }
    }
}

unsafe impl AsTypeRef for StructType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.struct_type.ty
    }
}

impl Display for StructType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

/// Iterate over all `BasicTypeEnum`s in a struct.
#[derive(Debug)]
pub struct FieldTypesIter<'ctx> {
    st: StructType<'ctx>,
    i: u32,
    count: u32,
}

impl<'ctx> Iterator for FieldTypesIter<'ctx> {
    type Item = BasicTypeEnum<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.count {
            let result = unsafe { self.st.get_field_type_at_index_unchecked(self.i) };
            self.i += 1;
            Some(result)
        } else {
            None
        }
    }
}
