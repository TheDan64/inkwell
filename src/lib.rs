extern crate llvm_sys;

pub mod basic_block;
pub mod builder;
pub mod context;
pub mod data_layout;
pub mod execution_engine;
pub mod module;
pub mod pass_manager;
pub mod target_data;
pub mod types;
pub mod values;

// Misc Notes
// Always pass a c_string.as_ptr() call into the function call directly and never
// before hand. Seems to make a huge difference (stuff stops working) otherwise
