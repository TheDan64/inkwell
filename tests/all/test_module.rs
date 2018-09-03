extern crate inkwell;

use self::inkwell::OptimizationLevel;
use self::inkwell::context::Context;
use self::inkwell::memory_buffer::MemoryBuffer;
use self::inkwell::module::Module;
use self::inkwell::targets::{InitializationConfig, Target};

use std::env::temp_dir;
use std::ffi::CString;
use std::fs::{File, remove_file};
use std::io::Read;
use std::path::Path;
use std::str::from_utf8;

#[test]
fn test_write_bitcode_to_path() {
    let mut path = temp_dir();

    path.push("temp.bc");

    let context = Context::create();
    let module = context.create_module("my_module");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    module.add_function("my_fn", &fn_type, None);
    module.write_bitcode_to_path(&path);

    let mut contents = Vec::new();
    let mut file = File::open(&path).expect("Could not open temp file");

    file.read_to_end(&mut contents).expect("Unable to verify written file");

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

//     module.add_function("my_fn", &fn_type, None);
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

    assert_eq!(*module.get_context(), context);
    assert!(module.get_first_function().is_none());
    assert!(module.get_last_function().is_none());
    assert!(module.get_function("some_fn").is_none());

    let void_type = context.void_type();
    let some_fn_type = void_type.fn_type(&[], false);
    let some_fn = module.add_function("some_fn", &some_fn_type, None);
    let first_fn = module.get_first_function().unwrap();
    let last_fn = module.get_last_function().unwrap();
    let named_fn = module.get_function("some_fn").unwrap();

    assert_eq!(first_fn, some_fn);
    assert_eq!(last_fn, some_fn);
    assert_eq!(named_fn, some_fn);
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
    let function = module.add_function("my_fn", &function_type, None);
    let basic_block = function.append_basic_block("entry");

    builder.position_at_end(&basic_block);
    builder.build_return(None);

    let memory_buffer = module.write_bitcode_to_memory();

    assert!(memory_buffer.get_size() > 0);

    // REVIEW: This returns a null ptr or segfaults :(
    // let object_file = memory_buffer.create_object_file();

    let module2 = context.create_module_from_ir(memory_buffer).unwrap();

    assert_eq!(module2.get_function("my_fn").unwrap().print_to_string(), function.print_to_string());
}

#[test]
fn test_garbage_ir_fails_create_module_from_ir() {
    let context = Context::create();
    let memory_buffer = MemoryBuffer::create_from_memory_range("garbage ir data", "my_ir");

    assert_eq!(memory_buffer.get_size(), 15);
    assert_eq!(from_utf8(memory_buffer.as_slice()).unwrap(), "garbage ir data");
    assert!(context.create_module_from_ir(memory_buffer).is_err());
}

#[test]
fn test_garbage_ir_fails_create_module_from_ir_copy() {
    let context = Context::create();
    let memory_buffer = MemoryBuffer::create_from_memory_range_copy("garbage ir data", "my_ir");

    assert_eq!(memory_buffer.get_size(), 15);
    assert_eq!(from_utf8(memory_buffer.as_slice()).unwrap(), "garbage ir data");
    assert!(context.create_module_from_ir(memory_buffer).is_err());
}

#[test]
fn test_get_type() {
    let context = Context::create();
    let module = context.create_module("my_module");

    assert_eq!(*module.get_context(), context);
    assert!(module.get_type("foo").is_none());

    let opaque = context.opaque_struct_type("foo");

    assert_eq!(module.get_type("foo").unwrap().into_struct_type(), opaque);
}

#[test]
fn test_get_type_global_context() {
    let context = Context::get_global();
    let module = Module::create("my_module");

    assert_eq!(module.get_context(), context);
    assert!(module.get_type("foo").is_none());

    let opaque = context.opaque_struct_type("foo");

    assert_eq!(module.get_type("foo").unwrap().into_struct_type(), opaque);
}

#[test]
fn test_module_no_double_free() {
    let _module = {
        let context = Context::create();

        context.create_module("my_mod")
    };

    // Context will live on in the module until here
}

#[test]
fn test_owned_module_dropped_ee_and_context() {
    Target::initialize_native(&InitializationConfig::default()).unwrap();

    let _module = {
        let context = Context::create();
        let module = context.create_module("my_mod");

        module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
        module
    };

    // Context and EE will live on in the module until here
}

#[test]
fn test_parse_from_buffer() {
    let context = Context::create();
    let garbage_buffer = MemoryBuffer::create_from_memory_range("garbage ir data", "my_ir");
    let module_result = Module::parse_bitcode_from_buffer(&garbage_buffer);

    assert!(module_result.is_err());

    let module = context.create_module("mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let f = module.add_function("f", &fn_type, None);
    let basic_block = f.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&basic_block);
    builder.build_return(None);

    assert!(module.verify().is_ok());

    let buffer = module.write_bitcode_to_memory();
    let module2_result = Module::parse_bitcode_from_buffer(&buffer);

    assert!(module2_result.is_ok());
    assert_eq!(module2_result.unwrap().get_context(), Context::get_global());

    let module3_result = Module::parse_bitcode_from_buffer_in_context(&garbage_buffer, &context);

    assert!(module3_result.is_err());

    let buffer2 = module.write_bitcode_to_memory();
    let module4_result = Module::parse_bitcode_from_buffer_in_context(&buffer2, &context);

    assert!(module4_result.is_ok());
    assert_eq!(*module4_result.unwrap().get_context(), context);
}

#[test]
fn test_parse_from_path() {
    let context = Context::create();
    let garbage_path = Path::new("foo/bar");
    let module_result = Module::parse_bitcode_from_path(&garbage_path);

    assert!(module_result.is_err(), "1");

    let module_result2 = Module::parse_bitcode_from_path_in_context(&garbage_path, &context);

    assert!(module_result2.is_err(), "2");

    let module = context.create_module("mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let f = module.add_function("f", &fn_type, None);
    let basic_block = f.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&basic_block);
    builder.build_return(None);

    assert!(module.verify().is_ok(), "3");

    // FIXME: Wasn't able to test success case. Got "invalid bitcode signature"
    let mut temp_path = temp_dir();

    temp_path.push("module.bc");

    module.write_bitcode_to_path(&temp_path);

    let module3_result = Module::parse_bitcode_from_path(&temp_path);

    assert!(module3_result.is_ok());
    assert_eq!(module3_result.unwrap().get_context(), Context::get_global());

    let module4_result = Module::parse_bitcode_from_path_in_context(&temp_path, &context);

    assert!(module4_result.is_ok());
    assert_eq!(*module4_result.unwrap().get_context(), context);
}

#[test]
fn test_clone() {
    let context = Context::create();
    let module = context.create_module("mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let f = module.add_function("f", &fn_type, None);
    let basic_block = f.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&basic_block);
    builder.build_return(None);

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
    let f = module.add_function("f", &fn_type, None);
    let basic_block = f.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&basic_block);
    builder.build_return(None);

    let bad_path = Path::new("/tmp/some/silly/path/that/sure/doesn't/exist");

    assert_eq!(*module.print_to_file(bad_path).unwrap_err(), *CString::new("No such file or directory").unwrap());

    let mut temp_path = temp_dir();

    temp_path.push("module");

    assert!(module.print_to_file(&temp_path).is_ok());
}

#[test]
fn test_get_set_target() {
    Target::initialize_x86(&Default::default());

    let context = Context::create();
    let module = context.create_module("mod");
    let target = Target::from_name("x86-64").unwrap();

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
    assert_eq!(*module.get_name(), *CString::new("mod").unwrap());
    assert!(module.get_target().is_none());

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
    module.set_name("mod2");
    module.set_target(&target);

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
    assert_eq!(*module.get_name(), *CString::new("mod2").unwrap());
    assert_eq!(module.get_target().unwrap(), target);
}
