use llvm_sys::debuginfo::{LLVMDebugMetadataVersion, LLVMDisposeDIBuilder};
use llvm_sys::prelude::LLVMDIBuilderRef;

/// Gets the version of debug metadata produced by the current LLVM version.
pub fn debug_metadata_version() -> libc::c_uint {
    unsafe {
        LLVMDebugMetadataVersion()
    }
}

#[derive(Debug)]
pub struct DebugInfoBuilder(LLVMDIBuilderRef);

impl DebugInfoBuilder {
    #[cfg(feature = "experimental")]
    pub(crate) fn new(dib: LLVMDIBuilderRef) -> Self {
        debug_assert!(!dib.is_null());

        DebugInfoBuilder(dib)
    }
}

impl Drop for DebugInfoBuilder {
    fn drop(&mut self) {
        // FIXME: Docs say DIB must be finalize before calling this
        unsafe {
            LLVMDisposeDIBuilder(self.0)
        }
    }
}
