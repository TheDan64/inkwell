extern crate inkwell;

use self::inkwell::OptimizationLevel;
use self::inkwell::context::Context;
use self::inkwell::execution_engine::FunctionLookupError;
use self::inkwell::targets::{InitializationConfig, Target};

type Thunk = unsafe extern "C" fn();

#[test]
fn test_get_function_address() {
    let context = Context::create();
    // let module = context.create_module("errors_abound");
    let builder = context.create_builder();
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    // FIXME: LLVM's global state is leaking, causing this to fail in `cargo test` but not `cargo test test_get_function_address`
    // nor (most of the time) with `cargo test -- --test-threads LARGE_NUM`
    // assert_eq!(module.create_jit_execution_engine(OptimizationLevel::None), Err("Unable to find target for this triple (no targets are registered)".into()));
    let module = context.create_module("errors_abound");

    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    // REVIEW: Here and at end of function; LLVM 5 doesn't seem to like getting a function address which does not exist
    // and crashes/exits with "LLVM Error: (blank)"
    #[cfg(not(feature = "llvm5-0"))]
    unsafe {
        assert_eq!(execution_engine.get_function::<Thunk>("errors").unwrap_err(),
            FunctionLookupError::FunctionNotFound);
    }

    let module = context.create_module("errors_abound");
    let fn_value = module.add_function("func", &fn_type, None);
    let basic_block = context.append_basic_block(&fn_value, "entry");

    builder.position_at_end(&basic_block);
    builder.build_return(None);

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    unsafe {
        // REVIEW: See earlier remark on get function address
        #[cfg(not(feature = "llvm5-0"))]
        assert_eq!(execution_engine.get_function::<Thunk>("errors").unwrap_err(),
            FunctionLookupError::FunctionNotFound);

        assert!(execution_engine.get_function::<Thunk>("func").is_ok());
    }
}

#[test]
fn test_execution_engine() {
    let context = Context::create();
    let module = context.create_module("errors_abound");

    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    assert!(module.create_execution_engine().is_ok());
}

#[test]
fn test_interpreter_execution_engine() {
    let context = Context::create();
    let module = context.create_module("errors_abound");

    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    assert!(module.create_interpreter_execution_engine().is_ok());
}

// #[test]
// fn test_get_function_value() {
//     let context = Context::create();
//     let builder = context.create_builder();
//     let module = context.create_module("errors_abound");
//     // let mut execution_engine = ExecutionEngine::create_jit_from_module(module, 0);
//     let mut execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
//     let void_type = context.void_type();
//     let fn_type = void_type.fn_type(&[], false);
//     let fn_value = module.add_function("func", &fn_type, None);
//     let basic_block = context.append_basic_block(&fn_value, "entry");

//     builder.position_at_end(&basic_block);
//     builder.build_return(None);

//     assert_eq!(execution_engine.get_function_value("errors"), Err(FunctionLookupError::JITNotEnabled));

//     // Regain ownership of module
//     let module = execution_engine.remove_module(&module).unwrap();

//     Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

//     let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

//     assert_eq!(execution_engine.get_function_value("errors"), Err(FunctionLookupError::FunctionNotFound));

//     assert!(execution_engine.get_function_value("func").is_ok());
// }
