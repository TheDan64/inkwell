//! TODO

use values::{BasicValueEnum, InstructionValue};
use llvm_sys::core::{LLVMGetNextUse, LLVMGetUser, LLVMGetUsedValue};
use llvm_sys::prelude::LLVMUseRef;

/// TODO
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValueUse(LLVMUseRef);

impl ValueUse {
    pub(crate) fn new(use_: LLVMUseRef) -> Self {
        debug_assert!(!use_.is_null());

        ValueUse(use_)
    }

    /// TODO
    pub fn get_next_use(&self) -> Option<Self> {
        let use_ = unsafe {
            LLVMGetNextUse(self.0)
        };

        if use_.is_null() {
            return None;
        }

        Some(ValueUse::new(use_))
    }

    /// TODO
    pub fn get_user(&self) -> InstructionValue {
        let user = unsafe {
            LLVMGetUser(self.0)
        };

        InstructionValue::new(user)
    }

    /// TODO
    pub fn get_used_value(&self) -> BasicValueEnum {
        let used_value = unsafe {
            LLVMGetUsedValue(self.0)
        };

        BasicValueEnum::new(used_value)
    }
}
