use llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction};
use llvm_sys::core::{LLVMAddFunction, LLVMAddGlobal, LLVMCreateFunctionPassManagerForModule, LLVMDisposeMessage, LLVMDumpModule, LLVMGetNamedFunction, LLVMGetTypeByName, LLVMSetDataLayout, LLVMSetInitializer};
use llvm_sys::execution_engine::{LLVMCreateExecutionEngineForModule, LLVMLinkInInterpreter, LLVMLinkInMCJIT};
use llvm_sys::prelude::LLVMModuleRef;
use llvm_sys::target::{LLVM_InitializeNativeTarget, LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeAsmParser, LLVM_InitializeNativeDisassembler};

// REVIEW: Drop for Module? There's a LLVM method, but I read context dispose takes care of it...
use std::ffi::{CString, CStr};
use std::mem::{uninitialized, zeroed};

use data_layout::DataLayout;
use execution_engine::ExecutionEngine;
use pass_manager::PassManager;
use types::{AnyType, FunctionType, Type};
use values::{FunctionValue, Value};

pub struct Module {
    pub(crate) module: LLVMModuleRef,
}

impl Module {
    pub(crate) fn new(module: LLVMModuleRef) -> Self {
        assert!(!module.is_null());

        Module {
            module: module
        }
    }

    pub fn add_function(&self, name: &str, return_type: FunctionType) -> FunctionValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMAddFunction(self.module, c_string.as_ptr(), return_type.fn_type)
        };

        // unsafe {
        //     LLVMSetLinkage(value, LLVMCommonLinkage);
        // }

        FunctionValue::new(value)
    }

    pub fn get_function(&self, name: &str) -> Option<FunctionValue> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMGetNamedFunction(self.module, c_string.as_ptr())
        };

        if value.is_null() {
            return None;
        }

        Some(FunctionValue::new(value))
    }

    // FIXME: Return AnyType? Enum may be value to transfer ownership.
    // Maybe even a goog impl AnyType candidate
    pub fn get_type(&self, name: &str) -> Option<Type> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let type_ = unsafe {
            LLVMGetTypeByName(self.module, c_string.as_ptr())
        };

        if type_.is_null() {
            return None;
        }

        Some(Type::new(type_))
    }

    pub fn create_execution_engine(&self, jit_mode: bool) -> Result<ExecutionEngine, String> {
        let mut execution_engine = unsafe { uninitialized() };
        let mut err_str = unsafe { zeroed() };

        if jit_mode {
            unsafe {
                LLVMLinkInMCJIT();
            }
        }

        // TODO: Check that these calls are even needed
        let code = unsafe {
            LLVM_InitializeNativeTarget()
        };

        if code == 1 {
            return Err("Unknown error in initializing native target".into());
        }

        let code = unsafe {
            LLVM_InitializeNativeAsmPrinter()
        };

        if code == 1 {
            return Err("Unknown error in initializing native asm printer".into());
        }

        let code = unsafe {
            LLVM_InitializeNativeAsmParser()
        };

        if code == 1 { // REVIEW: Does parser need to go before printer?
            return Err("Unknown error in initializing native asm parser".into());
        }

        let code = unsafe {
            LLVM_InitializeNativeDisassembler()
        };

        if code == 1 {
            return Err("Unknown error in initializing native disassembler".into());
        }

        unsafe {
            LLVMLinkInInterpreter();
        }

        let code = unsafe {
            LLVMCreateExecutionEngineForModule(&mut execution_engine, self.module, &mut err_str) // Should take ownership of module
        };

        if code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(err_str);

                rust_str
            };

            return Err(rust_str);
        }

        Ok(ExecutionEngine::new(execution_engine, jit_mode))
    }

    pub fn create_function_pass_manager(&self) -> PassManager {
        let pass_manager = unsafe {
            LLVMCreateFunctionPassManagerForModule(self.module)
        };

        PassManager::new(pass_manager)
    }

    pub fn add_global(&self, type_: &AnyType, init_value: &Option<Value>, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMAddGlobal(self.module, type_.as_ref().type_, c_string.as_ptr())
        };

        if let Some(ref init_val) = *init_value {
            unsafe {
                LLVMSetInitializer(value, init_val.value)
            }
        }

        Value::new(value)
    }

    pub fn verify(&self, print: bool) -> bool {
        let err_str: *mut *mut i8 = unsafe { zeroed() };

        let action = if print {
            LLVMVerifierFailureAction::LLVMPrintMessageAction
        } else {
            LLVMVerifierFailureAction::LLVMReturnStatusAction
        };

        let code = unsafe {
            LLVMVerifyModule(self.module, action, err_str)
        };

        if code == 1 && !err_str.is_null() {
            unsafe {
                if print {
                    let rust_str = CStr::from_ptr(*err_str).to_str().unwrap();

                    println!("{}", rust_str); // FIXME: Should probably be stderr?
                }

                LLVMDisposeMessage(*err_str);
            }
        }

        code == 0
    }

    pub fn set_data_layout(&self, data_layout: DataLayout) {
        unsafe {
            LLVMSetDataLayout(self.module, data_layout.data_layout)
        }
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpModule(self.module);
        }
    }
}
