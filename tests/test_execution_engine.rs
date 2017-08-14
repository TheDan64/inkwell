extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::execution_engine::FunctionLookupError;
use self::inkwell::targets::{InitializationConfig, Target};

#[test]
fn test_get_function_address() {
    let context = Context::create();
    let module = context.create_module("errors_abound");
    let builder = context.create_builder();
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let ee = module.create_jit_execution_engine(0);

    assert_eq!(ee.err(), Some("Unable to find target for this triple (no targets are registered)".into()));

    let module = context.create_module("errors_abound");

    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let execution_engine = module.create_jit_execution_engine(0).unwrap();

    assert_eq!(execution_engine.get_function_address("errors"), Err(FunctionLookupError::FunctionNotFound));

    let module = context.create_module("errors_abound");
    let fn_value = module.add_function("func", &fn_type, None);
    let basic_block = context.append_basic_block(&fn_value, "entry");

    builder.position_at_end(&basic_block);
    builder.build_return(None);

    let execution_engine = module.create_jit_execution_engine(0).unwrap();

    assert_eq!(execution_engine.get_function_address("errors"), Err(FunctionLookupError::FunctionNotFound));

    assert!(execution_engine.get_function_address("func").is_ok());
}

// #[test]
// fn test_get_function_value() {
//     let context = Context::create();
//     let builder = context.create_builder();
//     let module = context.create_module("errors_abound");
//     let mut execution_engine = module.create_jit_execution_engine(0).unwrap();
//     let module = execution_engine.get_module_at(0);
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

//     let execution_engine = module.create_jit_execution_engine(0).unwrap();

//     assert_eq!(execution_engine.get_function_value("errors"), Err(FunctionLookupError::FunctionNotFound));

//     assert!(execution_engine.get_function_value("func").is_ok());
// }
