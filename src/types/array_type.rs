use llvm_sys::core::{LLVMConstArray, LLVMGetArrayLength};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use AddressSpace;
use context::ContextRef;
use support::LLVMString;
use types::traits::AsTypeRef;
use types::{Type, BasicTypeEnum, PointerType, FunctionType};
use values::{AsValueRef, ArrayValue, IntValue};

/// An `ArrayType` is the type of contiguous constants or variables.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ArrayType {
    array_type: Type,
}

impl ArrayType {
    pub(crate) fn new(array_type: LLVMTypeRef) -> Self {
        assert!(!array_type.is_null());

        ArrayType {
            array_type: Type::new(array_type),
        }
    }

    // REVIEW: Can be unsized if inner type is opaque struct
    /// Gets whether or not this `ArrayType` is sized or not.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    ///
    /// assert!(i8_array_type.is_sized());
    /// ```
    pub fn is_sized(&self) -> bool {
        self.array_type.is_sized()
    }

    // TODO: impl only for ArrayType<!StructType<Opaque>>
    /// Gets the size of this `ArrayType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_type_size = i8_array_type.size_of();
    /// ```
    pub fn size_of(&self) -> Option<IntValue> {
        if self.is_sized() {
            return Some(self.array_type.size_of())
        }

        None
    }

    /// Gets the alignment of this `ArrayType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_type_alignment = i8_array_type.get_alignment();
    /// ```
    pub fn get_alignment(&self) -> IntValue {
        self.array_type.get_alignment()
    }

    /// Creates a `PointerType` with this `ArrayType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_ptr_type = i8_array_type.ptr_type(AddressSpace::Generic);
    ///
    /// assert_eq!(i8_array_ptr_type.get_element_type().into_array_type(), i8_array_type);
    /// ```
    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.array_type.ptr_type(address_space)
    }

    /// Gets a reference to the `Context` this `ArrayType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    ///
    /// assert_eq!(*i8_array_type.get_context(), context);
    /// ```
    pub fn get_context(&self) -> ContextRef {
        self.array_type.get_context()
    }

    /// Creates a `FunctionType` with this `ArrayType` for its return type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let fn_type = i8_array_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(&self, param_types: &[BasicTypeEnum], is_var_args: bool) -> FunctionType {
        self.array_type.fn_type(param_types, is_var_args)
    }

    /// Creates an `ArrayType` with this `ArrayType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_array_type = i8_array_type.array_type(3);
    ///
    /// assert_eq!(i8_array_array_type.len(), 3);
    /// assert_eq!(i8_array_array_type.get_element_type().into_array_type(), i8_array_type);
    /// ```
    pub fn array_type(&self, size: u32) -> ArrayType {
        self.array_type.array_type(size)
    }

    /// Creates a constant `ArrayValue`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_array_type = f32_type.array_type(3);
    /// let f32_array_val = f32_array_type.const_zero();
    /// let f32_array_array = f32_array_type.const_array(&[f32_array_val, f32_array_val]);
    ///
    /// assert!(f32_array_array.is_const());
    /// ```
    pub fn const_array(&self, values: &[ArrayValue]) -> ArrayValue {
        let mut values: Vec<LLVMValueRef> = values.iter()
                                                  .map(|val| val.as_value_ref())
                                                  .collect();
        let value = unsafe {
            LLVMConstArray(self.as_type_ref(), values.as_mut_ptr(), values.len() as u32)
        };

        ArrayValue::new(value)
    }

    /// Creates a null `ArrayValue` of this `ArrayType`.
    /// It will be automatically assigned this `ArrayType`'s `Context`.
    ///
    /// # Example
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::types::FloatType;
    /// use inkwell::AddressSpace;
    ///
    /// // Global Context
    /// let f32_type = FloatType::f32_type();
    /// let f32_array_type = f32_type.array_type(7);
    /// let f32_array_null = f32_array_type.const_null();
    ///
    /// assert!(f32_array_null.is_null());
    ///
    /// // Custom Context
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_array_type = f32_type.array_type(7);
    /// let f32_array_null = f32_array_type.const_null();
    ///
    /// assert!(f32_array_null.is_null());
    /// ```
    pub fn const_null(&self) -> ArrayValue {
        ArrayValue::new(self.array_type.const_null())
    }

    /// Creates a constant zero value of this `ArrayType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_zero = i8_array_type.const_zero();
    /// ```
    pub fn const_zero(&self) -> ArrayValue {
        ArrayValue::new(self.array_type.const_zero())
    }

    /// Gets the length of this `ArrayType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    ///
    /// assert_eq!(i8_array_type.len(), 3);
    /// ```
    pub fn len(&self) -> u32 {
        unsafe {
            LLVMGetArrayLength(self.as_type_ref())
        }
    }

    /// Prints the definition of a `ArrayType` to a `LLVMString`.
    pub fn print_to_string(&self) -> LLVMString {
        self.array_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of an `ArrayType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7 => 4.0)]
    pub fn print_to_stderr(&self) {
        self.array_type.print_to_stderr()
    }

    /// Creates an undefined instance of a `ArrayType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_undef = i8_array_type.get_undef();
    ///
    /// assert!(i8_array_undef.is_undef());
    /// ```
    pub fn get_undef(&self) -> ArrayValue {
        ArrayValue::new(self.array_type.get_undef())
    }

    // SubType: ArrayType<BT> -> BT?
    /// Gets the element type of this `ArrayType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    ///
    /// assert_eq!(i8_array_type.get_element_type().into_int_type(), i8_type);
    /// ```
    pub fn get_element_type(&self) -> BasicTypeEnum {
        self.array_type.get_element_type().to_basic_type_enum()
    }

}

impl AsTypeRef for ArrayType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.array_type.type_
    }
}
