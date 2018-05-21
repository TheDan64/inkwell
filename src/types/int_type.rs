use llvm_sys::core::{LLVMInt1Type, LLVMInt8Type, LLVMInt16Type, LLVMInt32Type, LLVMInt64Type, LLVMConstInt, LLVMConstNull, LLVMConstAllOnes, LLVMIntType, LLVMGetIntTypeWidth, LLVMConstIntOfStringAndSize, LLVMConstIntOfArbitraryPrecision};
use llvm_sys::execution_engine::LLVMCreateGenericValueOfInt;
use llvm_sys::prelude::LLVMTypeRef;

use std::ffi::CStr;

use AddressSpace;
use context::ContextRef;
use types::traits::AsTypeRef;
use types::{Type, ArrayType, BasicType, VectorType, PointerType, FunctionType};
use values::{GenericValue, IntValue, PointerValue};

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
    /// ```
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
    /// ```
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
    /// ```
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
    /// ```
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
    /// ```
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
    /// ```
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
    /// ```
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
    /// ```
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

    // TODOC: LLVM will parse as best as it can, without any error for invalid input
    // ie ("012", 2) => int 1
    pub fn const_int_from_string(&self, slice: &str, radix: u8) -> IntValue {
        let value = unsafe {
            LLVMConstIntOfStringAndSize(self.as_type_ref(), slice.as_ptr() as *const i8, slice.len() as u32, radix)
        };

        IntValue::new(value)
    }

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

    /// Creates a `PointerValue` representing a constant value of zero (null pointer) pointing to this `IntType`. It will be automatically assigned this `IntType`'s `Context`.
    ///
    /// # Example
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::types::IntType;
    ///
    /// // Global Context
    /// let i32_type = IntType::i32_type();
    /// let i32_value = i32_type.const_null_ptr();
    ///
    /// // Custom Context
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    /// let i32_value = i32_type.const_null_ptr();
    /// ```
    pub fn const_null_ptr(&self) -> PointerValue {
        self.int_type.const_null_ptr()
    }

    pub fn const_null(&self) -> IntValue {
        let null = unsafe {
            LLVMConstNull(self.as_type_ref())
        };

        IntValue::new(null)
    }

    pub fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
        self.int_type.fn_type(param_types, is_var_args)
    }

    pub fn array_type(&self, size: u32) -> ArrayType {
        self.int_type.array_type(size)
    }

    pub fn vec_type(&self, size: u32) -> VectorType {
        self.int_type.vec_type(size)
    }

    pub fn get_context(&self) -> ContextRef {
        self.int_type.get_context()
    }

    // REVIEW: Always true -> const fn?
    pub fn is_sized(&self) -> bool {
        self.int_type.is_sized()
    }

    pub fn size_of(&self) -> IntValue {
        self.int_type.size_of()
    }

    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.int_type.ptr_type(address_space)
    }

    pub fn get_bit_width(&self) -> u32 {
        unsafe {
            LLVMGetIntTypeWidth(self.as_type_ref())
        }
    }

    pub fn print_to_string(&self) -> &CStr {
        self.int_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm5-0")))]
    pub fn print_to_stderr(&self) {
        self.int_type.print_to_stderr()
    }

    pub fn get_undef(&self) -> IntValue {
        IntValue::new(self.int_type.get_undef())
    }

    pub fn create_generic_value(&self, value: u64, is_signed: bool) -> GenericValue {
        let value = unsafe {
            LLVMCreateGenericValueOfInt(self.as_type_ref(), value, is_signed as i32)
        };

        GenericValue::new(value)
    }
}

impl AsTypeRef for IntType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.int_type.type_
    }
}
