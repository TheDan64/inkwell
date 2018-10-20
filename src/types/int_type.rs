use llvm_sys::core::{LLVMInt1Type, LLVMInt8Type, LLVMInt16Type, LLVMInt32Type, LLVMInt64Type, LLVMConstInt, LLVMConstAllOnes, LLVMIntType, LLVMGetIntTypeWidth, LLVMConstIntOfStringAndSize, LLVMConstIntOfArbitraryPrecision, LLVMConstArray};
use llvm_sys::execution_engine::LLVMCreateGenericValueOfInt;
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use AddressSpace;
use context::ContextRef;
use support::LLVMString;
use types::traits::AsTypeRef;
use types::{Type, ArrayType, BasicTypeEnum, VectorType, PointerType, FunctionType};
use values::{AsValueRef, ArrayValue, GenericValue, IntValue};

/// An `IntType` is the type of an integer constant or variable.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IntType {
    int_type: Type,
}

impl IntType {
    pub(crate) fn new(int_type: LLVMTypeRef) -> Self {
        assert!(!int_type.is_null());

        IntType {
            int_type: Type::new(int_type),
        }
    }

    /// Gets the `IntType` representing 1 bit width. It will be automatically assigned the global context.
    ///
    /// To use your own `Context`, see [inkwell::context::bool_type()](../context/struct.Context.html#method.bool_type)
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// let bool_type = IntType::bool_type();
    ///
    /// assert_eq!(bool_type.get_bit_width(), 1);
    /// assert_eq!(bool_type.get_context(), Context::get_global());
    /// ```
    pub fn bool_type() -> Self {
        let type_ = unsafe {
            LLVMInt1Type()
        };

        IntType::new(type_)
    }

    /// Gets the `IntType` representing 8 bit width. It will be automatically assigned the global context.
    ///
    /// To use your own `Context`, see [inkwell::context::i8_type()](../context/struct.Context.html#method.i8_type)
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// let i8_type = IntType::i8_type();
    ///
    /// assert_eq!(i8_type.get_bit_width(), 8);
    /// assert_eq!(i8_type.get_context(), Context::get_global());
    /// ```
    pub fn i8_type() -> Self {
        let type_ = unsafe {
            LLVMInt8Type()
        };

        IntType::new(type_)
    }

    /// Gets the `IntType` representing 16 bit width. It will be automatically assigned the global context.
    ///
    /// To use your own `Context`, see [inkwell::context::i16_type()](../context/struct.Context.html#method.i16_type)
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// let i16_type = IntType::i16_type();
    ///
    /// assert_eq!(i16_type.get_bit_width(), 16);
    /// assert_eq!(i16_type.get_context(), Context::get_global());
    /// ```
    pub fn i16_type() -> Self {
        let type_ = unsafe {
            LLVMInt16Type()
        };

        IntType::new(type_)
    }

    /// Gets the `IntType` representing 32 bit width. It will be automatically assigned the global context.
    ///
    /// To use your own `Context`, see [inkwell::context::i32_type()](../context/struct.Context.html#method.i32_type)
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// let i32_type = IntType::i32_type();
    ///
    /// assert_eq!(i32_type.get_bit_width(), 32);
    /// assert_eq!(i32_type.get_context(), Context::get_global());
    /// ```
    pub fn i32_type() -> Self {
        let type_ = unsafe {
            LLVMInt32Type()
        };

        IntType::new(type_)
    }

    /// Gets the `IntType` representing 64 bit width. It will be automatically assigned the global context.
    ///
    /// To use your own `Context`, see [inkwell::context::i64_type()](../context/struct.Context.html#method.i64_type)
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// let i64_type = IntType::i64_type();
    ///
    /// assert_eq!(i64_type.get_bit_width(), 64);
    /// assert_eq!(i64_type.get_context(), Context::get_global());
    /// ```
    pub fn i64_type() -> Self {
        let type_ = unsafe {
            LLVMInt64Type()
        };

        IntType::new(type_)
    }

    /// Gets the `IntType` representing 128 bit width. It will be automatically assigned the global context.
    ///
    /// To use your own `Context`, see [inkwell::context::i128_type()](../context/struct.Context.html#method.i128_type)
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// let i128_type = IntType::i128_type();
    ///
    /// assert_eq!(i128_type.get_bit_width(), 128);
    /// assert_eq!(i128_type.get_context(), Context::get_global());
    /// ```
    pub fn i128_type() -> Self {
        // REVIEW: The docs says there's a LLVMInt128Type, but
        // it might only be in a newer version

        Self::custom_width_int_type(128)
    }

    /// Gets the `IntType` representing a custom bit width. It will be automatically assigned the global context.
    ///
    /// To use your own `Context`, see [inkwell::context::custom_width_int_type()](../context/struct.Context.html#method.custom_width_int_type)
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// let i42_type = IntType::custom_width_int_type(42);
    ///
    /// assert_eq!(i42_type.get_bit_width(), 42);
    /// assert_eq!(i42_type.get_context(), Context::get_global());
    /// ```
    pub fn custom_width_int_type(bits: u32) -> Self {
        let type_ = unsafe {
            LLVMIntType(bits)
        };

        IntType::new(type_)
    }

    /// Creates an `IntValue` repesenting a constant value of this `IntType`. It will be automatically assigned this `IntType`'s `Context`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// // Global Context
    /// let i32_type = IntType::i32_type();
    /// let i32_value = i32_type.const_int(42, false);
    ///
    /// // Custom Context
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    /// let i32_value = i32_type.const_int(42, false);
    /// ```
    // TODOC: Maybe better explain sign extension
    pub fn const_int(&self, value: u64, sign_extend: bool) -> IntValue {
        let value = unsafe {
            LLVMConstInt(self.as_type_ref(), value, sign_extend as i32)
        };

        IntValue::new(value)
    }

    /// Create an `IntValue` from a string and radix. LLVM provides no error handling here,
    /// so this may produce unexpected results and should not be relied upon for validation.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_val = i8_type.const_int_from_string("0121", 10);
    ///
    /// assert_eq!(i8_val.print_to_string().to_string(), "i8 121");
    ///
    /// let i8_val = i8_type.const_int_from_string("0121", 3);
    ///
    /// assert_eq!(i8_val.print_to_string().to_string(), "i8 16");
    ///
    /// // Unexpected outputs:
    /// let i8_val = i8_type.const_int_from_string("0121", 2);
    ///
    /// assert_eq!(i8_val.print_to_string().to_string(), "i8 3");
    ///
    /// let i8_val = i8_type.const_int_from_string("ABCD", 2);
    ///
    /// assert_eq!(i8_val.print_to_string().to_string(), "i8 -15");
    /// ```
    pub fn const_int_from_string(&self, slice: &str, radix: u8) -> IntValue {
        let value = unsafe {
            LLVMConstIntOfStringAndSize(self.as_type_ref(), slice.as_ptr() as *const i8, slice.len() as u32, radix)
        };

        IntValue::new(value)
    }

    /// Create a constant `IntValue` of arbitrary precision.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i64_type = context.i64_type();
    /// let i64_val = i64_type.const_int_arbitrary_precision(&[1, 2]);
    /// ```
    pub fn const_int_arbitrary_precision(&self, words: &[u64]) -> IntValue {
        let value = unsafe {
            LLVMConstIntOfArbitraryPrecision(self.as_type_ref(), words.len() as u32, words.as_ptr())
        };

        IntValue::new(value)
    }

    /// Creates an `IntValue` representing a constant value of all one bits of this `IntType`. It will be automatically assigned this `IntType`'s `Context`.
    ///
    /// # Example
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// // Global Context
    /// let i32_type = IntType::i32_type();
    /// let i32_ptr_value = i32_type.const_all_ones();
    ///
    /// // Custom Context
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    /// let i32_ptr_value = i32_type.const_all_ones();
    /// ```
    pub fn const_all_ones(&self) -> IntValue {
        let value = unsafe {
            LLVMConstAllOnes(self.as_type_ref())
        };

        IntValue::new(value)
    }

    /// Creates a constant null value of this `IntType`.
    ///
    /// # Example
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// // Global Context
    /// let i32_type = IntType::i32_type();
    /// let i32_value = i32_type.const_null();
    ///
    /// // Custom Context
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    /// let i32_value = i32_type.const_null();
    /// ```
    pub fn const_null(&self) -> IntValue {
        IntValue::new(self.int_type.const_null())
    }

    /// Creates a constant zero value of this `IntType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_zero = i8_type.const_zero();
    ///
    /// assert_eq!(i8_zero.print_to_string().to_string(), "i8 0");
    /// ```
    pub fn const_zero(&self) -> IntValue {
        IntValue::new(self.int_type.const_zero())
    }

    /// Creates a `FunctionType` with this `IntType` for its return type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let fn_type = i8_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(&self, param_types: &[BasicTypeEnum], is_var_args: bool) -> FunctionType {
        self.int_type.fn_type(param_types, is_var_args)
    }

    /// Creates an `ArrayType` with this `IntType` for its element type.
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
    /// assert_eq!(i8_array_type.get_element_type().into_int_type(), i8_type);
    /// ```
    pub fn array_type(&self, size: u32) -> ArrayType {
        self.int_type.array_type(size)
    }

    /// Creates a `VectorType` with this `IntType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_vector_type = i8_type.vec_type(3);
    ///
    /// assert_eq!(i8_vector_type.get_size(), 3);
    /// assert_eq!(i8_vector_type.get_element_type().into_int_type(), i8_type);
    /// ```
    pub fn vec_type(&self, size: u32) -> VectorType {
        self.int_type.vec_type(size)
    }

    /// Gets a reference to the `Context` this `IntType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    ///
    /// assert_eq!(*i8_type.get_context(), context);
    /// ```
    pub fn get_context(&self) -> ContextRef {
        self.int_type.get_context()
    }

    // REVIEW: Always true -> const fn on trait?
    /// Gets whether or not this `IntType` is sized or not. This is likely
    /// always true and may be removed in the future.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    ///
    /// assert!(i8_type.is_sized());
    /// ```
    pub fn is_sized(&self) -> bool {
        self.int_type.is_sized()
    }

    /// Gets the size of this `IntType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_type_size = i8_type.size_of();
    /// ```
    pub fn size_of(&self) -> IntValue {
        self.int_type.size_of()
    }

    /// Gets the alignment of this `IntType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_type_alignment = i8_type.get_alignment();
    /// ```
    pub fn get_alignment(&self) -> IntValue {
        self.int_type.get_alignment()
    }

    /// Creates a `PointerType` with this `IntType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_ptr_type = i8_type.ptr_type(AddressSpace::Generic);
    ///
    /// assert_eq!(i8_ptr_type.get_element_type().into_int_type(), i8_type);
    /// ```
    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.int_type.ptr_type(address_space)
    }

    /// Gets the bit width of an `IntType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::types::IntType;
    ///
    /// let bool_type = IntType::bool_type();
    ///
    /// assert_eq!(bool_type.get_bit_width(), 1);
    /// ```
    pub fn get_bit_width(&self) -> u32 {
        unsafe {
            LLVMGetIntTypeWidth(self.as_type_ref())
        }
    }

    /// Prints the definition of an `IntType` to a `LLVMString`.
    pub fn print_to_string(&self) -> LLVMString {
        self.int_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of an `IntType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7 => 4.0)]
    pub fn print_to_stderr(&self) {
        self.int_type.print_to_stderr()
    }

    /// Creates an undefined instance of an `IntType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_undef = i8_type.get_undef();
    ///
    /// assert!(i8_undef.is_undef());
    /// ```
    pub fn get_undef(&self) -> IntValue {
        IntValue::new(self.int_type.get_undef())
    }

    /// Creates a `GenericValue` for use with `ExecutionEngine`s.
    pub fn create_generic_value(&self, value: u64, is_signed: bool) -> GenericValue {
        let value = unsafe {
            LLVMCreateGenericValueOfInt(self.as_type_ref(), value, is_signed as i32)
        };

        GenericValue::new(value)
    }

    /// Creates a constant `ArrayValue`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_val = i8_type.const_int(0, false);
    /// let i8_val2 = i8_type.const_int(2, false);
    /// let i8_array = i8_type.const_array(&[i8_val, i8_val2]);
    ///
    /// assert!(i8_array.is_const());
    /// ```
    pub fn const_array(&self, values: &[IntValue]) -> ArrayValue {
        let mut values: Vec<LLVMValueRef> = values.iter()
                                                  .map(|val| val.as_value_ref())
                                                  .collect();
        let value = unsafe {
            LLVMConstArray(self.as_type_ref(), values.as_mut_ptr(), values.len() as u32)
        };

        ArrayValue::new(value)
    }
}

impl AsTypeRef for IntType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.int_type.type_
    }
}
