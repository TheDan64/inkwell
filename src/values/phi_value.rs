use llvm_sys::core::{LLVMAddIncoming, LLVMCountIncoming, LLVMGetIncomingBlock, LLVMGetIncomingValue};
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMValueRef};

use std::ffi::CStr;

use crate::basic_block::BasicBlock;
use crate::values::traits::AsValueRef;
use crate::values::{BasicValue, BasicValueEnum, InstructionValue, Value};

// REVIEW: Metadata for phi values?
/// A Phi Instruction returns a value based on which basic block branched into
/// the Phi's containing basic block.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PhiValue<'ctx> {
    phi_value: Value<'ctx>,
}

impl<'ctx> PhiValue<'ctx> {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        PhiValue {
            phi_value: Value::new(value),
        }
    }

    pub fn add_incoming(self, incoming: &[(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)]) {
        let (mut values, mut basic_blocks): (Vec<LLVMValueRef>, Vec<LLVMBasicBlockRef>) = {
            incoming.iter()
                    .map(|&(v, bb)| (v.as_value_ref(), bb.basic_block))
                    .unzip()
        };

        unsafe {
            LLVMAddIncoming(self.as_value_ref(), values.as_mut_ptr(), basic_blocks.as_mut_ptr(), incoming.len() as u32);
        }
    }

    pub fn count_incoming(self) -> u32 {
        unsafe {
            LLVMCountIncoming(self.as_value_ref())
        }
    }

    pub fn get_incoming(self, index: u32) -> Option<(BasicValueEnum<'ctx>, BasicBlock<'ctx>)> {
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

    /// Gets the name of a `ArrayValue`. If the value is a constant, this will
    /// return an empty string.
    pub fn get_name(&self) -> &CStr {
        self.phi_value.get_name()
    }

    // I believe PhiValue is never a constant, so this should always work
    pub fn set_name(self, name: &str) {
        self.phi_value.set_name(name);
    }

    pub fn is_null(self) -> bool {
        self.phi_value.is_null()
    }

    pub fn is_undef(self) -> bool {
        self.phi_value.is_undef()
    }

    // SubType: -> InstructionValue<Phi>
    pub fn as_instruction(self) -> InstructionValue<'ctx> {
        self.phi_value.as_instruction().expect("PhiValue should always be a Phi InstructionValue")
    }

    pub fn replace_all_uses_with(self, other: &PhiValue<'ctx>) {
        self.phi_value.replace_all_uses_with(other.as_value_ref())
    }

    pub fn as_basic_value(self) -> BasicValueEnum<'ctx> {
        BasicValueEnum::new(self.as_value_ref())
    }
}

impl AsValueRef for PhiValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.phi_value.value
    }
}
