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

// Misc Notes
// Always pass a c_string.as_ptr() call into the function call directly and never
// before hand. Seems to make a huge difference (stuff stops working) otherwise

#[test]
fn test_tari_example() {
    use context::Context;
    use std::mem::transmute;
    use targets::{InitializationConfig, Target};

    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();
    let execution_engine = module.create_execution_engine(true).unwrap();

    let i64_type = context.i64_type();
    let fn_type = i64_type.fn_type(&[&i64_type, &i64_type, &i64_type], false);

    let function = module.add_function("sum", &fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let x = function.get_nth_param(0).unwrap().into_int_value();
    let y = function.get_nth_param(1).unwrap().into_int_value();
    let z = function.get_nth_param(2).unwrap().into_int_value();

    let sum = builder.build_int_add(&x, &y, "sum");
    let sum = builder.build_int_add(&sum, &z, "sum");

    builder.build_return(Some(&sum));

    let addr = execution_engine.get_function_address("sum").unwrap();

    let sum: extern "C" fn(u64, u64, u64) -> u64 = unsafe { transmute(addr) };

    let x = 1u64;
    let y = 2u64;
    let z = 3u64;

    assert_eq!(sum(x, y, z), x + y + z);
}
