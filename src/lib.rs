extern crate either;
#[macro_use]
extern crate enum_methods;
extern crate llvm_sys;

pub mod basic_block;
pub mod builder;
pub mod context;
pub mod data_layout;
pub mod execution_engine;
pub mod memory_buffer;
pub mod module;
pub mod object_file;
pub mod pass_manager;
pub mod targets;
pub mod types;
pub mod values;

// TODO: Probably move into error handling module
pub fn enable_llvm_pretty_stack_trace() {
    // use llvm_sys::error_handling::LLVMEnablePrettyStackTrace; // v3.8
    use llvm_sys::core::LLVMEnablePrettyStackTrace;

    unsafe {
        LLVMEnablePrettyStackTrace()
    }
}

// TODO: Move to better location?
// REVIEW: Not sure how safe this is. What happens when you make an llvm call after
// shutdown_llvm has been called?
pub fn shutdown_llvm() {
    use llvm_sys::core::LLVMShutdown;

    unsafe {
        LLVMShutdown()
    }
}

// Misc Notes
// Always pass a c_string.as_ptr() call into the function call directly and never
// before hand. Seems to make a huge difference (stuff stops working) otherwise

// Initializer (new) strategy:
// assert!(!val.is_null()); where null is not expected to ever occur, but Option<Self>
// when null is expected to be passed at some point
