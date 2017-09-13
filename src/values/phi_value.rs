use llvm_sys::core::LLVMAddIncoming;
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMValueRef};

use std::ffi::CStr;

use basic_block::BasicBlock;
use values::traits::AsValueRef;
use values::{BasicValue, InstructionValue, Value};

// REVIEW: Metadata for phi values?
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

    fn add_incoming(&self, incoming: &[(&BasicValue, &BasicBlock)]) {
        let (mut values, mut basic_blocks): (Vec<LLVMValueRef>, Vec<LLVMBasicBlockRef>) = {
            incoming.iter()
                    .map(|&(v, bb)| (v.as_value_ref(), bb.basic_block))
                    .unzip()
        };

        unsafe {
            LLVMAddIncoming(self.as_value_ref(), values.as_mut_ptr(), basic_blocks.as_mut_ptr(), incoming.len() as u32);
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
