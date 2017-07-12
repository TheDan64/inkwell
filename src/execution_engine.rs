use llvm_sys::execution_engine::{LLVMGetExecutionEngineTargetData, LLVMExecutionEngineRef, LLVMRunFunction, LLVMRunFunctionAsMain, LLVMDisposeExecutionEngine, LLVMGetFunctionAddress, LLVMAddModule};

use module::Module;
use target_data::TargetData;
use values::{AsLLVMValueRef, FunctionValue};

use std::ffi::CString;

pub struct ExecutionEngine {
    execution_engine: LLVMExecutionEngineRef,
    jit_mode: bool,
}

impl ExecutionEngine {
    pub(crate) fn new(execution_engine: LLVMExecutionEngineRef, jit_mode: bool) -> ExecutionEngine {
        assert!(!execution_engine.is_null());

        ExecutionEngine {
            execution_engine: execution_engine,
            jit_mode: jit_mode,
        }
    }

    pub fn add_module(&self, module: &Module) {
        unsafe {
            LLVMAddModule(self.execution_engine, module.module)
        }
    }

    /// WARNING: The returned address *will* be invalid if the EE drops first
    /// Do not attempt to transmute it to a function if the ExecutionEngine is gone
    pub fn get_function_address(&self, fn_name: &str) -> Result<u64, String> {

        if !self.jit_mode {
            return Err("ExecutionEngineError: Cannot use get_function_address in non jit_mode".into());
        }

        let c_string = CString::new(fn_name).expect("Conversion to CString failed unexpectedly");

        let address = unsafe {
            LLVMGetFunctionAddress(self.execution_engine, c_string.as_ptr())
        };

        if address == 0 {
            return Err(format!("ExecutionEngineError: Could not find function {}", fn_name));
        }

        Ok(address)
    }

    pub fn get_target_data(&self) -> TargetData {
        let target_data = unsafe {
            LLVMGetExecutionEngineTargetData(self.execution_engine)
        };

        TargetData::new(target_data)
    }

    pub fn run_function(&self, function: FunctionValue) {
        let mut args = vec![]; // TODO: Support args

        unsafe {
            LLVMRunFunction(self.execution_engine, function.as_llvm_value_ref(), args.len() as u32, args.as_mut_ptr()); // REVIEW: usize to u32 ok??
        }
    }

    pub fn run_function_as_main(&self, function: FunctionValue) {
        let args = vec![]; // TODO: Support argc, argv
        let env_p = vec![]; // REVIEW: No clue what this is

        unsafe {
            LLVMRunFunctionAsMain(self.execution_engine, function.as_llvm_value_ref(), args.len() as u32, args.as_ptr(), env_p.as_ptr()); // REVIEW: usize to u32 cast ok??
        }
    }
}

impl Drop for ExecutionEngine {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeExecutionEngine(self.execution_engine);
        }
    }
}
