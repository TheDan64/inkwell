// Installs an error handler to be called before LLVM exits
// REVIEW: Maybe it's possible to have a safe wrapper? If we can
// wrap the provided function input ptr into a &CStr somehow
// TODOC: Can be used like this:
// extern "C" fn print_before_exit(msg: *const i8) {
//    let c_str = unsafe { ::std::ffi::CStr::from_ptr(msg) };
//
//    panic!("LLVM fatally errored: {:?}", c_str);
// }
// unsafe {
//     install_fatal_error_handler(print_before_exit);
// }
// and will be called before LLVM calls C exit(). IIRC
// it's safe to panic from C in newer versions of rust
pub unsafe fn install_fatal_error_handler(handler: extern "C" fn(*const i8)) {
    #[cfg(any(feature = "llvm3-6", feature = "llvm3-7"))]
    use llvm_sys::core::LLVMInstallFatalErrorHandler;
    #[cfg(any(feature = "llvm3-8", feature = "llvm3-9", feature = "llvm4-0", feature = "llvm5-0"))]
    use llvm_sys::error_handling::LLVMInstallFatalErrorHandler;

    LLVMInstallFatalErrorHandler(Some(handler))
}

/// Resets LLVM's fatal error handler back to the default
pub fn reset_fatal_error_handler() {
    #[cfg(any(feature = "llvm3-6", feature = "llvm3-7"))]
    use llvm_sys::core::LLVMResetFatalErrorHandler;
    #[cfg(any(feature = "llvm3-8", feature = "llvm3-9", feature = "llvm4-0", feature = "llvm5-0"))]
    use llvm_sys::error_handling::LLVMResetFatalErrorHandler;

    unsafe {
        LLVMResetFatalErrorHandler()
    }
}
