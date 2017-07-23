use llvm_sys::execution_engine::{LLVMGetExecutionEngineTargetData, LLVMExecutionEngineRef, LLVMRunFunction, LLVMRunFunctionAsMain, LLVMDisposeExecutionEngine, LLVMGetFunctionAddress, LLVMAddModule};

use module::Module;
use targets::TargetData;
use values::{AsValueRef, FunctionValue};

use std::ffi::CString;

#[derive(Debug, PartialEq)]
pub enum GetFunctionAddressError {
    JITNotEnabled,
    FunctionNotFound, // 404!
}

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
    pub fn get_function_address(&self, fn_name: &str) -> Result<u64, GetFunctionAddressError> {
        if !self.jit_mode {
            return Err(GetFunctionAddressError::JITNotEnabled);
        }

        let c_string = CString::new(fn_name).expect("Conversion to CString failed unexpectedly");

        let address = unsafe {
            LLVMGetFunctionAddress(self.execution_engine, c_string.as_ptr())
        };

        // REVIEW: Can also return 0 if no targets are initialized.
        // One option might be to set a global to true if any at all of the targets have been
        // initialized (maybe we could figure out which config in particular is the trigger)
        // and if not return an "NoTargetsInitialized" error, instead of not found.
        if address == 0 {
            return Err(GetFunctionAddressError::FunctionNotFound);
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
            LLVMRunFunction(self.execution_engine, function.as_value_ref(), args.len() as u32, args.as_mut_ptr()); // REVIEW: usize to u32 ok??
        }
    }

    pub fn run_function_as_main(&self, function: FunctionValue) {
        let args = vec![]; // TODO: Support argc, argv
        let env_p = vec![]; // REVIEW: No clue what this is

        unsafe {
            LLVMRunFunctionAsMain(self.execution_engine, function.as_value_ref(), args.len() as u32, args.as_ptr(), env_p.as_ptr()); // REVIEW: usize to u32 cast ok??
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

#[test]
fn test_get_function_address_errors() {
    use context::Context;
    use targets::{InitializationConfig, Target};

    let context = Context::create();
    let module = context.create_module("errors_abound");
    let execution_engine = module.create_execution_engine(false).unwrap();

    assert_eq!(execution_engine.get_function_address("errors"), Err(GetFunctionAddressError::JITNotEnabled));

    // FIXME: The following results in an invalid memory access somehow. Commenting out EE drop stops it from happenening
    // which is weird because both created EEs are separate instances

    // Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    // let execution_engine = module.create_execution_engine(true).unwrap();

    // assert_eq!(execution_engine.get_function_address("errors"), Err(GetFunctionAddressError::FunctionNotFound));
}
