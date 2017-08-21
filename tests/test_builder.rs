extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::targets::{InitializationConfig, Target};

use std::ptr::null;
use std::mem::transmute;

#[test]
fn test_build_call() {
    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();

    let f32_type = context.f32_type();
    let fn_type = f32_type.fn_type(&[], false);

    let function = module.add_function("get_pi", &fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let pi = f32_type.const_float(::std::f64::consts::PI);

    builder.build_return(Some(&pi));

    let function2 = module.add_function("wrapper", &fn_type, None);
    let basic_block2 = context.append_basic_block(&function2, "entry");

    builder.position_at_end(&basic_block2);

    let pi2 = builder.build_call(&function, &[], "get_pi", false).left().unwrap();

    builder.build_return(Some(&pi2));
}

#[test]
fn test_null_checked_ptr_ops() {
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let context = Context::create();
    let module = context.create_module("unsafe");
    let builder = context.create_builder();
    let execution_engine = module.create_jit_execution_engine(0).unwrap();
    let module = execution_engine.get_module_at(0);

    // Here we're going to create a function that looks roughly like:
    // fn check_null_index1(ptr: *const i8) -> i8 {
    //     if ptr.is_null() {
    //         return -1;
    //     } else {
    //         return ptr[1];
    //     }
    // }

    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(0);
    let i64_type = context.i64_type();
    let fn_type = i8_type.fn_type(&[&i8_ptr_type], false);
    let neg_one = i8_type.const_all_ones();
    let one = i64_type.const_int(1, false);

    let function = module.add_function("check_null_index1", &fn_type, None);
    let entry = context.append_basic_block(&function, "entry");

    builder.position_at_end(&entry);

    let ptr = function.get_first_param().unwrap().into_pointer_value();

    let is_null = builder.build_is_null(&ptr, "is_null");

    let ret_0 = function.append_basic_block("ret_0");
    let ret_idx = function.append_basic_block("ret_idx");

    builder.build_conditional_branch(&is_null, &ret_0, &ret_idx);

    builder.position_at_end(&ret_0);
    builder.build_return(Some(&neg_one));

    builder.position_at_end(&ret_idx);

    // FIXME: This might not work if compiled on non 64bit devices. Ideally we'd
    // be able to create pointer sized ints easily
    let ptr_as_int = builder.build_ptr_to_int(&ptr, &i64_type, "ptr_as_int");
    let new_ptr_as_int = builder.build_int_add(&ptr_as_int, &one, "add");
    let new_ptr = builder.build_int_to_ptr(&new_ptr_as_int, &i8_ptr_type, "int_as_ptr");
    let index1 = builder.build_load(&new_ptr, "deref");

    builder.build_return(Some(&index1));

    let addr = execution_engine.get_function_address("check_null_index1").unwrap();

    let check_null_index1: extern "C" fn(*const i8) -> i8 = unsafe { transmute(addr) };

    assert_eq!(check_null_index1(null()), -1i8);

    let array = &[100i8, 42i8];

    assert_eq!(check_null_index1(array.as_ptr()), 42i8);
}
