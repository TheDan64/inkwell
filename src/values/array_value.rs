use llvm_sys::core::{LLVMIsAConstantArray, LLVMIsAConstantDataArray};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;
use std::fmt;

use crate::types::ArrayType;
use crate::values::traits::{AnyValue, AsValueRef};
use crate::values::{Value, InstructionValue};

/// An `ArrayValue` is a block of contiguous constants or variables.
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct ArrayValue<'ctx> {
    array_value: Value<'ctx>,
}

impl<'ctx> ArrayValue<'ctx> {
    pub(crate) unsafe fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        ArrayValue {
            array_value: Value::new(value),
        }
    }

    /// Gets the name of an `ArrayValue`. If the value is a constant, this will
    /// return an empty string.
    pub fn get_name(&self) -> &CStr {
        self.array_value.get_name()
    }

    /// Gets the type of this `ArrayValue`.
    pub fn get_type(self) -> ArrayType<'ctx> {
        unsafe {
            ArrayType::new(self.array_value.get_type())
        }
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
}

impl AsValueRef for ArrayValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.array_value.value
    }
}

impl fmt::Debug for ArrayValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = self.print_to_string();
        let llvm_type = self.get_type();
        let name = self.get_name();
        let is_const = self.is_const();
        let is_null = self.is_null();
        let is_const_array = unsafe {
            !LLVMIsAConstantArray(self.as_value_ref()).is_null()
        };
        let is_const_data_array = unsafe {
            !LLVMIsAConstantDataArray(self.as_value_ref()).is_null()
        };

        f.debug_struct("ArrayValue")
            .field("name", &name)
            .field("address", &self.as_value_ref())
            .field("is_const", &is_const)
            .field("is_const_array", &is_const_array)
            .field("is_const_data_array", &is_const_data_array)
            .field("is_null", &is_null)
            .field("llvm_value", &llvm_value)
            .field("llvm_type",  &llvm_type)
            .finish()
    }
}
