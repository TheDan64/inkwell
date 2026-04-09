use llvm_sys::orc2::{
    LLVMOrcCreateNewThreadSafeContext, LLVMOrcCreateNewThreadSafeModule, LLVMOrcDisposeThreadSafeContext,
    LLVMOrcDisposeThreadSafeModule, LLVMOrcThreadSafeContextRef, LLVMOrcThreadSafeModuleRef,
};

use crate::module::Module;

#[derive(Debug, PartialEq, Eq)]
pub struct ThreadSafeContext {
    pub(crate) ctx: LLVMOrcThreadSafeContextRef,
}

impl ThreadSafeContext {
    pub fn new() -> Self {
        unsafe {
            let ctx = LLVMOrcCreateNewThreadSafeContext();
            assert!(!ctx.is_null(), "LLVMOrcCreateNewThreadSafeContext returned a null handle");
            Self { ctx }
        }
    }
}

impl Drop for ThreadSafeContext {
    fn drop(&mut self) {
        unsafe { LLVMOrcDisposeThreadSafeContext(self.ctx) }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ThreadSafeModule<'ctx> {
    pub(crate) module: LLVMOrcThreadSafeModuleRef,
    _marker: std::marker::PhantomData<&'ctx ()>,
}

impl<'ctx> ThreadSafeModule<'ctx> {
    pub fn new(module: Module<'ctx>, tsc: &ThreadSafeContext) -> Self {
        // Module ownership is transferred to ThreadSafeModule
        let m = module.module.get();

        unsafe {
            let ts_mod = LLVMOrcCreateNewThreadSafeModule(m, tsc.ctx);
            // Prevent drop of the original module wrapper so we don't double free
            std::mem::forget(module);
            
            Self {
                module: ts_mod,
                _marker: std::marker::PhantomData,
            }
        }
    }
}

impl Drop for ThreadSafeModule<'_> {
    fn drop(&mut self) {
        unsafe { LLVMOrcDisposeThreadSafeModule(self.module) }
    }
}
