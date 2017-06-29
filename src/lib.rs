extern crate llvm_sys;

// TODO: Remove allow dead code

#[allow(dead_code)]
pub mod basic_block;
#[allow(dead_code)]
pub mod builder;
#[allow(dead_code)]
pub mod context;
#[allow(dead_code)]
pub mod data_layout;
#[allow(dead_code)]
pub mod execution_engine;
#[allow(dead_code)]
pub mod module;
#[allow(dead_code)]
pub mod pass_manager;
#[allow(dead_code)]
pub mod target_data;
#[allow(dead_code)]
pub mod types;
#[allow(dead_code)]
pub mod values;

// Misc Notes
// Always pass a c_string.as_ptr() call into the function call directly and never
// before hand. Seems to make a huge difference (stuff stops working) otherwise
