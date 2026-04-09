use llvm_sys::error::LLVMGetErrorMessage;
use llvm_sys::orc2::lljit::{
    LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcDisposeLLJIT, LLVMOrcLLJITAddLLVMIRModule,
    LLVMOrcLLJITBuilderRef, LLVMOrcLLJITGetMainJITDylib, LLVMOrcLLJITLookup, LLVMOrcLLJITRef,
};

use crate::execution_engine::UnsafeFunctionPointer;
use crate::orc::thread_safe::ThreadSafeModule;
use std::ffi::CString;
use std::ptr;

#[derive(Debug)]
pub struct LLJITBuilder {
    builder: LLVMOrcLLJITBuilderRef,
}

impl Drop for LLJITBuilder {
    fn drop(&mut self) {
        unsafe {
            llvm_sys::orc2::lljit::LLVMOrcDisposeLLJITBuilder(self.builder);
        }
    }
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
                let msg = crate::support::LLVMString::new(msg_ptr as *const libc::c_char);
                return Err(msg.to_string());
            }
            // By calling mem::forget, we prevent Drop from being called, so builder isn't double-freed.
            // (LLVMOrcCreateLLJIT takes ownership of the builder on both success and failure!)
            std::mem::forget(self);
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
                let msg = crate::support::LLVMString::new(msg_ptr as *const libc::c_char);
                return Err(msg.to_string());
            }
            Ok(())
        }
    }

    pub fn lookup<F>(&self, name: &str) -> Result<OrcJitFunction<F>, String>
    where
        F: UnsafeFunctionPointer,
    {
        assert_eq!(
            std::mem::size_of::<F>(),
            std::mem::size_of::<usize>(),
            "OrcJitFunction must be an unsafe function pointer"
        );
        let cstr = CString::new(name).unwrap();
        let mut addr: llvm_sys::orc2::LLVMOrcExecutorAddress = 0;

        unsafe {
            let err = LLVMOrcLLJITLookup(self.jit, &mut addr, cstr.as_ptr());
            if !err.is_null() {
                let msg_ptr = LLVMGetErrorMessage(err);
                let msg = crate::support::LLVMString::new(msg_ptr as *const libc::c_char);
                return Err(msg.to_string());
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
