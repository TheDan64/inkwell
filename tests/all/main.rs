//! Having a main.rs in a directory w/ mods will force tests to be built in a single binary

extern crate either;
#[macro_use]
extern crate inkwell_internals;

#[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
mod test_attributes;
mod test_basic_block;
mod test_builder;
mod test_context;
#[cfg(not(any(
    feature = "llvm3-6",
    feature = "llvm3-7",
    feature = "llvm3-8",
    feature = "llvm3-9",
    feature = "llvm4-0",
    feature = "llvm5-0",
    feature = "llvm6-0"
)))]
mod test_debug_info;
mod test_execution_engine;
mod test_instruction_values;
mod test_module;
mod test_object_file;
mod test_passes;
mod test_targets;
mod test_tari_example;
mod test_types;
mod test_values;
