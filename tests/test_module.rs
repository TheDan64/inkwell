extern crate inkwell;

use self::inkwell::context::Context;
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
