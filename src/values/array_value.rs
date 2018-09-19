use llvm_sys::core::{LLVMIsAConstantArray, LLVMIsAConstantDataArray};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;
use std::fmt;

use support::LLVMString;
use types::ArrayType;
use values::traits::AsValueRef;
use values::{Value, InstructionValue, MetadataValue};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ArrayValue {
    array_value: Value
}

impl ArrayValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        ArrayValue {
            array_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.array_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.array_value.set_name(name);
    }

    pub fn get_type(&self) -> ArrayType {
        ArrayType::new(self.array_value.get_type())
    }

    pub fn is_null(&self) -> bool {
        self.array_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.array_value.is_undef()
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.array_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.array_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.array_value.as_instruction()
    }

    pub fn has_metadata(&self) -> bool {
        self.array_value.has_metadata()
    }

    pub fn get_metadata(&self, kind_id: u32) -> Option<MetadataValue> {
        self.array_value.get_metadata(kind_id)
    }

    pub fn set_metadata(&self, metadata: MetadataValue, kind_id: u32) {
        self.array_value.set_metadata(metadata, kind_id)
    }

    pub fn replace_all_uses_with(&self, other: ArrayValue) {
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
    pub fn is_const(&self) -> bool {
        self.array_value.is_const()
    }
}

impl AsValueRef for ArrayValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.array_value.value
    }
}

impl fmt::Debug for ArrayValue {
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
