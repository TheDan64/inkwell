extern crate inkwell;

use self::inkwell::OptimizationLevel;
use self::inkwell::context::Context;
use self::inkwell::builder::Builder;
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
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let module = execution_engine.get_module_at(0);

    // Here we're going to create a function that looks roughly like:
    // fn check_null_index1(ptr: *const i8) -> i8 {
    //     if ptr.is_null() {
    //         -1
    //     } else {
    //         ptr[1]
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

#[test]
fn test_binary_ops() {
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let context = Context::create();
    let module = context.create_module("unsafe");
    let builder = context.create_builder();
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let module = execution_engine.get_module_at(0);

    // Here we're going to create an and function which looks roughly like:
    // fn and(left: bool, right: bool) -> bool {
    //     left && right
    // }

    let bool_type = context.bool_type();
    let fn_type = bool_type.fn_type(&[&bool_type, &bool_type], false);
    let fn_value = module.add_function("and", &fn_type, None);
    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let left = fn_value.get_first_param().unwrap().into_int_value();
    let right = fn_value.get_last_param().unwrap().into_int_value();

    let and = builder.build_and(&left, &right, "and_op");

    builder.build_return(Some(&and));

    // Here we're going to create an or function which looks roughly like:
    // fn or(left: bool, right: bool) -> bool {
    //     left || right
    // }

    let fn_value = module.add_function("or", &fn_type, None);
    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let left = fn_value.get_first_param().unwrap().into_int_value();
    let right = fn_value.get_last_param().unwrap().into_int_value();

    let or = builder.build_or(&left, &right, "or_op");

    builder.build_return(Some(&or));

    // Here we're going to create a xor function which looks roughly like:
    // fn xor(left: bool, right: bool) -> bool {
    //     left || right
    // }

    let fn_value = module.add_function("xor", &fn_type, None);
    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let left = fn_value.get_first_param().unwrap().into_int_value();
    let right = fn_value.get_last_param().unwrap().into_int_value();

    let xor = builder.build_xor(&left, &right, "xor_op");

    builder.build_return(Some(&xor));

    let addr = execution_engine.get_function_address("and").unwrap();
    let and: extern "C" fn(bool, bool) -> bool = unsafe { transmute(addr) };
    let addr = execution_engine.get_function_address("or").unwrap();
    let or: extern "C" fn(bool, bool) -> bool = unsafe { transmute(addr) };
    let addr = execution_engine.get_function_address("xor").unwrap();
    let xor: extern "C" fn(bool, bool) -> bool = unsafe { transmute(addr) };

    assert!(!and(false, false));
    assert!(!and(true, false));
    assert!(!and(false, true));
    assert!(and(true, true));

    assert!(!or(false, false));
    assert!(or(true, false));
    assert!(or(false, true));
    assert!(or(true, true));

    assert!(!xor(false, false));
    assert!(xor(true, false));
    assert!(xor(false, true));
    assert!(!xor(true, true));
}

#[test]
fn test_switch() {
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let context = Context::create();
    let module = context.create_module("unsafe");
    let builder = context.create_builder();
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let module = execution_engine.get_module_at(0);

    // Here we're going to create a function which looks roughly like:
    // fn switch(val: u8) -> u8 {
    //     if val == 0 {
    //         1
    //     } else if val == 42 {
    //         255
    //     } else {
    //         val * 2
    //     }
    // }

    let i8_type = context.i8_type();
    let fn_type = i8_type.fn_type(&[&i8_type], false);
    let fn_value = module.add_function("switch", &fn_type, None);
    let i8_zero = i8_type.const_int(0, false);
    let i8_one = i8_type.const_int(1, false);
    let i8_two = i8_type.const_int(2, false);
    let i8_42 = i8_type.const_int(42, false);
    let i8_255 = i8_type.const_int(255, false);
    let entry = fn_value.append_basic_block("entry");
    let check = fn_value.append_basic_block("check");
    let elif = fn_value.append_basic_block("elif");
    let else_ = fn_value.append_basic_block("else");
    let value = fn_value.get_first_param().unwrap().into_int_value();

    builder.position_at_end(&entry);
    builder.build_switch(&value, &else_, &[(&i8_zero, &check), (&i8_42, &elif)]);

    builder.position_at_end(&check);
    builder.build_return(Some(&i8_one));

    builder.position_at_end(&elif);
    builder.build_return(Some(&i8_255));

    builder.position_at_end(&else_);

    let double = builder.build_int_mul(&value, &i8_two, "double");

    builder.build_return(Some(&double));

    let addr = execution_engine.get_function_address("switch").unwrap();
    let switch: extern "C" fn(u8) -> u8 = unsafe { transmute(addr) };

    assert_eq!(switch(0), 1);
    assert_eq!(switch(1), 2);
    assert_eq!(switch(3), 6);
    assert_eq!(switch(10), 20);
    assert_eq!(switch(42), 255);
}

#[test]
fn test_bit_shifts() {
    Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");

    let context = Context::create();
    let module = context.create_module("unsafe");
    let builder = context.create_builder();
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let module = execution_engine.get_module_at(0);

    // Here we're going to create a function which looks roughly like:
    // fn left_shift(value: u8, bits: u8) -> u8 {
    //     value << bits
    // }
    let i8_type = context.i8_type();
    let fn_type = i8_type.fn_type(&[&i8_type, &i8_type], false);
    let fn_value = module.add_function("left_shift", &fn_type, None);
    let value = fn_value.get_first_param().unwrap().into_int_value();
    let bits = fn_value.get_nth_param(1).unwrap().into_int_value();

    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let shift = builder.build_left_shift(&value, &bits, "shl");

    builder.build_return(Some(&shift));

    // Here we're going to create a function which looks roughly like:
    // fn right_shift(value: u8, bits: u8) -> u8 {
    //     value >> bits
    // }
    let fn_value = module.add_function("right_shift", &fn_type, None);
    let value = fn_value.get_first_param().unwrap().into_int_value();
    let bits = fn_value.get_nth_param(1).unwrap().into_int_value();

    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let shift = builder.build_right_shift(&value, &bits, false, "shr");

    builder.build_return(Some(&shift));

    // Here we're going to create a function which looks roughly like:
    // fn right_shift(value: u8, bits: u8) -> u8 {
    //     value >> bits
    // }
    let fn_value = module.add_function("right_shift_sign_extend", &fn_type, None);
    let value = fn_value.get_first_param().unwrap().into_int_value();
    let bits = fn_value.get_nth_param(1).unwrap().into_int_value();

    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let shift = builder.build_right_shift(&value, &bits, true, "shr");

    builder.build_return(Some(&shift));

    let addr = execution_engine.get_function_address("left_shift").unwrap();
    let left_shift: extern "C" fn(u8, u8) -> u8 = unsafe { transmute(addr) };
    let addr = execution_engine.get_function_address("right_shift").unwrap();
    let right_shift: extern "C" fn(u8, u8) -> u8 = unsafe { transmute(addr) };
    let addr = execution_engine.get_function_address("right_shift_sign_extend").unwrap();
    let right_shift_sign_extend: extern "C" fn(i8, u8) -> i8 = unsafe { transmute(addr) };

    assert_eq!(left_shift(0, 0), 0);
    assert_eq!(left_shift(0, 4), 0);
    assert_eq!(left_shift(1, 0), 1);
    assert_eq!(left_shift(1, 1), 2);
    assert_eq!(left_shift(1, 2), 4);
    assert_eq!(left_shift(1, 3), 8);
    assert_eq!(left_shift(64, 1), 128);

    assert_eq!(right_shift(128, 1), 64);
    assert_eq!(right_shift(8, 3), 1);
    assert_eq!(right_shift(4, 2), 1);
    assert_eq!(right_shift(2, 1), 1);
    assert_eq!(right_shift(1, 0), 1);
    assert_eq!(right_shift(0, 4), 0);
    assert_eq!(right_shift(0, 0), 0);

    assert_eq!(right_shift_sign_extend(8, 3), 1);
    assert_eq!(right_shift_sign_extend(4, 2), 1);
    assert_eq!(right_shift_sign_extend(2, 1), 1);
    assert_eq!(right_shift_sign_extend(1, 0), 1);
    assert_eq!(right_shift_sign_extend(0, 4), 0);
    assert_eq!(right_shift_sign_extend(0, 0), 0);
    assert_eq!(right_shift_sign_extend(-127, 1), -64);
    assert_eq!(right_shift_sign_extend(-127, 8), -1);
    assert_eq!(right_shift_sign_extend(-65, 3), -9);
    assert_eq!(right_shift_sign_extend(-64, 3), -8);
    assert_eq!(right_shift_sign_extend(-63, 3), -8);
}

#[test]
fn test_global_builder() {
    // Unfortunately LLVM doesn't provide us with a get_context method like it does for
    // modules and types, so we can't assert it actualy is of the same global context...
    Builder::create();
}
