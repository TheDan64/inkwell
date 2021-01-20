use llvm_sys::core::{LLVMConstInt, LLVMConstAllOnes, LLVMGetIntTypeWidth, LLVMConstIntOfStringAndSize, LLVMConstIntOfArbitraryPrecision, LLVMConstArray};
use llvm_sys::execution_engine::LLVMCreateGenericValueOfInt;
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};
use regex::Regex;

use crate::AddressSpace;
use crate::context::ContextRef;
use crate::types::traits::AsTypeRef;
use crate::types::{Type, ArrayType, BasicTypeEnum, VectorType, PointerType, FunctionType};
use crate::values::{AsValueRef, ArrayValue, GenericValue, IntValue};

use std::convert::TryFrom;

/// How to interpret a string or digits used to construct an integer constant.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StringRadix {
    /// Binary 0 or 1
    Binary = 2,
    /// Octal 0-7
    Octal = 8,
    /// Decimal 0-9
    Decimal = 10,
    /// Hexadecimal with upper or lowercase letters up to F.
    Hexadecimal = 16,
    /// Alphanumeric, 0-9 and all 26 letters in upper or lowercase.
    Alphanumeric = 36,
}

impl TryFrom<u8> for StringRadix {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(StringRadix::Binary),
            8 => Ok(StringRadix::Octal),
            10 => Ok(StringRadix::Decimal),
            16 => Ok(StringRadix::Hexadecimal),
            36 => Ok(StringRadix::Alphanumeric),
            _ => Err(()),
        }
    }
}

impl StringRadix {
    /// Create a Regex that matches valid strings for the given radix.
    pub fn to_regex(&self) -> Regex {
        match self {
            StringRadix::Binary => Regex::new(r"^[-+]?[01]+$").unwrap(),
            StringRadix::Octal => Regex::new(r"^[-+]?[0-7]+$").unwrap(),
            StringRadix::Decimal => Regex::new(r"^[-+]?[0-9]+$").unwrap(),
            StringRadix::Hexadecimal => Regex::new(r"^[-+]?[0-9abcdefABCDEF]+$").unwrap(),
            StringRadix::Alphanumeric => Regex::new(r"^[-+]?[0-9[:alpha:]]+$").unwrap(),
        }
    }
}

/// An `IntType` is the type of an integer constant or variable.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IntType<'ctx> {
    int_type: Type<'ctx>,
}

impl<'ctx> IntType<'ctx> {
    pub(crate) fn new(int_type: LLVMTypeRef) -> Self {
        assert!(!int_type.is_null());

        IntType {
            int_type: Type::new(int_type),
        }
    }

    /// Creates an `IntValue` repesenting a constant value of this `IntType`. It will be automatically assigned this `IntType`'s `Context`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// // Local Context
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    /// let i32_value = i32_type.const_int(42, false);
    /// ```
    // TODOC: Maybe better explain sign extension
    pub fn const_int(self, value: u64, sign_extend: bool) -> IntValue<'ctx> {
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
    /// use std::convert::TryFrom;
    ///
    /// use inkwell::context::Context;
    /// use inkwell::types::StringRadix;
    /// use inkwell::values::AnyValue;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_val = i8_type.const_int_from_string("0121", StringRadix::Decimal).unwrap();
    ///
    /// assert_eq!(i8_val.print_to_string().to_string(), "i8 121");
    ///
    /// let i8_val = i8_type.const_int_from_string("0121", StringRadix::try_from(10).unwrap()).unwrap();
    ///
    /// assert_eq!(i8_val.print_to_string().to_string(), "i8 16");
    ///
    /// let i8_val = i8_type.const_int_from_string("0121", StringRadix::Binary);
    /// assert!(i8_val.is_none());
    ///
    /// let i8_val = i8_type.const_int_from_string("ABCD", StringRadix::Binary);
    /// assert!(i8_val.is_none());
    /// ```
    pub fn const_int_from_string(self, slice: &str, radix: StringRadix) -> Option<IntValue<'ctx>> {
        if !radix.to_regex().is_match(slice) {
            return None
        }
        let value = unsafe {
            LLVMConstIntOfStringAndSize(self.as_type_ref(), slice.as_ptr() as *const ::libc::c_char, slice.len() as u32, radix as u8)
        };
        Some(IntValue::new(value))
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
    pub fn const_int_arbitrary_precision(self, words: &[u64]) -> IntValue<'ctx> {
        let value = unsafe {
            LLVMConstIntOfArbitraryPrecision(self.as_type_ref(), words.len() as u32, words.as_ptr())
        };

        IntValue::new(value)
    }

    /// Creates an `IntValue` representing a constant value of all one bits of this `IntType`. It will be automatically assigned this `IntType`'s `Context`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// // Local Context
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    /// let i32_ptr_value = i32_type.const_all_ones();
    /// ```
    pub fn const_all_ones(self) -> IntValue<'ctx> {
        let value = unsafe {
            LLVMConstAllOnes(self.as_type_ref())
        };

        IntValue::new(value)
    }

    /// Creates a constant zero value of this `IntType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::values::AnyValue;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_zero = i8_type.const_zero();
    ///
    /// assert_eq!(i8_zero.print_to_string().to_string(), "i8 0");
    /// ```
    pub fn const_zero(self) -> IntValue<'ctx> {
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
    pub fn fn_type(self, param_types: &[BasicTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
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
    pub fn array_type(self, size: u32) -> ArrayType<'ctx> {
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
    pub fn vec_type(self, size: u32) -> VectorType<'ctx> {
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
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.int_type.get_context()
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
    pub fn size_of(self) -> IntValue<'ctx> {
        self.int_type.size_of().unwrap()
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
    pub fn get_alignment(self) -> IntValue<'ctx> {
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
    pub fn ptr_type(self, address_space: AddressSpace) -> PointerType<'ctx> {
        self.int_type.ptr_type(address_space)
    }

    /// Gets the bit width of an `IntType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let bool_type = context.bool_type();
    ///
    /// assert_eq!(bool_type.get_bit_width(), 1);
    /// ```
    pub fn get_bit_width(self) -> u32 {
        unsafe {
            LLVMGetIntTypeWidth(self.as_type_ref())
        }
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of an `IntType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7..=4.0)]
    pub fn print_to_stderr(self) {
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
    pub fn get_undef(self) -> IntValue<'ctx> {
        IntValue::new(self.int_type.get_undef())
    }

    /// Creates a `GenericValue` for use with `ExecutionEngine`s.
    pub fn create_generic_value(self, value: u64, is_signed: bool) -> GenericValue<'ctx> {
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
    pub fn const_array(self, values: &[IntValue<'ctx>]) -> ArrayValue<'ctx> {
        let mut values: Vec<LLVMValueRef> = values.iter()
                                                  .map(|val| val.as_value_ref())
                                                  .collect();
        let value = unsafe {
            LLVMConstArray(self.as_type_ref(), values.as_mut_ptr(), values.len() as u32)
        };

        ArrayValue::new(value)
    }
}

impl AsTypeRef for IntType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.int_type.ty
    }
}
