use llvm_sys::core::{LLVMAddIncoming, LLVMCountIncoming, LLVMGetIncomingBlock, LLVMGetIncomingValue};
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMValueRef};

use std::ffi::CStr;

use basic_block::BasicBlock;
use values::traits::AsValueRef;
use values::{BasicValue, BasicValueEnum, InstructionValue, Value};

// REVIEW: Metadata for phi values?
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

    pub fn add_incoming(&self, incoming: &[(&BasicValue, &BasicBlock)]) {
        let (mut values, mut basic_blocks): (Vec<LLVMValueRef>, Vec<LLVMBasicBlockRef>) = {
            incoming.iter()
                    .map(|&(v, bb)| (v.as_value_ref(), bb.basic_block))
                    .unzip()
        };

        unsafe {
            LLVMAddIncoming(self.as_value_ref(), values.as_mut_ptr(), basic_blocks.as_mut_ptr(), incoming.len() as u32);
        }
    }

    pub fn count_incoming(&self) -> u32 {
        unsafe {
            LLVMCountIncoming(self.as_value_ref())
        }
    }

    pub fn get_incoming(&self, index: u32) -> Option<(BasicValueEnum, BasicBlock)> {
        if index >= self.count_incoming() {
            return None;
        }

        let basic_block = unsafe {
            LLVMGetIncomingBlock(self.as_value_ref(), index)
        };
        let value = unsafe {
            LLVMGetIncomingValue(self.as_value_ref(), index)
        };

        Some((BasicValueEnum::new(value), BasicBlock::new(basic_block).expect("Invalid BasicBlock")))
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

    pub fn replace_all_uses_with(&self, other: &PhiValue) {
        self.phi_value.replace_all_uses_with(other.as_value_ref())
    }

    pub fn as_basic_value(&self) -> BasicValueEnum {
        BasicValueEnum::new(self.as_value_ref())
    }
}

impl AsValueRef for PhiValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.phi_value.value
    }
}
