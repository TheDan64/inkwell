//! Having a main.rs in a directory w/ mods will force tests to be built in a single binary

#[macro_use]
extern crate inkwell_internals;

mod test_attributes;
mod test_basic_block;
mod test_builder;
mod test_context;
#[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
mod test_debug_info;
mod test_execution_engine;
mod test_instruction_conversion;
mod test_instruction_values;
mod test_intrinsics;
mod test_module;
mod test_object_file;
#[cfg(not(any(feature = "llvm17-0", feature = "llvm18-0")))]
mod test_passes;
mod test_targets;
mod test_tari_example;
mod test_types;
mod test_values;
