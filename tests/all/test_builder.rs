extern crate inkwell;

use self::inkwell::{AddressSpace, OptimizationLevel};
use self::inkwell::context::Context;
use self::inkwell::builder::Builder;
use self::inkwell::values::BasicValue;

// use std::ffi::CString;
use std::ptr::null;

#[test]
fn test_build_call() {
    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();

    let f32_type = context.f32_type();
    let fn_type = f32_type.fn_type(&[], false);

    let function = module.add_function("get_pi", fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let pi = f32_type.const_float(::std::f64::consts::PI);

    builder.build_return(Some(&pi));

    let function2 = module.add_function("wrapper", fn_type, None);
    let basic_block2 = context.append_basic_block(&function2, "entry");

    builder.position_at_end(&basic_block2);

    let pi2_call_site = builder.build_call(function, &[], "get_pi");

    assert!(!pi2_call_site.is_tail_call());

    pi2_call_site.set_tail_call(true);

    assert!(pi2_call_site.is_tail_call());

    let pi2 = pi2_call_site.try_as_basic_value().left().unwrap();

    builder.build_return(Some(&pi2));

    assert!(module.verify().is_ok());

    // Test using function `PointerValue`
    let void_type = context.void_type();
    let fn_type2 = void_type.fn_type(&[], false);
    let function3 = module.add_function("call_fn", fn_type2, None);
    let basic_block3 = context.append_basic_block(&function3, "entry");
    let fn_ptr = function3.as_global_value().as_pointer_value();
    let fn_ptr_type = fn_ptr.get_type();

    builder.position_at_end(&basic_block3);

    let alloca = builder.build_alloca(fn_ptr_type, "alloca");

    builder.build_store(alloca, fn_ptr);

    let load = builder.build_load(alloca, "load").into_pointer_value();

    builder.build_call(load, &[], "call");
    builder.build_return(None);

    assert!(module.verify().is_ok());
}

#[test]
fn test_null_checked_ptr_ops() {
    let context = Context::create();
    let module = context.create_module("unsafe");
    let builder = context.create_builder();

    // Here we're going to create a function that looks roughly like:
    // fn check_null_index1(ptr: *const i8) -> i8 {
    //     if ptr.is_null() {
    //         -1
    //     } else {
    //         ptr[1]
    //     }
    // }

    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(AddressSpace::Generic);
    let i64_type = context.i64_type();
    let fn_type = i8_type.fn_type(&[i8_ptr_type.into()], false);
    let neg_one = i8_type.const_all_ones();
    let one = i64_type.const_int(1, false);

    let function = module.add_function("check_null_index1", fn_type, None);
    let entry = context.append_basic_block(&function, "entry");

    builder.position_at_end(&entry);

    let ptr = function.get_first_param().unwrap().into_pointer_value();

    let is_null = builder.build_is_null(ptr, "is_null");

    let ret_0 = function.append_basic_block("ret_0");
    let ret_idx = function.append_basic_block("ret_idx");

    builder.build_conditional_branch(is_null, &ret_0, &ret_idx);

    builder.position_at_end(&ret_0);
    builder.build_return(Some(&neg_one));

    builder.position_at_end(&ret_idx);

    // FIXME: This might not work if compiled on non 64bit devices. Ideally we'd
    // be able to create pointer sized ints easily
    let ptr_as_int = builder.build_ptr_to_int(ptr, i64_type, "ptr_as_int");
    let new_ptr_as_int = builder.build_int_add(ptr_as_int, one, "add");
    let new_ptr = builder.build_int_to_ptr(new_ptr_as_int, i8_ptr_type, "int_as_ptr");
    let index1 = builder.build_load(new_ptr, "deref");

    builder.build_return(Some(&index1));

    // Here we're going to create a function that looks roughly like:
    // fn check_null_index2(ptr: *const i8) -> i8 {
    //     if !ptr.is_null() {
    //         ptr[1]
    //     } else {
    //         -1
    //     }
    // }

    let function = module.add_function("check_null_index2", fn_type, None);
    let entry = context.append_basic_block(&function, "entry");

    builder.position_at_end(&entry);

    let ptr = function.get_first_param().unwrap().into_pointer_value();

    let is_not_null = builder.build_is_not_null(ptr, "is_not_null");

    let ret_idx = function.append_basic_block("ret_idx");
    let ret_0 = function.append_basic_block("ret_0");

    builder.build_conditional_branch(is_not_null, &ret_idx, &ret_0);

    builder.position_at_end(&ret_0);
    builder.build_return(Some(&neg_one));

    builder.position_at_end(&ret_idx);

    // FIXME: This might not work if compiled on non 64bit devices. Ideally we'd
    // be able to create pointer sized ints easily
    let ptr_as_int = builder.build_ptr_to_int(ptr, i64_type, "ptr_as_int");
    let new_ptr_as_int = builder.build_int_add(ptr_as_int, one, "add");
    let new_ptr = builder.build_int_to_ptr(new_ptr_as_int, i8_ptr_type, "int_as_ptr");
    let index1 = builder.build_load(new_ptr, "deref");

    builder.build_return(Some(&index1));

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    unsafe {
        let check_null_index1 = execution_engine.get_function::<unsafe extern "C" fn(*const i8) -> i8>("check_null_index1").unwrap();

        let array = &[100i8, 42i8];

        assert_eq!(check_null_index1.call(null()), -1i8);
        assert_eq!(check_null_index1.call(array.as_ptr()), 42i8);

        let check_null_index2 = execution_engine.get_function::<unsafe extern "C" fn(*const i8) -> i8>("check_null_index2").unwrap();

        assert_eq!(check_null_index2.call(null()), -1i8);
        assert_eq!(check_null_index2.call(array.as_ptr()), 42i8);
    }
}

#[test]
fn test_binary_ops() {
    let context = Context::create();
    let module = context.create_module("unsafe");
    let builder = context.create_builder();
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    // Here we're going to create an and function which looks roughly like:
    // fn and(left: bool, right: bool) -> bool {
    //     left && right
    // }

    let bool_type = context.bool_type();
    let fn_type = bool_type.fn_type(&[bool_type.into(), bool_type.into()], false);
    let fn_value = module.add_function("and", fn_type, None);
    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let left = fn_value.get_first_param().unwrap().into_int_value();
    let right = fn_value.get_last_param().unwrap().into_int_value();

    let and = builder.build_and(left, right, "and_op");

    builder.build_return(Some(&and));

    // Here we're going to create an or function which looks roughly like:
    // fn or(left: bool, right: bool) -> bool {
    //     left || right
    // }

    let fn_value = module.add_function("or", fn_type, None);
    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let left = fn_value.get_first_param().unwrap().into_int_value();
    let right = fn_value.get_last_param().unwrap().into_int_value();

    let or = builder.build_or(left, right, "or_op");

    builder.build_return(Some(&or));

    // Here we're going to create a xor function which looks roughly like:
    // fn xor(left: bool, right: bool) -> bool {
    //     left || right
    // }

    let fn_value = module.add_function("xor", fn_type, None);
    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let left = fn_value.get_first_param().unwrap().into_int_value();
    let right = fn_value.get_last_param().unwrap().into_int_value();

    let xor = builder.build_xor(left, right, "xor_op");

    builder.build_return(Some(&xor));

    unsafe {
        type BoolFunc = unsafe extern "C" fn(bool, bool) -> bool;

        let and = execution_engine.get_function::<BoolFunc>("and").unwrap();
        let or = execution_engine.get_function::<BoolFunc>("or").unwrap();
        let xor = execution_engine.get_function::<BoolFunc>("xor").unwrap();

        assert!(!and.call(false, false));
        assert!(!and.call(true, false));
        assert!(!and.call(false, true));
        assert!(and.call(true, true));

        assert!(!or.call(false, false));
        assert!(or.call(true, false));
        assert!(or.call(false, true));
        assert!(or.call(true, true));

        assert!(!xor.call(false, false));
        assert!(xor.call(true, false));
        assert!(xor.call(false, true));
        assert!(!xor.call(true, true));
    }
}

#[test]
fn test_switch() {
    let context = Context::create();
    let module = context.create_module("unsafe");
    let builder = context.create_builder();
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

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
    let fn_type = i8_type.fn_type(&[i8_type.into()], false);
    let fn_value = module.add_function("switch", fn_type, None);
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
    builder.build_switch(value, &else_, &[(i8_zero, &check), (i8_42, &elif)]);

    builder.position_at_end(&check);
    builder.build_return(Some(&i8_one));

    builder.position_at_end(&elif);
    builder.build_return(Some(&i8_255));

    builder.position_at_end(&else_);

    let double = builder.build_int_mul(value, i8_two, "double");

    builder.build_return(Some(&double));

    unsafe {
        let switch = execution_engine.get_function::<unsafe extern "C" fn(u8) -> u8>("switch").unwrap();

        assert_eq!(switch.call(0), 1);
        assert_eq!(switch.call(1), 2);
        assert_eq!(switch.call(3), 6);
        assert_eq!(switch.call(10), 20);
        assert_eq!(switch.call(42), 255);
    }
}

#[test]
fn test_bit_shifts() {
    let context = Context::create();
    let module = context.create_module("unsafe");
    let builder = context.create_builder();
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    // Here we're going to create a function which looks roughly like:
    // fn left_shift(value: u8, bits: u8) -> u8 {
    //     value << bits
    // }
    let i8_type = context.i8_type();
    let fn_type = i8_type.fn_type(&[i8_type.into(), i8_type.into()], false);
    let fn_value = module.add_function("left_shift", fn_type, None);
    let value = fn_value.get_first_param().unwrap().into_int_value();
    let bits = fn_value.get_nth_param(1).unwrap().into_int_value();

    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let shift = builder.build_left_shift(value, bits, "shl");

    builder.build_return(Some(&shift));

    // Here we're going to create a function which looks roughly like:
    // fn right_shift(value: u8, bits: u8) -> u8 {
    //     value >> bits
    // }
    let fn_value = module.add_function("right_shift", fn_type, None);
    let value = fn_value.get_first_param().unwrap().into_int_value();
    let bits = fn_value.get_nth_param(1).unwrap().into_int_value();

    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let shift = builder.build_right_shift(value, bits, false, "shr");

    builder.build_return(Some(&shift));

    // Here we're going to create a function which looks roughly like:
    // fn right_shift(value: u8, bits: u8) -> u8 {
    //     value >> bits
    // }
    let fn_value = module.add_function("right_shift_sign_extend", fn_type, None);
    let value = fn_value.get_first_param().unwrap().into_int_value();
    let bits = fn_value.get_nth_param(1).unwrap().into_int_value();

    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let shift = builder.build_right_shift(value, bits, true, "shr");

    builder.build_return(Some(&shift));

    unsafe {
        let left_shift = execution_engine.get_function::<unsafe extern "C" fn(u8, u8) -> u8>("left_shift").unwrap();
        let right_shift  = execution_engine.get_function::<unsafe extern "C" fn(u8, u8) -> u8>("right_shift").unwrap();
        let right_shift_sign_extend = execution_engine.get_function::<unsafe extern "C" fn(i8, u8) -> i8>("right_shift_sign_extend").unwrap();

        assert_eq!(left_shift.call(0, 0), 0);
        assert_eq!(left_shift.call(0, 4), 0);
        assert_eq!(left_shift.call(1, 0), 1);
        assert_eq!(left_shift.call(1, 1), 2);
        assert_eq!(left_shift.call(1, 2), 4);
        assert_eq!(left_shift.call(1, 3), 8);
        assert_eq!(left_shift.call(64, 1), 128);

        assert_eq!(right_shift.call(128, 1), 64);
        assert_eq!(right_shift.call(8, 3), 1);
        assert_eq!(right_shift.call(4, 2), 1);
        assert_eq!(right_shift.call(2, 1), 1);
        assert_eq!(right_shift.call(1, 0), 1);
        assert_eq!(right_shift.call(0, 4), 0);
        assert_eq!(right_shift.call(0, 0), 0);

        assert_eq!(right_shift_sign_extend.call(8, 3), 1);
        assert_eq!(right_shift_sign_extend.call(4, 2), 1);
        assert_eq!(right_shift_sign_extend.call(2, 1), 1);
        assert_eq!(right_shift_sign_extend.call(1, 0), 1);
        assert_eq!(right_shift_sign_extend.call(0, 4), 0);
        assert_eq!(right_shift_sign_extend.call(0, 0), 0);
        assert_eq!(right_shift_sign_extend.call(-127, 1), -64);
        assert_eq!(right_shift_sign_extend.call(-127, 8), -1);
        assert_eq!(right_shift_sign_extend.call(-65, 3), -9);
        assert_eq!(right_shift_sign_extend.call(-64, 3), -8);
        assert_eq!(right_shift_sign_extend.call(-63, 3), -8);
    }
}

#[test]
fn test_global_builder() {
    // Unfortunately LLVM doesn't provide us with a get_context method like it does for
    // modules and types, so we can't assert it actualy is of the same global context...
    Builder::create();
}

#[test]
fn test_unconditional_branch() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("my_fn", fn_type, None);
    let entry_bb = fn_value.append_basic_block("entry");
    let skipped_bb = fn_value.append_basic_block("skipped");
    let end_bb = fn_value.append_basic_block("end");

    builder.position_at_end(&entry_bb);
    builder.build_unconditional_branch(&end_bb);

    builder.position_at_end(&skipped_bb);
    builder.build_unreachable();
}

#[test]
fn test_no_builder_double_free() {
    let context = Context::create();
    let builder = context.create_builder();

    drop(builder);
    drop(context);
}

#[test]
fn test_no_builder_double_free2() {
    let builder = {
        let context = Context::create();

        context.create_builder()

        // Original Context drops fine
    };

    // Builder continues to function with different context
    let context = Context::create();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("my_fn", fn_type, None);
    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);
    // FIXME: Builder segfaults when making build calls with different context
    // as of newer rust versions(late 2018+?). Maybe this isn't actually something
    // you're suppose to do in LLVM and LTO(?) has made it a more prominent issue?
    // builder.build_unreachable();

    // #[cfg(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8"))]
    // assert_eq!(*module.print_to_string(), *CString::new("; ModuleID = \'my_mod\'\n\ndefine void @my_fn() {\nentry:\n  unreachable\n}\n").unwrap());
    // #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
    // assert_eq!(*module.print_to_string(), *CString::new("; ModuleID = \'my_mod\'\nsource_filename = \"my_mod\"\n\ndefine void @my_fn() {\nentry:\n  unreachable\n}\n").unwrap());

    // 2nd Context drops fine
    // Builder drops fine
}

#[test]
fn test_vector_convert_ops() {
    let context = Context::create();
    let module = context.create_module("test");
    let int8_vec_type = context.i8_type().vec_type(3);
    let int32_vec_type = context.i32_type().vec_type(3);
    let float32_vec_type = context.f32_type().vec_type(3);
    let float16_vec_type = context.f16_type().vec_type(3);

    // Here we're building a function that takes in a <3 x i8> and returns it casted to and from a <3 x i32>
    // Casting to and from means we can ensure the cast build functions return a vector when one is provided.
    let fn_type = int32_vec_type.fn_type(&[int8_vec_type.into()], false);
    let fn_value = module.add_function("test_int_vec_cast", fn_type, None);
    let entry = fn_value.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&entry);
    let in_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let casted_vec = builder.build_int_cast(in_vec, int32_vec_type, "casted_vec");
    let _uncasted_vec = builder.build_int_cast(casted_vec, int8_vec_type, "uncasted_vec");
    builder.build_return(Some(&casted_vec));
    assert!(fn_value.verify(true));

    // Here we're building a function that takes in a <3 x f32> and returns it casted to and from a <3 x f16>
    let fn_type = float16_vec_type.fn_type(&[float32_vec_type.into()], false);
    let fn_value = module.add_function("test_float_vec_cast", fn_type, None);
    let entry = fn_value.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&entry);
    let in_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let casted_vec = builder.build_float_cast(in_vec, float16_vec_type, "casted_vec");
    let _uncasted_vec = builder.build_float_cast(casted_vec, float32_vec_type, "uncasted_vec");
    builder.build_return(Some(&casted_vec));
    assert!(fn_value.verify(true));

    // Here we're building a function that takes in a <3 x f32> and returns it casted to and from a <3 x i32>
    let fn_type = int32_vec_type.fn_type(&[float32_vec_type.into()], false);
    let fn_value = module.add_function("test_float_to_int_vec_cast", fn_type, None);
    let entry = fn_value.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&entry);
    let in_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let casted_vec = builder.build_float_to_signed_int(in_vec, int32_vec_type, "casted_vec");
    let _uncasted_vec = builder.build_signed_int_to_float(casted_vec, float32_vec_type, "uncasted_vec");
    builder.build_return(Some(&casted_vec));
    assert!(fn_value.verify(true));
}

#[test]
fn test_vector_binary_ops() {
    let context = Context::create();
    let module = context.create_module("test");
    let int32_vec_type = context.i32_type().vec_type(2);
    let float32_vec_type = context.f32_type().vec_type(2);
    let bool_vec_type = context.bool_type().vec_type(2);

    // Here we're building a function that takes in three <2 x i32>s and returns them added together as a <2 x i32>
    let fn_type = int32_vec_type.fn_type(&[int32_vec_type.into(), int32_vec_type.into(), int32_vec_type.into()], false);
    let fn_value = module.add_function("test_int_vec_add", fn_type, None);
    let entry = fn_value.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&entry);
    let p1_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let p2_vec = fn_value.get_nth_param(1).unwrap().into_vector_value();
    let p3_vec = fn_value.get_nth_param(2).unwrap().into_vector_value();
    let added_vec = builder.build_int_add(p1_vec, p2_vec, "added_vec");
    let added_vec = builder.build_int_add(added_vec, p3_vec, "added_vec");
    builder.build_return(Some(&added_vec));
    assert!(fn_value.verify(true));

    // Here we're building a function that takes in three <2 x f32>s and returns x * y / z as an
    // <2 x f32>
    let fn_type = float32_vec_type.fn_type(&[float32_vec_type.into(), float32_vec_type.into(), float32_vec_type.into()], false);
    let fn_value = module.add_function("test_float_vec_mul", fn_type, None);
    let entry = fn_value.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&entry);
    let p1_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let p2_vec = fn_value.get_nth_param(1).unwrap().into_vector_value();
    let p3_vec = fn_value.get_nth_param(2).unwrap().into_vector_value();
    let multiplied_vec = builder.build_float_mul(p1_vec, p2_vec, "multipled_vec");
    let divided_vec = builder.build_float_div(multiplied_vec, p3_vec, "divided_vec");
    builder.build_return(Some(&divided_vec));
    assert!(fn_value.verify(true));

    // Here we're building a function that takes two <2 x f32>s and a <2 x bool> and returns (x < y) * z
    // as a <2 x bool>
    let fn_type = bool_vec_type.fn_type(&[float32_vec_type.into(), float32_vec_type.into(), bool_vec_type.into()], false);
    let fn_value = module.add_function("test_float_vec_compare", fn_type, None);
    let entry = fn_value.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&entry);
    let p1_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let p2_vec = fn_value.get_nth_param(1).unwrap().into_vector_value();
    let p3_vec = fn_value.get_nth_param(2).unwrap().into_vector_value();
    let compared_vec = builder.build_float_compare(self::inkwell::FloatPredicate::OLT, p1_vec, p2_vec, "compared_vec");
    let multiplied_vec = builder.build_int_mul(compared_vec, p3_vec, "multiplied_vec");
    builder.build_return(Some(&multiplied_vec));
    assert!(fn_value.verify(true));
}

#[test]
fn test_vector_pointer_ops() {
    let context = Context::create();
    let module = context.create_module("test");
    let int32_vec_type = context.i32_type().vec_type(4);
    let i8_ptr_vec_type = context.i8_type().ptr_type(AddressSpace::Generic).vec_type(4);
    let bool_vec_type = context.bool_type().vec_type(4);

    // Here we're building a function that takes a <4 x i32>, converts it to a <4 x i8*> and returns a
    // <4 x bool> if the pointer is null
    let fn_type = bool_vec_type.fn_type(&[int32_vec_type.into()], false);
    let fn_value = module.add_function("test_ptr_null", fn_type, None);
    let entry = fn_value.append_basic_block("entry");
    let builder = context.create_builder();

    builder.position_at_end(&entry);
    let in_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let ptr_vec = builder.build_int_to_ptr(in_vec, i8_ptr_vec_type, "ptr_vec");
    let is_null_vec = builder.build_is_null(ptr_vec, "is_null_vec");
    builder.build_return(Some(&is_null_vec));
    assert!(fn_value.verify(true));
}

#[test]
fn test_insert_value() {
    let context = Context::create();
    let module = context.create_module("av");
    let void_type = context.void_type();
    let f32_type = context.f32_type();
    let i32_type = context.i32_type();
    let struct_type = context.struct_type(&[i32_type.into(), f32_type.into()], false);
    let array_type = i32_type.array_type(3);
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("av_fn", fn_type, None);
    let builder = context.create_builder();
    let entry = fn_value.append_basic_block("entry");

    builder.position_at_end(&entry);

    let array_alloca = builder.build_alloca(array_type, "array_alloca");
    let array = builder.build_load(array_alloca, "array_load").into_array_value();
    let const_int1 = i32_type.const_int(2, false);
    let const_int2 = i32_type.const_int(5, false);
    let const_int3 = i32_type.const_int(6, false);
    let const_float = f32_type.const_float(3.14);

    assert!(builder.build_insert_value(array, const_int1, 0, "insert").is_some());
    assert!(builder.build_insert_value(array, const_int2, 1, "insert").is_some());
    assert!(builder.build_insert_value(array, const_int3, 2, "insert").is_some());
    assert!(builder.build_insert_value(array, const_int3, 3, "insert").is_none());
    assert!(builder.build_insert_value(array, const_int3, 4, "insert").is_none());

    assert!(builder.build_extract_value(array, 0, "extract").unwrap().is_int_value());
    assert!(builder.build_extract_value(array, 1, "extract").unwrap().is_int_value());
    assert!(builder.build_extract_value(array, 2, "extract").unwrap().is_int_value());
    assert!(builder.build_extract_value(array, 3, "extract").is_none());

    let struct_alloca = builder.build_alloca(struct_type, "struct_alloca");
    let struct_value = builder.build_load(struct_alloca, "struct_load").into_struct_value();

    assert!(builder.build_insert_value(struct_value, const_int2, 0, "insert").is_some());
    assert!(builder.build_insert_value(struct_value, const_float, 1, "insert").is_some());
    assert!(builder.build_insert_value(struct_value, const_float, 2, "insert").is_none());
    assert!(builder.build_insert_value(struct_value, const_float, 3, "insert").is_none());

    assert!(builder.build_extract_value(struct_value, 0, "extract").unwrap().is_int_value());
    assert!(builder.build_extract_value(struct_value, 1, "extract").unwrap().is_float_value());
    assert!(builder.build_extract_value(struct_value, 2, "extract").is_none());
    assert!(builder.build_extract_value(struct_value, 3, "extract").is_none());

    builder.build_return(None);

    assert!(module.verify().is_ok());
}

#[test]
fn test_bitcast() {
    let context = Context::create();
    let module = context.create_module("bc");
    let void_type = context.void_type();
    let f32_type = context.f32_type();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let i32_ptr_type = i32_type.ptr_type(AddressSpace::Generic);
    let i64_ptr_type = i64_type.ptr_type(AddressSpace::Generic);
    let i32_vec_type = i32_type.vec_type(2);
    let arg_types = [
        i32_type.into(),
        f32_type.into(),
        i32_vec_type.into(),
        i32_ptr_type.into(),
    ];
    let fn_type = void_type.fn_type(&arg_types, false);
    let fn_value = module.add_function("bc", fn_type, None);
    let builder = context.create_builder();
    let entry = fn_value.append_basic_block("entry");
    let i32_arg = fn_value.get_first_param().unwrap().into_int_value();
    let f32_arg = fn_value.get_nth_param(1).unwrap().into_float_value();
    let i32_vec_arg = fn_value.get_nth_param(2).unwrap().into_vector_value();
    let i32_ptr_arg = fn_value.get_nth_param(3).unwrap().into_pointer_value();

    builder.position_at_end(&entry);

    let cast = builder.build_bitcast(i32_arg, f32_type, "i32tof32");

    builder.build_bitcast(f32_arg, f32_type, "f32tof32");
    builder.build_bitcast(i32_vec_arg, i64_type, "2xi32toi64");
    builder.build_bitcast(i32_ptr_arg, i64_ptr_type, "i32*toi64*");

    builder.build_return(None);

    assert!(module.verify().is_ok(), module.print_to_string().to_string());

    let first_iv = cast.as_instruction_value().unwrap();

    builder.position_before(&first_iv);
    builder.build_bitcast(f32_arg, i64_type, "f32toi64");

    assert!(module.verify().is_err());
}
