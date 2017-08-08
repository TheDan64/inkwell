use llvm_sys::core::LLVMAddIncoming;
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use basic_block::BasicBlock;
use values::traits::AsValueRef;
use values::{BasicValue, InstructionValue, Value};

#[derive(Debug, PartialEq, Eq)]
pub struct PhiValue {
    phi_value: Value
}

impl PhiValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        PhiValue {
            phi_value: Value::new(value),
        }
    }

    // REVIEW: Is incoming_values really ArrayValue? Or an &[BasicValue]?
    fn add_incoming(&self, incoming_values: &BasicValue, incoming_basic_block: &BasicBlock, count: u32) {
        let value = &mut [incoming_values.as_value_ref()];
        let basic_block = &mut [incoming_basic_block.basic_block];

        unsafe {
            LLVMAddIncoming(self.as_value_ref(), value.as_mut_ptr(), basic_block.as_mut_ptr(), count);
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.phi_value.get_name()
    }

    pub fn is_null(&self) -> bool {
        self.phi_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.phi_value.is_undef()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.phi_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.phi_value.print_to_stderr()
    }

    // REVIEW: Maybe this is should always return InstructionValue?
    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.phi_value.as_instruction()
    }
}

impl AsValueRef for PhiValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.phi_value.value
    }
}
