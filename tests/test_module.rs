extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::memory_buffer::MemoryBuffer;
use self::inkwell::module::Module;
use std::env::temp_dir;
use std::fs::{File, remove_file};
use std::io::Read;

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
fn test_owned_data_layout_disposed_safely() {
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

    // REVIEW: This returns a null ptr :(
    // let object_file = memory_buffer.create_object_file();

    let module2 = context.create_module_from_ir(memory_buffer).unwrap();

    assert_eq!(module2.get_function("my_fn").unwrap().print_to_string(), function.print_to_string());
}

#[test]
fn test_garbage_ir_fails_create_module_from_ir() {
    let context = Context::create();
    let memory_buffer = MemoryBuffer::create_from_memory_range("garbage ir data", "my_ir");

    // REVIEW: Why isn't this "garbage ir data"?
    assert_eq!(memory_buffer.as_slice().to_str().unwrap(), "");
    assert!(memory_buffer.get_size() > 0);
    assert!(context.create_module_from_ir(memory_buffer).is_err());
}

#[test]
fn test_get_type() {
    let context = Context::get_global();
    let module = Module::create("my_module");

    assert_eq!(module.get_context(), context);
    assert!(module.get_type("foo").is_none());

    let opaque = context.opaque_struct_type("foo");

    assert_eq!(module.get_type("foo").unwrap().into_struct_type(), opaque);
}
