use llvm_sys::core::{LLVMIsAConstantVector, LLVMIsAConstantDataVector, LLVMConstInsertElement, LLVMConstExtractElement};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use types::{VectorType};
use values::traits::AsValueRef;
use values::{BasicValueEnum, BasicValue, InstructionValue, Value, IntValue, MetadataValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VectorValue {
    vec_value: Value,
}

impl VectorValue {
    pub(crate) fn new(vector_value: LLVMValueRef) -> Self {
        assert!(!vector_value.is_null());

        VectorValue {
            vec_value: Value::new(vector_value)
        }
    }

    // REVIEW: Should this be !int_value.is_null() to return bool?
    pub fn is_constant_vector(&self) -> IntValue { // TSv2: IntValue<bool>
        let int_value = unsafe {
            LLVMIsAConstantVector(self.as_value_ref())
        };

        IntValue::new(int_value)
    }

    // REVIEW: Should this be !int_value.is_null() to return bool?
    pub fn is_constant_data_vector(&self) -> IntValue { // TSv2: IntValue<bool>
        let int_value = unsafe {
            LLVMIsAConstantDataVector(self.as_value_ref())
        };

        IntValue::new(int_value)
    }

    pub fn print_to_string(&self) -> &CStr {
        self.vec_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.vec_value.print_to_stderr()
    }

    pub fn get_name(&self) -> &CStr {
        self.vec_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.vec_value.set_name(name);
    }

    pub fn get_type(&self) -> VectorType {
        VectorType::new(self.vec_value.get_type())
    }

    pub fn is_null(&self) -> bool {
        self.vec_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.vec_value.is_undef()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.vec_value.as_instruction()
    }

    pub fn const_extract_element(&self, index: &IntValue) -> BasicValueEnum {
        let value = unsafe {
            LLVMConstExtractElement(self.as_value_ref(), index.as_value_ref())
        };

        BasicValueEnum::new(value)
    }

    pub fn const_insert_element(&self, index: &IntValue, value: &BasicValue) -> BasicValueEnum {
        let value = unsafe {
            LLVMConstInsertElement(self.as_value_ref(), value.as_value_ref(), index.as_value_ref())
        };

        BasicValueEnum::new(value)
    }

    pub fn has_metadata(&self) -> bool {
        self.vec_value.has_metadata()
    }

    pub fn get_metadata(&self, kind_id: u32) -> Option<MetadataValue> {
        self.vec_value.get_metadata(kind_id)
    }

    pub fn set_metadata(&self, metadata: &MetadataValue, kind_id: u32) {
        self.vec_value.set_metadata(metadata, kind_id)
    }
}

impl AsValueRef for VectorValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.vec_value.value
    }
}
