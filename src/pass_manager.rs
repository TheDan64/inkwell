use llvm_sys::core::{LLVMDisposePassManager, LLVMInitializeFunctionPassManager};
use llvm_sys::prelude::LLVMPassManagerRef;
use llvm_sys::target::LLVMAddTargetData;
use llvm_sys::transforms::scalar::LLVMAddMemCpyOptPass;

use target_data::TargetData;

pub struct PassManager {
    pass_manager: LLVMPassManagerRef,
}

impl PassManager {
    pub(crate) fn new(pass_manager: LLVMPassManagerRef) -> PassManager {
        assert!(!pass_manager.is_null());

        PassManager {
            pass_manager: pass_manager
        }
    }

    pub fn initialize(&self) -> bool {
        // return true means some pass modified the module, not an error occurred
        unsafe {
            LLVMInitializeFunctionPassManager(self.pass_manager) == 1
        }
    }

    pub fn add_optimize_memcpy_pass(&self) {
        unsafe {
            LLVMAddMemCpyOptPass(self.pass_manager)
        }
    }

    pub fn add_target_data(&self, target_data: TargetData) {
        unsafe {
            LLVMAddTargetData(target_data.target_data, self.pass_manager)
        }
    }
}

impl Drop for PassManager {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposePassManager(self.pass_manager)
        }
    }
}
