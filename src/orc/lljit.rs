use llvm_sys::orc2::lljit::{
    LLVMOrcCreateLLJITBuilder,
    LLVMOrcCreateLLJIT,
    LLVMOrcDisposeLLJIT,
    LLVMOrcLLJITAddLLVMIRModule,
    LLVMOrcLLJITGetMainJITDylib,
    LLVMOrcLLJITLookup,
    LLVMOrcLLJITRef,
    LLVMOrcLLJITBuilderRef,
};
use llvm_sys::error::LLVMGetErrorMessage;

use crate::orc::thread_safe::ThreadSafeModule;
use crate::execution_engine::UnsafeFunctionPointer;
use std::ptr;
use std::ffi::CString;

#[derive(Debug)]
pub struct LLJITBuilder {
    builder: LLVMOrcLLJITBuilderRef,
}

impl LLJITBuilder {
    pub fn new() -> Self {
        unsafe {
            Self {
                builder: LLVMOrcCreateLLJITBuilder(),
            }
        }
    }

    pub fn build(self) -> Result<LLJIT, String> {
        let mut jit: LLVMOrcLLJITRef = ptr::null_mut();
        unsafe {
            let err = LLVMOrcCreateLLJIT(&mut jit, self.builder);
            if !err.is_null() {
                let msg_ptr = LLVMGetErrorMessage(err);
                let msg = std::ffi::CStr::from_ptr(msg_ptr).to_string_lossy().into_owned();
                // Notice: llvm-sys doesn't expose LLVMDisposeErrorMessage publicly without another feature
                // but LLVMGetErrorMessage consumes the error.
                return Err(msg);
            }
            Ok(LLJIT { jit })
        }
    }
}

#[derive(Debug)]
pub struct LLJIT {
    jit: LLVMOrcLLJITRef,
}

#[derive(Debug)]
pub struct OrcJitFunction<F> {
    inner: F,
}

impl<F: Copy> OrcJitFunction<F> {
    pub unsafe fn as_raw(&self) -> F {
        self.inner
    }
}

impl LLJIT {
    pub fn add_module(&self, tsm: ThreadSafeModule) -> Result<(), String> {
        unsafe {
            let dylib = LLVMOrcLLJITGetMainJITDylib(self.jit);
            // Ownership of ThreadSafeModule is transferred to the JIT
            let module_ref = tsm.module;
            std::mem::forget(tsm);

            let err = LLVMOrcLLJITAddLLVMIRModule(self.jit, dylib, module_ref);
            if !err.is_null() {
                let msg_ptr = LLVMGetErrorMessage(err);
                let msg = std::ffi::CStr::from_ptr(msg_ptr).to_string_lossy().into_owned();
                return Err(msg);
            }
            Ok(())
        }
    }

    pub fn lookup<F>(&self, name: &str) -> Result<OrcJitFunction<F>, String>
    where
        F: UnsafeFunctionPointer,
    {
        let cstr = CString::new(name).unwrap();
        let mut addr: llvm_sys::orc2::LLVMOrcExecutorAddress = 0;
        
        unsafe {
            let err = LLVMOrcLLJITLookup(self.jit, &mut addr, cstr.as_ptr());
            if !err.is_null() {
                let msg_ptr = LLVMGetErrorMessage(err);
                let msg = std::ffi::CStr::from_ptr(msg_ptr).to_string_lossy().into_owned();
                return Err(msg);
            }
            // `addr` is generally a u64 representing the JIT'd address
            let ptr_addr = addr as usize;
            
            // Assume the user wants a function wrapping this
            let inner_f = std::mem::transmute_copy(&ptr_addr);
            Ok(OrcJitFunction { inner: inner_f })
        }
    }
}

impl Drop for LLJIT {
    fn drop(&mut self) {
        unsafe {
            let err = LLVMOrcDisposeLLJIT(self.jit);
            if !err.is_null() {
                // Log or ignore on drop
            }
        }
    }
}
