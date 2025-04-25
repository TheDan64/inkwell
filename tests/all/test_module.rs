use inkwell::context::Context;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::targets::{Target, TargetTriple};
use inkwell::values::AnyValue;
use inkwell::OptimizationLevel;

use std::env::temp_dir;
use std::fs::{self, remove_file};
use std::path::Path;

#[test]
fn test_write_bitcode_to_path() {
    let mut path = temp_dir();

    path.push("temp.bc");

    let context = Context::create();
    let module = context.create_module("my_module");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    module.add_function("my_fn", fn_type, None);
    module.write_bitcode_to_path(&path);

    let contents = fs::read(&path).expect("Could not read back written file.");
    assert!(!contents.is_empty());

    remove_file(&path).unwrap();
}

// REVIEW: This test infrequently fails. Seems to happen more often on travis.
// Possibly a LLVM bug? Wrapper is really straightforward. See issue #6 on GH
// #[test]
// fn test_write_bitcode_to_file() {
//     use context::Context;
//     use std::env::temp_dir;
//     use std::fs::{File, remove_file};
//     use std::io::{Read, Seek, SeekFrom};

//     let mut path = temp_dir();

//     path.push("temp2.bc");

//     let mut file = File::create(&path).unwrap();

//     let context = Context::create();
//     let module = context.create_module("my_module");
//     let void_type = context.void_type();
//     let fn_type = void_type.fn_type(&[], false);

//     module.add_function("my_fn", fn_type, None);
//     module.write_bitcode_to_file(&file, true, false);

//     let mut contents = Vec::new();
//     let mut file2 = File::open(&path).expect("Could not open temp file");

//     file.read_to_end(&mut contents).expect("Unable to verify written file");

//     assert!(contents.len() > 0);

//     remove_file(&path).unwrap();
// }

#[test]
fn test_get_function() {
    let context = Context::create();
    let module = context.create_module("my_module");

    assert_eq!(module.get_context(), context);
    assert!(module.get_first_function().is_none());
    assert!(module.get_last_function().is_none());
    assert!(module.get_function("function_1").is_none());

    let functions: Vec<_> = module.get_functions().collect();
    assert!(functions.is_empty());

    let void_type = context.void_type();
    let function_1_type = void_type.fn_type(&[], false);
    let function_1 = module.add_function("function_1", function_1_type, None);

    let first_fn = module.get_first_function().unwrap();
    let last_fn = module.get_last_function().unwrap();
    let named_fn = module.get_function("function_1").unwrap();

    assert_eq!(first_fn, function_1);
    assert_eq!(last_fn, function_1);
    assert_eq!(named_fn, function_1);

    let functions: Vec<_> = module.get_functions().collect();
    assert_eq!(functions, vec![function_1]);

    let void_type = context.void_type();
    let function_2_type = void_type.fn_type(&[], false);
    let function_2 = module.add_function("function_2", function_2_type, None);

    let first_fn = module.get_first_function().unwrap();
    let last_fn = module.get_last_function().unwrap();

    assert_eq!(first_fn, function_1);
    assert_eq!(last_fn, function_2);

    let functions: Vec<_> = module.get_functions().collect();
    assert_eq!(functions, vec![function_1, function_2]);
}

#[test]
fn test_module_owned_data_layout_disposed_safely() {
    let context = Context::create();

    context.create_module("test");
}

#[test]
fn test_write_and_load_memory_buffer() {
    let context = Context::create();
    let module = context.create_module("my_module");
    let builder = context.create_builder();
    let void_type = context.void_type();
    let function_type = void_type.fn_type(&[], false);
    let function = module.add_function("my_fn", function_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);
    builder.build_return(None).unwrap();

    let memory_buffer = module.write_bitcode_to_memory();

    assert!(memory_buffer.get_size() > 0);

    let module2 = context.create_module_from_ir(memory_buffer).unwrap();

    assert_eq!(
        module2.get_function("my_fn").unwrap().print_to_string(),
        function.print_to_string()
    );

    let memory_buffer2 = module.write_bitcode_to_memory();
    let object_file = memory_buffer2.create_object_file();

    assert!(object_file.is_err());
}

#[test]
fn test_garbage_ir_fails_create_module_from_ir() {
    let context = Context::create();
    let memory_buffer = MemoryBuffer::create_from_memory_range(b"garbage ir data", "my_ir");

    assert_eq!(memory_buffer.get_size(), 15);
    assert_eq!(memory_buffer.as_slice(), b"garbage ir data");
    assert!(context.create_module_from_ir(memory_buffer).is_err());
}

#[test]
fn test_garbage_ir_fails_create_module_from_ir_copy() {
    let context = Context::create();
    let memory_buffer = MemoryBuffer::create_from_memory_range_copy(b"garbage ir data", "my_ir");

    assert_eq!(memory_buffer.get_size(), 15);
    assert_eq!(memory_buffer.as_slice(), b"garbage ir data");
    assert!(context.create_module_from_ir(memory_buffer).is_err());
}

#[test]
fn test_get_struct_type() {
    let context = Context::create();
    let module = context.create_module("my_module");

    assert_eq!(module.get_context(), context);
    assert!(module.get_struct_type("foo").is_none());

    let opaque = context.opaque_struct_type("foo");

    assert_eq!(module.get_struct_type("foo").unwrap(), opaque);
}

#[test]
fn test_get_struct_type_global_context() {
    unsafe {
        Context::get_global(|context| {
            let module = context.create_module("my_module");

            assert_eq!(module.get_context(), *context);
            assert!(module.get_struct_type("foo").is_none());

            let opaque = context.opaque_struct_type("foo");

            assert_eq!(module.get_struct_type("foo").unwrap(), opaque);
        })
    }
}

// TODO: test compile fail
// #[test]
// fn test_module_no_double_free() {
// let _module = {
//     let context = Context::create();

//     context.create_module("my_mod")
// };
// }

// #[test]
// fn test_owned_module_dropped_ee_and_context() {
// let _module = {
//     let context = Context::create();
//     let module = context.create_module("my_mod");

//     module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
//     module
// };

// Context and EE will live on in the module until here
// }

#[test]
fn test_parse_from_buffer() {
    let context = Context::create();
    let garbage_buffer = MemoryBuffer::create_from_memory_range(b"garbage ir data", "my_ir");
    let module_result = Module::parse_bitcode_from_buffer(&garbage_buffer, &context);

    assert!(module_result.is_err());

    let module = context.create_module("mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let f = module.add_function("f", fn_type, None);
    let basic_block = context.append_basic_block(f, "entry");
    let builder = context.create_builder();

    builder.position_at_end(basic_block);
    builder.build_return(None).unwrap();

    assert!(module.verify().is_ok());

    let buffer = module.write_bitcode_to_memory();
    let module2_result = Module::parse_bitcode_from_buffer(&buffer, &context);

    assert!(module2_result.is_ok());
    assert_eq!(module2_result.unwrap().get_context(), context);

    let module3_result = Module::parse_bitcode_from_buffer(&garbage_buffer, &context);

    assert!(module3_result.is_err());

    let buffer2 = module.write_bitcode_to_memory();
    let module4_result = Module::parse_bitcode_from_buffer(&buffer2, &context);

    assert!(module4_result.is_ok());
    assert_eq!(module4_result.unwrap().get_context(), context);
}

#[test]
fn test_parse_from_path() {
    let context = Context::create();
    let garbage_path = Path::new("foo/bar");
    let module_result = Module::parse_bitcode_from_path(garbage_path, &context);
    assert!(module_result.is_err(), "1");

    let module_result2 = Module::parse_bitcode_from_path(garbage_path, &context);
    assert!(module_result2.is_err(), "2");

    let module = context.create_module("mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let f = module.add_function("f", fn_type, None);
    let basic_block = context.append_basic_block(f, "entry");
    let builder = context.create_builder();

    builder.position_at_end(basic_block);
    builder.build_return(None).unwrap();

    assert!(module.verify().is_ok(), "3");

    // FIXME: Wasn't able to test success case. Got "invalid bitcode signature"
    let mut temp_path = temp_dir();

    temp_path.push("module.bc");

    module.write_bitcode_to_path(&temp_path);

    let module3_result = Module::parse_bitcode_from_path(&temp_path, &context);

    assert!(module3_result.is_ok());
    assert_eq!(module3_result.unwrap().get_context(), context);
}

#[test]
fn test_clone() {
    let context = Context::create();
    let module = context.create_module("mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let f = module.add_function("f", fn_type, None);
    let basic_block = context.append_basic_block(f, "entry");
    let builder = context.create_builder();

    builder.position_at_end(basic_block);
    builder.build_return(None).unwrap();

    let module2 = module.clone();

    assert_ne!(module, module2);
    assert_eq!(module.print_to_string(), module2.print_to_string());
}

#[test]
fn test_print_to_file() {
    let context = Context::create();
    let module = context.create_module("mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let f = module.add_function("f", fn_type, None);
    let basic_block = context.append_basic_block(f, "entry");
    let builder = context.create_builder();

    builder.position_at_end(basic_block);
    builder.build_return(None).unwrap();

    let bad_path = Path::new("/tmp/some/silly/path/that/sure/doesn't/exist");

    assert_eq!(
        module.print_to_file(bad_path).unwrap_err().to_str(),
        Ok("No such file or directory")
    );

    let mut temp_path = temp_dir();

    temp_path.push("module");

    assert!(module.print_to_file(&temp_path).is_ok());
}

#[test]
fn test_get_set_target() {
    Target::initialize_x86(&Default::default());
    let context = Context::create();
    let module = context.create_module("mod");
    let triple = TargetTriple::create("x86_64-pc-linux-gnu");

    assert_eq!(module.get_name().to_str(), Ok("mod"));
    assert_eq!(module.get_triple(), TargetTriple::create(""));

    assert_eq!(module.get_source_file_name().to_str(), Ok("mod"));

    module.set_name("mod2");
    module.set_triple(&triple);

    assert_eq!(module.get_name().to_str(), Ok("mod2"));
    assert_eq!(module.get_triple(), triple);

    module.set_source_file_name("foo.rs");

    assert_eq!(module.get_source_file_name().to_str(), Ok("foo.rs"));
    assert_eq!(module.get_name().to_str(), Ok("mod2"));
}

#[test]
fn test_linking_modules() {
    let context = Context::create();
    let module = context.create_module("mod");
    let void_type = context.void_type();
    let builder = context.create_builder();
    let fn_type = void_type.fn_type(&[], false);
    let fn_val = module.add_function("f", fn_type, None);
    let basic_block = context.append_basic_block(fn_val, "entry");

    builder.position_at_end(basic_block);
    builder.build_return(None).unwrap();

    let module2 = context.create_module("mod2");

    // Unowned module links in unowned (empty) module
    assert!(module.link_in_module(module2).is_ok());
    assert_eq!(module.get_function("f"), Some(fn_val));
    assert!(module.get_function("f2").is_none());

    let module3 = context.create_module("mod3");
    let fn_val2 = module3.add_function("f2", fn_type, None);
    let basic_block2 = context.append_basic_block(fn_val2, "entry");

    builder.position_at_end(basic_block2);
    builder.build_return(None).unwrap();

    // Unowned module links in unowned module
    assert!(module.link_in_module(module3).is_ok());
    assert_eq!(module.get_function("f"), Some(fn_val));

    // fn_val2 is no longer the same instance of f2
    assert_ne!(module.get_function("f2"), Some(fn_val2));

    let _execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .expect("Could not create Execution Engine");
    let module4 = context.create_module("mod4");

    // EE owned module links in unowned (empty) module
    assert!(module.link_in_module(module4).is_ok());

    let module5 = context.create_module("mod5");
    let fn_val3 = module5.add_function("f2", fn_type, None);
    let basic_block3 = context.append_basic_block(fn_val3, "entry");

    builder.position_at_end(basic_block3);
    builder.build_return(None).unwrap();

    // EE owned module links in unowned module which has
    // another definition for the same function name, "f2"
    assert_eq!(
        module.link_in_module(module5).unwrap_err().to_str(),
        Ok("Linking globals named \'f2\': symbol multiply defined!")
    );

    let module6 = context.create_module("mod5");
    let fn_val4 = module6.add_function("f4", fn_type, None);
    let basic_block4 = context.append_basic_block(fn_val4, "entry");

    builder.position_at_end(basic_block4);
    builder.build_return(None).unwrap();

    let execution_engine2 = module6
        .create_jit_execution_engine(OptimizationLevel::None)
        .expect("Could not create Execution Engine");

    // EE owned module cannot link another EE owned module
    assert!(module.link_in_module(module6).is_err());
    assert_eq!(execution_engine2.get_function_value("f4"), Ok(fn_val4));
}

#[test]
fn test_metadata_flags() {
    let context = Context::create();
    let module = context.create_module("my_module");

    use inkwell::module::FlagBehavior;

    assert!(module.get_flag("some_key").is_none());

    let md = context.metadata_string("lots of metadata here");

    module.add_metadata_flag("some_key", FlagBehavior::Error, md);

    // These have different addresses but same value
    assert!(module.get_flag("some_key").is_some());

    let f64_type = context.f64_type();
    let f64_val = f64_type.const_float(std::f64::consts::PI);

    assert!(module.get_flag("some_key2").is_none());

    module.add_basic_value_flag("some_key2", FlagBehavior::Error, f64_val);

    assert!(module.get_flag("some_key2").is_some());

    let struct_val = context.const_struct(&[f64_val.into()], false);

    assert!(module.get_flag("some_key3").is_none());

    module.add_basic_value_flag("some_key3", FlagBehavior::Error, struct_val);

    assert!(module.get_flag("some_key3").is_some());
    assert!(module.verify().is_ok());
}

#[test]
fn test_double_ee_from_same_module() {
    let context = Context::create();
    let module = context.create_module("mod");
    let void_type = context.void_type();
    let builder = context.create_builder();
    let fn_type = void_type.fn_type(&[], false);
    let fn_val = module.add_function("f", fn_type, None);
    let basic_block = context.append_basic_block(fn_val, "entry");

    builder.position_at_end(basic_block);
    builder.build_return(None).unwrap();

    module
        .create_execution_engine()
        .expect("Could not create Execution Engine");

    assert!(module.create_execution_engine().is_err());

    let module2 = module.clone();

    module2
        .create_jit_execution_engine(OptimizationLevel::None)
        .expect("Could not create Execution Engine");

    assert!(module.create_jit_execution_engine(OptimizationLevel::None).is_err());

    let module3 = module.clone();

    module3
        .create_interpreter_execution_engine()
        .expect("Could not create Execution Engine");

    assert!(module.create_interpreter_execution_engine().is_err());
}
