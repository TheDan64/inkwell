#[llvm_versions(..17)]
use llvm_sys::core::LLVMConstArray;
#[llvm_versions(17..)]
use llvm_sys::core::LLVMConstArray2 as LLVMConstArray;
use llvm_sys::core::{LLVMGetAsString, LLVMIsAConstantArray, LLVMIsAConstantDataArray, LLVMIsConstantString};
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;
use std::fmt::{self, Display};

use crate::types::{ArrayType, AsTypeRef};
use crate::values::traits::{AnyValue, AsValueRef};
use crate::values::{InstructionValue, Value};

/// An `ArrayValue` is a block of contiguous constants or variables.
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct ArrayValue<'ctx> {
    array_value: Value<'ctx>,
}

impl<'ctx> ArrayValue<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of type array.
    pub unsafe fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        ArrayValue {
            array_value: Value::new(value),
        }
    }

    /// Creates a new constant `ArrayValue` with the given type and values.
    ///
    /// # Safety
    ///
    /// `values` must be of the same type as `ty`.
    pub unsafe fn new_const_array<T: AsTypeRef, V: AsValueRef>(ty: &T, values: &[V]) -> Self {
        let values = values.iter().map(V::as_value_ref).collect::<Vec<_>>();
        Self::new_raw_const_array(ty.as_type_ref(), &values)
    }

    /// Creates a new constant `ArrayValue` with the given type and values.
    ///
    /// # Safety
    ///
    /// `values` must be of the same type as `ty`.
    pub unsafe fn new_raw_const_array(ty: LLVMTypeRef, values: &[LLVMValueRef]) -> Self {
        unsafe { Self::new(LLVMConstArray(ty, values.as_ptr().cast_mut(), values.len() as _)) }
    }

    /// Get name of the `ArrayValue`. If the value is a constant, this will
    /// return an empty string.
    pub fn get_name(&self) -> &CStr {
        self.array_value.get_name()
    }

    /// Set name of the `ArrayValue`.
    pub fn set_name(&self, name: &str) {
        self.array_value.set_name(name)
    }

    /// Gets the type of this `ArrayValue`.
    pub fn get_type(self) -> ArrayType<'ctx> {
        unsafe { ArrayType::new(self.array_value.get_type()) }
    }

    /// Determines whether or not this value is null.
    pub fn is_null(self) -> bool {
        self.array_value.is_null()
    }

    /// Determines whether or not this value is undefined.
    pub fn is_undef(self) -> bool {
        self.array_value.is_undef()
    }

    /// Prints this `ArrayValue` to standard error.
    pub fn print_to_stderr(self) {
        self.array_value.print_to_stderr()
    }

    /// Attempt to convert this `ArrayValue` to an `InstructionValue`, if possible.
    pub fn as_instruction(self) -> Option<InstructionValue<'ctx>> {
        self.array_value.as_instruction()
    }

    /// Replaces all uses of this value with another value of the same type.
    /// If used incorrectly this may result in invalid IR.
    pub fn replace_all_uses_with(self, other: ArrayValue<'ctx>) {
        self.array_value.replace_all_uses_with(other.as_value_ref())
    }

    /// Determines whether or not an `ArrayValue` is a constant.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i64_type = context.i64_type();
    /// let i64_val = i64_type.const_int(23, false);
    /// let array_val = i64_type.const_array(&[i64_val]);
    ///
    /// assert!(array_val.is_const());
    /// ```
    pub fn is_const(self) -> bool {
        self.array_value.is_const()
    }

    /// Determines whether or not an `ArrayValue` represents a constant array of `i8`s.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let string = context.const_string(b"my_string", false);
    ///
    /// assert!(string.is_const_string());
    /// ```
    // SubTypes: Impl only for ArrayValue<IntValue<i8>>
    pub fn is_const_string(self) -> bool {
        unsafe { LLVMIsConstantString(self.as_value_ref()) == 1 }
    }

    /// Obtain the string from the ArrayValue
    /// if the value points to a constant string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use std::ffi::CStr;
    ///
    /// let context = Context::create();
    /// let string = context.const_string(b"hello!", false);
    ///
    /// let result = b"hello!".as_slice();
    /// assert_eq!(string.as_const_string(), Some(result));
    /// ```
    // SubTypes: Impl only for ArrayValue<IntValue<i8>>
    pub fn as_const_string(&self) -> Option<&[u8]> {
        let mut len = 0;
        let ptr = unsafe { LLVMGetAsString(self.as_value_ref(), &mut len) };

        if ptr.is_null() {
            None
        } else {
            unsafe { Some(std::slice::from_raw_parts(ptr.cast(), len)) }
        }
    }

    /// Obtain the string from the ArrayValue
    /// if the value points to a constant string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use std::ffi::CStr;
    ///
    /// let context = Context::create();
    /// let string = context.const_string(b"hello!", true);
    ///
    /// let result = CStr::from_bytes_with_nul(b"hello!\0").unwrap();
    /// assert_eq!(string.get_string_constant(), Some(result));
    /// ```
    // SubTypes: Impl only for ArrayValue<IntValue<i8>>
    #[deprecated = "llvm strings can contain internal NULs, and this function truncates such values, use as_const_string instead"]
    pub fn get_string_constant(&self) -> Option<&CStr> {
        let mut len = 0;
        let ptr = unsafe { LLVMGetAsString(self.as_value_ref(), &mut len) };

        if ptr.is_null() {
            None
        } else {
            unsafe { Some(CStr::from_ptr(ptr)) }
        }
    }
}

unsafe impl AsValueRef for ArrayValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.array_value.value
    }
}

impl Display for ArrayValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl fmt::Debug for ArrayValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = self.print_to_string();
        let llvm_type = self.get_type();
        let name = self.get_name();
        let is_const = self.is_const();
        let is_null = self.is_null();
        let is_const_array = unsafe { !LLVMIsAConstantArray(self.as_value_ref()).is_null() };
        let is_const_data_array = unsafe { !LLVMIsAConstantDataArray(self.as_value_ref()).is_null() };

        f.debug_struct("ArrayValue")
            .field("name", &name)
            .field("address", &self.as_value_ref())
            .field("is_const", &is_const)
            .field("is_const_array", &is_const_array)
            .field("is_const_data_array", &is_const_data_array)
            .field("is_null", &is_null)
            .field("llvm_value", &llvm_value)
            .field("llvm_type", &llvm_type)
            .finish()
    }
}
