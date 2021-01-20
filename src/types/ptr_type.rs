use llvm_sys::core::{LLVMGetPointerAddressSpace, LLVMConstArray};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use crate::AddressSpace;
use crate::context::ContextRef;
use crate::types::traits::AsTypeRef;
use crate::types::{AnyTypeEnum, BasicTypeEnum, ArrayType, FunctionType, Type, VectorType};
use crate::values::{AsValueRef, ArrayValue, PointerValue, IntValue};

use std::convert::TryFrom;

/// A `PointerType` is the type of a pointer constant or variable.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PointerType<'ctx> {
    ptr_type: Type<'ctx>,
}

impl<'ctx> PointerType<'ctx> {
    pub(crate) fn new(ptr_type: LLVMTypeRef) -> Self {
        assert!(!ptr_type.is_null());

        PointerType {
            ptr_type: Type::new(ptr_type),
        }
    }

    /// Gets the size of this `PointerType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let f32_ptr_type_size = f32_ptr_type.size_of();
    /// ```
    pub fn size_of(self) -> IntValue<'ctx> {
        self.ptr_type.size_of().unwrap()
    }

    /// Gets the alignment of this `PointerType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let f32_ptr_type_alignment = f32_ptr_type.get_alignment();
    /// ```
    pub fn get_alignment(self) -> IntValue<'ctx> {
        self.ptr_type.get_alignment()
    }

    /// Creates a `PointerType` with this `PointerType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let f32_ptr_ptr_type = f32_ptr_type.ptr_type(AddressSpace::Generic);
    ///
    /// assert_eq!(f32_ptr_ptr_type.get_element_type().into_pointer_type(), f32_ptr_type);
    /// ```
    pub fn ptr_type(self, address_space: AddressSpace) -> PointerType<'ctx> {
        self.ptr_type.ptr_type(address_space)
    }

    /// Gets a reference to the `Context` this `PointerType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    ///
    /// assert_eq!(*f32_ptr_type.get_context(), context);
    /// ```
    // TODO: Move to AnyType trait
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.ptr_type.get_context()
    }

    /// Creates a `FunctionType` with this `PointerType` for its return type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = f32_ptr_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(self, param_types: &[BasicTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.ptr_type.fn_type(param_types, is_var_args)
    }

    /// Creates an `ArrayType` with this `PointerType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let f32_ptr_array_type = f32_ptr_type.array_type(3);
    ///
    /// assert_eq!(f32_ptr_array_type.len(), 3);
    /// assert_eq!(f32_ptr_array_type.get_element_type().into_pointer_type(), f32_ptr_type);
    /// ```
    pub fn array_type(self, size: u32) -> ArrayType<'ctx> {
        self.ptr_type.array_type(size)
    }

    /// Gets the `AddressSpace` a `PointerType` was created with.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    ///
    /// assert_eq!(f32_ptr_type.get_address_space(), AddressSpace::Generic);
    /// ```
    pub fn get_address_space(self) -> AddressSpace {
        let addr_space = unsafe {
            LLVMGetPointerAddressSpace(self.as_type_ref())
        };

        AddressSpace::try_from(addr_space).expect("Unexpectedly found invalid AddressSpace value")
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of an `IntType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7..=4.0)]
    pub fn print_to_stderr(self) {
        self.ptr_type.print_to_stderr()
    }

    /// Creates a null `PointerValue` of this `PointerType`.
    /// It will be automatically assigned this `PointerType`'s `Context`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// // Local Context
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let f32_ptr_null = f32_ptr_type.const_null();
    ///
    /// assert!(f32_ptr_null.is_null());
    /// ```
    pub fn const_null(self) -> PointerValue<'ctx> {
        PointerValue::new(self.ptr_type.const_zero())
    }

    // REVIEW: Unlike the other const_zero functions, this one becomes null instead of a 0 value. Maybe remove?
    /// Creates a constant null value of this `PointerType`.
    /// This is practically the same as calling `const_null` for this particular type and
    /// so this function may be removed in the future.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let f32_ptr_zero = f32_ptr_type.const_zero();
    /// ```
    pub fn const_zero(self) -> PointerValue<'ctx> {
        PointerValue::new(self.ptr_type.const_zero())
    }

    /// Creates an undefined instance of a `PointerType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let f32_ptr_undef = f32_ptr_type.get_undef();
    ///
    /// assert!(f32_ptr_undef.is_undef());
    /// ```
    pub fn get_undef(self) -> PointerValue<'ctx> {
        PointerValue::new(self.ptr_type.get_undef())
    }

    /// Creates a `VectorType` with this `PointerType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let f32_ptr_vec_type = f32_ptr_type.vec_type(3);
    ///
    /// assert_eq!(f32_ptr_vec_type.get_size(), 3);
    /// assert_eq!(f32_ptr_vec_type.get_element_type().into_pointer_type(), f32_ptr_type);
    /// ```
    pub fn vec_type(self, size: u32) -> VectorType<'ctx> {
        self.ptr_type.vec_type(size)
    }

    // SubType: PointerrType<BT> -> BT?
    /// Gets the element type of this `PointerType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    ///
    /// assert_eq!(f32_ptr_type.get_element_type().into_float_type(), f32_type);
    /// ```
    pub fn get_element_type(self) -> AnyTypeEnum<'ctx> {
        self.ptr_type.get_element_type()
    }

    /// Creates a constant `ArrayValue`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let f32_ptr_val = f32_ptr_type.const_null();
    /// let f32_ptr_array = f32_ptr_type.const_array(&[f32_ptr_val, f32_ptr_val]);
    ///
    /// assert!(f32_ptr_array.is_const());
    /// ```
    pub fn const_array(self, values: &[PointerValue<'ctx>]) -> ArrayValue<'ctx> {
        let mut values: Vec<LLVMValueRef> = values.iter()
                                                  .map(|val| val.as_value_ref())
                                                  .collect();
        let value = unsafe {
            LLVMConstArray(self.as_type_ref(), values.as_mut_ptr(), values.len() as u32)
        };

        ArrayValue::new(value)
    }
}

impl AsTypeRef for PointerType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.ptr_type.ty
    }
}
