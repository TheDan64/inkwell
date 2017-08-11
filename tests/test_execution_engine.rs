extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::execution_engine::FunctionLookupError;
// use self::inkwell::targets::{InitializationConfig, Target};

// use std::mem::forget;

#[test]
fn test_get_function_address() {
    let context = Context::create();
    let module = context.create_module("errors_abound");
    let builder = context.create_builder();
    let execution_engine = module.create_execution_engine(false).unwrap();

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("func", &fn_type, None);
    let basic_block = fn_value.append_basic_block("entry");

    builder.position_at_end(&basic_block);
    builder.build_return(None);

    assert_eq!(execution_engine.get_function_address("errors"), Err(FunctionLookupError::JITNotEnabled));

    // FIXME: The following results in an invalid memory access somehow. Commenting out EE drop stops it from happenening
    // which is weird because both created EEs are separate instances(verify!)
    // forget(execution_engine); // FIXME: This is a workaround so drop isn't called

    // Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    // let execution_engine = module.create_execution_engine(true).unwrap();

    // assert_eq!(execution_engine.get_function_address("errors"), Err(FunctionLookupError::FunctionNotFound));

    // // FIXME: Ocassionally fails with `cargo test test_get_function`
    // // Will either return Err(?) or SF? Inconsistent test..
    // assert!(execution_engine.get_function_address("func").is_ok());

    // forget(execution_engine); // FIXME
}

// #[test]
// fn test_get_function_value() {
//     let context = Context::create();
//     let builder = context.create_builder();
//     let module = context.create_module("errors_abound");
//     let execution_engine = module.create_execution_engine(false).unwrap();

//     let void_type = context.void_type();
//     let fn_type = void_type.fn_type(&[], false);
//     let fn_value = module.add_function("func", &fn_type, None);
//     let basic_block = fn_value.append_basic_block("entry");

//     builder.position_at_end(&basic_block);
//     builder.build_return(None);

//     assert_eq!(execution_engine.get_function_value("errors"), Err(FunctionLookupError::JITNotEnabled));

//     forget(execution_engine); // FIXME: Same issue as in test_get_function_address_errors

//     Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

//     let execution_engine = module.create_execution_engine(true).unwrap();

//     assert_eq!(execution_engine.get_function_value("errors"), Err(FunctionLookupError::FunctionNotFound));

//     assert!(execution_engine.get_function_value("func").is_ok());

//     forget(execution_engine); // FIXME
// }
