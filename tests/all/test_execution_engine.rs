extern crate inkwell;

use self::inkwell::{AddressSpace, OptimizationLevel, IntPredicate};
use self::inkwell::context::Context;
use self::inkwell::execution_engine::FunctionLookupError;
use self::inkwell::targets::{InitializationConfig, Target};

// use std::ffi::CString;

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

    unsafe {
        assert_eq!(execution_engine.get_function::<Thunk>("errors").unwrap_err(),
            FunctionLookupError::FunctionNotFound);
    }

    let module = context.create_module("errors_abound");
    let fn_value = module.add_function("func", fn_type, None);
    let basic_block = context.append_basic_block(&fn_value, "entry");

    builder.position_at_end(&basic_block);
    builder.build_return(None);

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    unsafe {
        assert_eq!(execution_engine.get_function::<Thunk>("errors").unwrap_err(),
            FunctionLookupError::FunctionNotFound);

        assert!(execution_engine.get_function::<Thunk>("func").is_ok());
    }
}

#[test]
fn test_jit_execution_engine() {
    let context = Context::create();
    let module = context.create_module("main_module");
    let builder = context.create_builder();
    let i8_type = context.i8_type();
    let i32_type = context.i32_type();
    let i8_ptr_type = i8_type.ptr_type(AddressSpace::Generic);
    let i8_ptr_ptr_type = i8_ptr_type.ptr_type(AddressSpace::Generic);
    let one_i32 = i32_type.const_int(1, false);
    let three_i32 = i32_type.const_int(3, false);
    let fourtytwo_i32 = i32_type.const_int(42, false);
    let fn_type = i32_type.fn_type(&[i32_type.into(), i8_ptr_ptr_type.into()], false);
    let fn_value = module.add_function("main", fn_type, None);
    let main_argc = fn_value.get_first_param().unwrap().into_int_value();
    // let main_argv = fn_value.get_nth_param(1).unwrap();
    let check_argc = context.append_basic_block(&fn_value, "check_argc");
    let check_arg3 = context.append_basic_block(&fn_value, "check_arg3");
    let error1 = context.append_basic_block(&fn_value, "error1");
    let success = context.append_basic_block(&fn_value, "success");

    main_argc.set_name("argc");

    // If anything goes wrong, jump to returning 1
    builder.position_at_end(&error1);
    builder.build_return(Some(&one_i32));

    // If successful, jump to returning 42
    builder.position_at_end(&success);
    builder.build_return(Some(&fourtytwo_i32));

    // See if argc == 3
    builder.position_at_end(&check_argc);

    let eq = IntPredicate::EQ;
    let argc_check = builder.build_int_compare(eq, main_argc, three_i32, "argc_cmp");

    builder.build_conditional_branch(argc_check, &check_arg3, &error1);

    builder.position_at_end(&check_arg3);
    builder.build_unconditional_branch(&success);

    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).expect("Could not create Execution Engine");

    let main = execution_engine.get_function_value("main").expect("Could not find main in ExecutionEngine");

    let ret = unsafe {
        execution_engine.run_function_as_main(&main, &["input", "bar"])
    };

    assert_eq!(ret, 1, "unexpected main return code: {}", ret);

    let ret = unsafe {
        execution_engine.run_function_as_main(&main, &["input", "bar", "baz"])
    };

    assert_eq!(ret, 42, "unexpected main return code: {}", ret);
}

// #[test]
// fn test_execution_engine_empty_module() {
//     let context = Context::create();
//     let module = context.create_module("fooo");
//     let builder = context.create_builder();

//     let ee = module.create_jit_execution_engine(OptimizationLevel::None); // Segfault?
// }

#[test]
fn test_execution_engine() {
    let context = Context::create();
    let module = context.create_module("main_module");

    assert!(module.create_execution_engine().is_ok());
}

#[test]
fn test_interpreter_execution_engine() {
    let context = Context::create();
    let module = context.create_module("main_module");

    assert!(module.create_interpreter_execution_engine().is_ok());
}


#[test]
fn test_add_remove_module() {
    let context = Context::create();
    let module = context.create_module("test");
    let ee = module.create_jit_execution_engine(OptimizationLevel::default()).unwrap();

    assert!(ee.add_module(&module).is_err());

    let module2 = context.create_module("mod2");

    assert!(ee.remove_module(&module2).is_err());
    assert!(ee.add_module(&module2).is_ok());
    assert!(ee.remove_module(&module).is_ok());
    assert!(ee.remove_module(&module2).is_ok());
}

// REVIEW: Global state pollution access tests cause this to pass when run individually
// but fail when multiple tests are run
// #[test]
// fn test_no_longer_segfaults() {
//     #[cfg(feature = "llvm3-6")]
//     Target::initialize_r600(&InitializationConfig::default());
//     #[cfg(not(feature = "llvm3-6"))]
//     Target::initialize_amd_gpu(&InitializationConfig::default());


//     let context = Context::create();
//     let module = context.create_module("test");

//     assert_eq!(*module.create_jit_execution_engine(OptimizationLevel::default()).unwrap_err(), *CString::new("No available targets are compatible with this triple.").unwrap());

//     // REVIEW: Module is being cloned in err case... should test Module still works as expected...
// }

// #[test]
// fn test_get_function_value() {
//     let context = Context::create();
//     let builder = context.create_builder();
//     let module = context.create_module("errors_abound");
//     // let mut execution_engine = ExecutionEngine::create_jit_from_module(module, 0);
//     let mut execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
//     let void_type = context.void_type();
//     let fn_type = void_type.fn_type(&[], false);
//     let fn_value = module.add_function("func", fn_type, None);
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


#[test]
fn test_previous_double_free() {
    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let context = Context::create();
    let module = context.create_module("sum");
    let _ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    drop(context);
    drop(module);
}

#[test]
fn test_previous_double_free2() {
    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let _execution_engine = {
        let context = Context::create();
        let module = context.create_module("sum");

        module.create_jit_execution_engine(OptimizationLevel::None).unwrap()
    };
}
