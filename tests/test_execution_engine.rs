extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::execution_engine::GetFunctionAddressError;
// use self::inkwell::targets::{InitializationConfig, Target};

#[test]
fn test_get_function_address_errors() {
    let context = Context::create();
    let module = context.create_module("errors_abound");
    let execution_engine = module.create_execution_engine(false).unwrap();

    assert_eq!(execution_engine.get_function_address("errors"), Err(GetFunctionAddressError::JITNotEnabled));

    // FIXME: The following results in an invalid memory access somehow. Commenting out EE drop stops it from happenening
    // which is weird because both created EEs are separate instances

    // Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    // let execution_engine = module.create_execution_engine(true).unwrap();

    // assert_eq!(execution_engine.get_function_address("errors"), Err(GetFunctionAddressError::FunctionNotFound));
}
