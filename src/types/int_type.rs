use llvm_sys::core::{
    LLVMConstAllOnes, LLVMConstInt, LLVMConstIntOfArbitraryPrecision, LLVMConstIntOfStringAndSize, LLVMGetIntTypeWidth,
};
use llvm_sys::execution_engine::LLVMCreateGenericValueOfInt;
use llvm_sys::prelude::LLVMTypeRef;

use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::types::traits::AsTypeRef;
use crate::types::{ArrayType, FunctionType, PointerType, Type, VectorType};
use crate::values::{ArrayValue, GenericValue, IntValue};
use crate::AddressSpace;

use crate::types::enums::BasicMetadataTypeEnum;
use std::convert::TryFrom;
use std::fmt::{self, Display};

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
    /// Is the string valid for the given radix?
    pub fn matches_str(&self, slice: &str) -> bool {
        // drop 1 optional + or -
        let slice = slice.strip_prefix(|c| c == '+' || c == '-').unwrap_or(slice);

        // there must be at least 1 actual digit
        if slice.is_empty() {
            return false;
        }

        // and all digits must be in the radix' character set
        slice.chars().all(|c| c.is_digit(*self as u32))
    }
}

/// An `IntType` is the type of an integer constant or variable.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IntType<'ctx> {
    int_type: Type<'ctx>,
}

impl<'ctx> IntType<'ctx> {
    /// Create `IntType` from [`LLVMTypeRef`]
    ///
    /// # Safety
    /// Undefined behavior, if referenced type isn't int type
    pub unsafe fn new(int_type: LLVMTypeRef) -> Self {
        assert!(!int_type.is_null());

        IntType {
            int_type: Type::new(int_type),
        }
    }

    /// Creates an `IntValue` representing a constant value of this `IntType`. It will be automatically assigned this `IntType`'s `Context`.
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
        unsafe { IntValue::new(LLVMConstInt(self.as_type_ref(), value, sign_extend as i32)) }
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
        if !radix.matches_str(slice) {
            return None;
        }

        unsafe {
            Some(IntValue::new(LLVMConstIntOfStringAndSize(
                self.as_type_ref(),
                slice.as_ptr() as *const ::libc::c_char,
                slice.len() as u32,
                radix as u8,
            )))
        }
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
        unsafe {
            IntValue::new(LLVMConstIntOfArbitraryPrecision(
                self.as_type_ref(),
                words.len() as u32,
                words.as_ptr(),
            ))
        }
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
        unsafe { IntValue::new(LLVMConstAllOnes(self.as_type_ref())) }
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
        unsafe { IntValue::new(self.int_type.const_zero()) }
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
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
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
    /// assert_eq!(i8_type.get_context(), context);
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
    /// let i8_ptr_type = i8_type.ptr_type(AddressSpace::default());
    ///
    /// #[cfg(any(
    ///     feature = "llvm4-0",
    ///     feature = "llvm5-0",
    ///     feature = "llvm6-0",
    ///     feature = "llvm7-0",
    ///     feature = "llvm8-0",
    ///     feature = "llvm9-0",
    ///     feature = "llvm10-0",
    ///     feature = "llvm11-0",
    ///     feature = "llvm12-0",
    ///     feature = "llvm13-0",
    ///     feature = "llvm14-0"
    /// ))]
    /// assert_eq!(i8_ptr_type.get_element_type().into_int_type(), i8_type);
    /// ```
    #[cfg_attr(
        any(
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ),
        deprecated(
            note = "Starting from version 15.0, LLVM doesn't differentiate between pointer types. Use Context::ptr_type instead."
        )
    )]
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
        unsafe { LLVMGetIntTypeWidth(self.as_type_ref()) }
    }

    /// Print the definition of an `IntType` to `LLVMString`.
    pub fn print_to_string(self) -> LLVMString {
        self.int_type.print_to_string()
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
        unsafe { IntValue::new(self.int_type.get_undef()) }
    }

    /// Creates a poison instance of an `IntType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    /// use inkwell::values::AnyValue;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_poison = i8_type.get_poison();
    ///
    /// assert!(i8_poison.is_poison());
    /// ```
    #[llvm_versions(12..)]
    pub fn get_poison(self) -> IntValue<'ctx> {
        unsafe { IntValue::new(self.int_type.get_poison()) }
    }

    /// Creates a `GenericValue` for use with `ExecutionEngine`s.
    pub fn create_generic_value(self, value: u64, is_signed: bool) -> GenericValue<'ctx> {
        unsafe { GenericValue::new(LLVMCreateGenericValueOfInt(self.as_type_ref(), value, is_signed as i32)) }
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
        unsafe { ArrayValue::new_const_array(&self, values) }
    }
}

unsafe impl AsTypeRef for IntType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.int_type.ty
    }
}

impl Display for IntType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
