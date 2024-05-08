use inkwell::builder::BuilderError;
use inkwell::context::Context;
use inkwell::{AddressSpace, AtomicOrdering, AtomicRMWBinOp, OptimizationLevel};

use std::ptr::null;

#[test]
fn test_build_call() {
    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();

    let f32_type = context.f32_type();
    let fn_type = f32_type.fn_type(&[], false);

    let function = module.add_function("get_pi", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let pi = f32_type.const_float(std::f64::consts::PI);

    builder.build_return(Some(&pi)).unwrap();

    let function2 = module.add_function("wrapper", fn_type, None);
    let basic_block2 = context.append_basic_block(function2, "entry");

    builder.position_at_end(basic_block2);

    let pi2_call_site = builder.build_call(function, &[], "get_pi").unwrap();

    assert!(!pi2_call_site.is_tail_call());

    pi2_call_site.set_tail_call(true);

    assert!(pi2_call_site.is_tail_call());

    let pi2 = pi2_call_site.try_as_basic_value().left().unwrap();

    builder.build_return(Some(&pi2)).unwrap();

    assert!(module.verify().is_ok());

    // Test using function `PointerValue`
    let void_type = context.void_type();
    let fn_type2 = void_type.fn_type(&[], false);
    let function3 = module.add_function("call_fn", fn_type2, None);
    let basic_block3 = context.append_basic_block(function3, "entry");
    let fn_ptr = function3.as_global_value().as_pointer_value();
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let fn_ptr_type = fn_ptr.get_type();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let fn_ptr_type = context.ptr_type(AddressSpace::default());

    builder.position_at_end(basic_block3);

    let alloca = builder.build_alloca(fn_ptr_type, "alloca").unwrap();

    builder.build_store(alloca, fn_ptr).unwrap();

    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    let load = builder.build_load(alloca, "load").unwrap().into_pointer_value();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let load = builder
        .build_load(fn_ptr_type, alloca, "load")
        .unwrap()
        .into_pointer_value();

    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    {
        use inkwell::values::CallableValue;
        let callable_value = CallableValue::try_from(load).unwrap();
        builder.build_call(callable_value, &[], "call").unwrap();
    }
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    builder.build_indirect_call(fn_type2, load, &[], "call").unwrap();
    builder.build_return(None).unwrap();

    assert!(module.verify().is_ok());
}

#[test]
fn test_build_invoke_cleanup_resume() {
    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();

    let f32_type = context.f32_type();
    let fn_type = f32_type.fn_type(&[], false);

    // we will pretend this function can throw an exception
    let function = module.add_function("bomb", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let pi = f32_type.const_float(std::f64::consts::PI);

    builder.build_return(Some(&pi)).unwrap();

    let function2 = module.add_function("wrapper", fn_type, None);
    let basic_block2 = context.append_basic_block(function2, "entry");

    builder.position_at_end(basic_block2);

    let then_block = context.append_basic_block(function2, "then_block");
    let catch_block = context.append_basic_block(function2, "catch_block");

    let call_site = builder
        .build_invoke(function, &[], then_block, catch_block, "get_pi")
        .unwrap();

    assert!(!call_site.is_tail_call());

    call_site.set_tail_call(true);

    assert!(call_site.is_tail_call());

    {
        builder.position_at_end(then_block);

        let result = call_site.try_as_basic_value().left().unwrap();

        builder.build_return(Some(&result)).unwrap();
    }

    {
        builder.position_at_end(catch_block);

        // the personality function used by C++
        let personality_function = {
            let name = "__gxx_personality_v0";

            module.add_function(name, context.i64_type().fn_type(&[], false), None)
        };

        // type of an exception in C++
        #[cfg(not(any(
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        )))]
        let i8_ptr_type = context.i32_type().ptr_type(AddressSpace::default());
        #[cfg(any(
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        let i8_ptr_type = context.ptr_type(AddressSpace::default());
        let i32_type = context.i32_type();
        let exception_type = context.struct_type(&[i8_ptr_type.into(), i32_type.into()], false);

        let res = builder
            .build_landing_pad(exception_type, personality_function, &[], true, "res")
            .unwrap();

        // do cleanup ...

        builder.build_resume(res).unwrap();
    }

    assert!(module.verify().is_ok());
}

#[test]
fn test_build_invoke_catch_all() {
    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();

    let f32_type = context.f32_type();
    let fn_type = f32_type.fn_type(&[], false);

    let function = module.add_function("get_pi", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let pi = f32_type.const_float(std::f64::consts::PI);

    builder.build_return(Some(&pi)).unwrap();

    let function2 = module.add_function("wrapper", fn_type, None);
    let basic_block2 = context.append_basic_block(function2, "entry");

    builder.position_at_end(basic_block2);

    let then_block = context.append_basic_block(function2, "then_block");
    let catch_block = context.append_basic_block(function2, "catch_block");

    let pi2_call_site = builder
        .build_invoke(function, &[], then_block, catch_block, "get_pi")
        .unwrap();

    assert!(!pi2_call_site.is_tail_call());

    pi2_call_site.set_tail_call(true);

    assert!(pi2_call_site.is_tail_call());

    {
        builder.position_at_end(then_block);

        let pi2 = pi2_call_site.try_as_basic_value().left().unwrap();

        builder.build_return(Some(&pi2)).unwrap();
    }

    {
        builder.position_at_end(catch_block);

        // the personality function used by C++
        let personality_function = {
            let name = "__gxx_personality_v0";

            module.add_function(name, context.i64_type().fn_type(&[], false), None)
        };

        // type of an exception in C++
        #[cfg(not(any(
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        )))]
        let i8_ptr_type = context.i32_type().ptr_type(AddressSpace::default());
        #[cfg(any(
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        let i8_ptr_type = context.ptr_type(AddressSpace::default());
        let i32_type = context.i32_type();
        let exception_type = context.struct_type(&[i8_ptr_type.into(), i32_type.into()], false);

        let null = i8_ptr_type.const_zero();
        builder
            .build_landing_pad(exception_type, personality_function, &[null.into()], false, "res")
            .unwrap();

        let fakepi = f32_type.const_zero();

        builder.build_return(Some(&fakepi)).unwrap();
    }

    assert!(module.verify().is_ok());
}

#[test]
fn landing_pad_filter() {
    use inkwell::module::Linkage;
    use inkwell::values::AnyValue;

    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();

    let f32_type = context.f32_type();
    let fn_type = f32_type.fn_type(&[], false);

    let function = module.add_function("get_pi", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let pi = f32_type.const_float(std::f64::consts::PI);

    builder.build_return(Some(&pi)).unwrap();

    let function2 = module.add_function("wrapper", fn_type, None);
    let basic_block2 = context.append_basic_block(function2, "entry");

    builder.position_at_end(basic_block2);

    let then_block = context.append_basic_block(function2, "then_block");
    let catch_block = context.append_basic_block(function2, "catch_block");

    let pi2_call_site = builder
        .build_invoke(function, &[], then_block, catch_block, "get_pi")
        .unwrap();

    assert!(!pi2_call_site.is_tail_call());

    pi2_call_site.set_tail_call(true);

    assert!(pi2_call_site.is_tail_call());

    {
        builder.position_at_end(then_block);

        let pi2 = pi2_call_site.try_as_basic_value().left().unwrap();

        builder.build_return(Some(&pi2)).unwrap();
    }

    {
        builder.position_at_end(catch_block);

        // the personality function used by C++
        let personality_function = {
            let name = "__gxx_personality_v0";

            module.add_function(name, context.i64_type().fn_type(&[], false), None)
        };

        // type of an exception in C++
        #[cfg(not(any(
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        )))]
        let i8_ptr_type = context.i32_type().ptr_type(AddressSpace::default());
        #[cfg(any(
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        let i8_ptr_type = context.ptr_type(AddressSpace::default());
        let i32_type = context.i32_type();
        let exception_type = context.struct_type(&[i8_ptr_type.into(), i32_type.into()], false);

        // link in the C++ type info for the i32 type
        let type_info_int = module.add_global(i8_ptr_type, Some(AddressSpace::default()), "_ZTIi");
        type_info_int.set_linkage(Linkage::External);

        // make the filter landing pad
        let filter_pattern = i8_ptr_type.const_array(&[type_info_int.as_any_value_enum().into_pointer_value()]);
        builder
            .build_landing_pad(
                exception_type,
                personality_function,
                &[filter_pattern.into()],
                false,
                "res",
            )
            .unwrap();

        let fakepi = f32_type.const_zero();

        builder.build_return(Some(&fakepi)).unwrap();
    }

    module.print_to_stderr();
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
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let i8_ptr_type = context.i32_type().ptr_type(AddressSpace::default());
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let i8_ptr_type = context.ptr_type(AddressSpace::default());
    let i64_type = context.i64_type();
    let fn_type = i8_type.fn_type(&[i8_ptr_type.into()], false);
    let neg_one = i8_type.const_all_ones();
    let one = i64_type.const_int(1, false);

    let function = module.add_function("check_null_index1", fn_type, None);
    let entry = context.append_basic_block(function, "entry");

    builder.position_at_end(entry);

    let ptr = function.get_first_param().unwrap().into_pointer_value();

    let is_null = builder.build_is_null(ptr, "is_null").unwrap();

    let ret_0 = context.append_basic_block(function, "ret_0");
    let ret_idx = context.append_basic_block(function, "ret_idx");

    builder.build_conditional_branch(is_null, ret_0, ret_idx).unwrap();

    builder.position_at_end(ret_0);
    builder.build_return(Some(&neg_one)).unwrap();

    builder.position_at_end(ret_idx);

    // FIXME: This might not work if compiled on non 64bit devices. Ideally we'd
    // be able to create pointer sized ints easily
    let ptr_as_int = builder.build_ptr_to_int(ptr, i64_type, "ptr_as_int").unwrap();
    let new_ptr_as_int = builder.build_int_add(ptr_as_int, one, "add").unwrap();
    let new_ptr = builder
        .build_int_to_ptr(new_ptr_as_int, i8_ptr_type, "int_as_ptr")
        .unwrap();
    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    let index1 = builder.build_load(new_ptr, "deref").unwrap();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let index1 = builder.build_load(i8_ptr_type, new_ptr, "deref").unwrap();

    builder.build_return(Some(&index1)).unwrap();

    // Here we're going to create a function that looks roughly like:
    // fn check_null_index2(ptr: *const i8) -> i8 {
    //     if !ptr.is_null() {
    //         ptr[1]
    //     } else {
    //         -1
    //     }
    // }

    let function = module.add_function("check_null_index2", fn_type, None);
    let entry = context.append_basic_block(function, "entry");

    builder.position_at_end(entry);

    let ptr = function.get_first_param().unwrap().into_pointer_value();

    let is_not_null = builder.build_is_not_null(ptr, "is_not_null").unwrap();

    let ret_idx = context.append_basic_block(function, "ret_idx");
    let ret_0 = context.append_basic_block(function, "ret_0");

    builder.build_conditional_branch(is_not_null, ret_idx, ret_0).unwrap();

    builder.position_at_end(ret_0);
    builder.build_return(Some(&neg_one)).unwrap();

    builder.position_at_end(ret_idx);

    // FIXME: This might not work if compiled on non 64bit devices. Ideally we'd
    // be able to create pointer sized ints easily
    let ptr_as_int = builder.build_ptr_to_int(ptr, i64_type, "ptr_as_int").unwrap();
    let new_ptr_as_int = builder.build_int_add(ptr_as_int, one, "add").unwrap();
    let new_ptr = builder
        .build_int_to_ptr(new_ptr_as_int, i8_ptr_type, "int_as_ptr")
        .unwrap();
    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    let index1 = builder.build_load(new_ptr, "deref").unwrap();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let index1 = builder.build_load(i8_ptr_type, new_ptr, "deref").unwrap();

    builder.build_return(Some(&index1)).unwrap();

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    unsafe {
        let check_null_index1 = execution_engine
            .get_function::<unsafe extern "C" fn(*const i8) -> i8>("check_null_index1")
            .unwrap();

        let array = &[100i8, 42i8];

        assert_eq!(check_null_index1.call(null()), -1i8);
        assert_eq!(check_null_index1.call(array.as_ptr()), 42i8);

        let check_null_index2 = execution_engine
            .get_function::<unsafe extern "C" fn(*const i8) -> i8>("check_null_index2")
            .unwrap();

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
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let left = fn_value.get_first_param().unwrap().into_int_value();
    let right = fn_value.get_last_param().unwrap().into_int_value();

    let and = builder.build_and(left, right, "and_op").unwrap();

    builder.build_return(Some(&and)).unwrap();

    // Here we're going to create an or function which looks roughly like:
    // fn or(left: bool, right: bool) -> bool {
    //     left || right
    // }

    let fn_value = module.add_function("or", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let left = fn_value.get_first_param().unwrap().into_int_value();
    let right = fn_value.get_last_param().unwrap().into_int_value();

    let or = builder.build_or(left, right, "or_op").unwrap();

    builder.build_return(Some(&or)).unwrap();

    // Here we're going to create a xor function which looks roughly like:
    // fn xor(left: bool, right: bool) -> bool {
    //     left || right
    // }

    let fn_value = module.add_function("xor", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let left = fn_value.get_first_param().unwrap().into_int_value();
    let right = fn_value.get_last_param().unwrap().into_int_value();

    let xor = builder.build_xor(left, right, "xor_op").unwrap();

    builder.build_return(Some(&xor)).unwrap();

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
    let entry = context.append_basic_block(fn_value, "entry");
    let check = context.append_basic_block(fn_value, "check");
    let elif = context.append_basic_block(fn_value, "elif");
    let else_ = context.append_basic_block(fn_value, "else");
    let value = fn_value.get_first_param().unwrap().into_int_value();

    builder.position_at_end(entry);
    builder
        .build_switch(value, else_, &[(i8_zero, check), (i8_42, elif)])
        .unwrap();

    builder.position_at_end(check);
    builder.build_return(Some(&i8_one)).unwrap();

    builder.position_at_end(elif);
    builder.build_return(Some(&i8_255)).unwrap();

    builder.position_at_end(else_);

    let double = builder.build_int_mul(value, i8_two, "double").unwrap();

    builder.build_return(Some(&double)).unwrap();

    unsafe {
        let switch = execution_engine
            .get_function::<unsafe extern "C" fn(u8) -> u8>("switch")
            .unwrap();

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

    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let shift = builder.build_left_shift(value, bits, "shl").unwrap();

    builder.build_return(Some(&shift)).unwrap();

    // Here we're going to create a function which looks roughly like:
    // fn right_shift(value: u8, bits: u8) -> u8 {
    //     value >> bits
    // }
    let fn_value = module.add_function("right_shift", fn_type, None);
    let value = fn_value.get_first_param().unwrap().into_int_value();
    let bits = fn_value.get_nth_param(1).unwrap().into_int_value();

    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let shift = builder.build_right_shift(value, bits, false, "shr").unwrap();

    builder.build_return(Some(&shift)).unwrap();

    // Here we're going to create a function which looks roughly like:
    // fn right_shift(value: u8, bits: u8) -> u8 {
    //     value >> bits
    // }
    let fn_value = module.add_function("right_shift_sign_extend", fn_type, None);
    let value = fn_value.get_first_param().unwrap().into_int_value();
    let bits = fn_value.get_nth_param(1).unwrap().into_int_value();

    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let shift = builder.build_right_shift(value, bits, true, "shr").unwrap();

    builder.build_return(Some(&shift)).unwrap();

    unsafe {
        let left_shift = execution_engine
            .get_function::<unsafe extern "C" fn(u8, u8) -> u8>("left_shift")
            .unwrap();
        let right_shift = execution_engine
            .get_function::<unsafe extern "C" fn(u8, u8) -> u8>("right_shift")
            .unwrap();
        let right_shift_sign_extend = execution_engine
            .get_function::<unsafe extern "C" fn(i8, u8) -> i8>("right_shift_sign_extend")
            .unwrap();

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
fn test_unconditional_branch() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("my_fn", fn_type, None);
    let entry_bb = context.append_basic_block(fn_value, "entry");
    let skipped_bb = context.append_basic_block(fn_value, "skipped");
    let end_bb = context.append_basic_block(fn_value, "end");

    builder.position_at_end(entry_bb);
    builder.build_unconditional_branch(end_bb).unwrap();

    builder.position_at_end(skipped_bb);
    builder.build_unreachable().unwrap();
}

#[test]
fn test_no_builder_double_free() {
    let context = Context::create();
    let builder = context.create_builder();

    drop(builder);
    drop(context);
}

// TODO: Compile-fail test this:
// let builder = {
//     let context = Context::create();
//
//     context.create_builder()
// };

#[test]
fn test_no_builder_double_free2() {
    let context = Context::create();
    let builder = context.create_builder();

    // Builder continues to function with different context
    let context = Context::create();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("my_fn", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);
    // FIXME: Builder segfaults when making build calls with different context
    // as of newer rust versions(late 2018+?). Maybe this isn't actually something
    // you're suppose to do in LLVM and LTO(?) has made it a more prominent issue?
    // builder.build_unreachable();

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
    let entry = context.append_basic_block(fn_value, "entry");
    let builder = context.create_builder();

    builder.position_at_end(entry);
    let in_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let casted_vec = builder.build_int_cast(in_vec, int32_vec_type, "casted_vec").unwrap();
    let _uncasted_vec = builder.build_int_cast(casted_vec, int8_vec_type, "uncasted_vec");
    builder.build_return(Some(&casted_vec)).unwrap();
    assert!(fn_value.verify(true));

    // Here we're building a function that takes in a <3 x f32> and returns it casted to and from a <3 x f16>
    let fn_type = float16_vec_type.fn_type(&[float32_vec_type.into()], false);
    let fn_value = module.add_function("test_float_vec_cast", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");
    let builder = context.create_builder();

    builder.position_at_end(entry);
    let in_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let casted_vec = builder
        .build_float_cast(in_vec, float16_vec_type, "casted_vec")
        .unwrap();
    let _uncasted_vec = builder.build_float_cast(casted_vec, float32_vec_type, "uncasted_vec");
    builder.build_return(Some(&casted_vec)).unwrap();
    assert!(fn_value.verify(true));

    // Here we're building a function that takes in a <3 x f32> and returns it casted to and from a <3 x i32>
    let fn_type = int32_vec_type.fn_type(&[float32_vec_type.into()], false);
    let fn_value = module.add_function("test_float_to_int_vec_cast", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");
    let builder = context.create_builder();

    builder.position_at_end(entry);
    let in_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let casted_vec = builder
        .build_float_to_signed_int(in_vec, int32_vec_type, "casted_vec")
        .unwrap();
    let _uncasted_vec = builder.build_signed_int_to_float(casted_vec, float32_vec_type, "uncasted_vec");
    builder.build_return(Some(&casted_vec)).unwrap();
    assert!(fn_value.verify(true));
}

#[llvm_versions(8..)]
#[test]
fn test_vector_convert_ops_respect_target_signedness() {
    let context = Context::create();
    let module = context.create_module("test");
    let int8_vec_type = context.i8_type().vec_type(3);

    // Here we're building a function that takes in a <3 x i8> (signed) and returns it casted to and from a <3 x i8> (unsigned)
    let fn_type = int8_vec_type.fn_type(&[int8_vec_type.into()], false);
    let fn_value = module.add_function("test_int_vec_cast", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");
    let builder = context.create_builder();

    builder.position_at_end(entry);
    let in_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let casted_vec = builder
        .build_int_cast_sign_flag(in_vec, int8_vec_type, true, "casted_vec")
        .unwrap();
    let _uncasted_vec = builder.build_int_cast_sign_flag(casted_vec, int8_vec_type, true, "uncasted_vec");
    builder.build_return(Some(&casted_vec)).unwrap();

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
    let fn_type = int32_vec_type.fn_type(
        &[int32_vec_type.into(), int32_vec_type.into(), int32_vec_type.into()],
        false,
    );
    let fn_value = module.add_function("test_int_vec_add", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");
    let builder = context.create_builder();

    builder.position_at_end(entry);
    let p1_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let p2_vec = fn_value.get_nth_param(1).unwrap().into_vector_value();
    let p3_vec = fn_value.get_nth_param(2).unwrap().into_vector_value();
    let added_vec = builder.build_int_add(p1_vec, p2_vec, "added_vec").unwrap();
    let added_vec = builder.build_int_add(added_vec, p3_vec, "added_vec").unwrap();
    builder.build_return(Some(&added_vec)).unwrap();
    assert!(fn_value.verify(true));

    // Here we're building a function that takes in three <2 x f32>s and returns x * y / z as an
    // <2 x f32>
    let fn_type = float32_vec_type.fn_type(
        &[
            float32_vec_type.into(),
            float32_vec_type.into(),
            float32_vec_type.into(),
        ],
        false,
    );
    let fn_value = module.add_function("test_float_vec_mul", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");
    let builder = context.create_builder();

    builder.position_at_end(entry);
    let p1_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let p2_vec = fn_value.get_nth_param(1).unwrap().into_vector_value();
    let p3_vec = fn_value.get_nth_param(2).unwrap().into_vector_value();
    let multiplied_vec = builder.build_float_mul(p1_vec, p2_vec, "multiplied_vec").unwrap();
    let divided_vec = builder.build_float_div(multiplied_vec, p3_vec, "divided_vec").unwrap();
    builder.build_return(Some(&divided_vec)).unwrap();
    assert!(fn_value.verify(true));

    // Here we're building a function that takes two <2 x f32>s and a <2 x bool> and returns (x < y) * z
    // as a <2 x bool>
    let fn_type = bool_vec_type.fn_type(
        &[float32_vec_type.into(), float32_vec_type.into(), bool_vec_type.into()],
        false,
    );
    let fn_value = module.add_function("test_float_vec_compare", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");
    let builder = context.create_builder();

    builder.position_at_end(entry);
    let p1_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let p2_vec = fn_value.get_nth_param(1).unwrap().into_vector_value();
    let p3_vec = fn_value.get_nth_param(2).unwrap().into_vector_value();
    let compared_vec = builder
        .build_float_compare(inkwell::FloatPredicate::OLT, p1_vec, p2_vec, "compared_vec")
        .unwrap();
    let multiplied_vec = builder.build_int_mul(compared_vec, p3_vec, "multiplied_vec").unwrap();
    builder.build_return(Some(&multiplied_vec)).unwrap();
    assert!(fn_value.verify(true));
}

#[test]
fn test_vector_pointer_ops() {
    let context = Context::create();
    let module = context.create_module("test");
    let int32_vec_type = context.i32_type().vec_type(4);
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let i8_ptr_vec_type = context.i8_type().ptr_type(AddressSpace::default()).vec_type(4);
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let i8_ptr_vec_type = context.ptr_type(AddressSpace::default()).vec_type(4);
    let bool_vec_type = context.bool_type().vec_type(4);

    // Here we're building a function that takes a <4 x i32>, converts it to a <4 x i8*> and returns a
    // <4 x bool> if the pointer is null
    let fn_type = bool_vec_type.fn_type(&[int32_vec_type.into()], false);
    let fn_value = module.add_function("test_ptr_null", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");
    let builder = context.create_builder();

    builder.position_at_end(entry);
    let in_vec = fn_value.get_first_param().unwrap().into_vector_value();
    let ptr_vec = builder.build_int_to_ptr(in_vec, i8_ptr_vec_type, "ptr_vec").unwrap();
    let is_null_vec = builder.build_is_null(ptr_vec, "is_null_vec").unwrap();
    builder.build_return(Some(&is_null_vec)).unwrap();
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
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let array_alloca = builder.build_alloca(array_type, "array_alloca").unwrap();
    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    let array = builder
        .build_load(array_alloca, "array_load")
        .unwrap()
        .into_array_value();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let array = builder
        .build_load(array_type, array_alloca, "array_load")
        .unwrap()
        .into_array_value();
    let const_int1 = i32_type.const_int(2, false);
    let const_int2 = i32_type.const_int(5, false);
    let const_int3 = i32_type.const_int(6, false);
    let const_float = f32_type.const_float(2.0);

    assert!(builder
        .build_insert_value(array, const_int1, 0, "insert")
        .unwrap()
        .is_array_value());
    assert!(builder
        .build_insert_value(array, const_int2, 1, "insert")
        .unwrap()
        .is_array_value());
    assert!(builder
        .build_insert_value(array, const_int3, 2, "insert")
        .unwrap()
        .is_array_value());
    assert!(builder
        .build_insert_value(array, const_int3, 3, "insert")
        .is_err_and(|e| e == BuilderError::ExtractOutOfRange));
    assert!(builder
        .build_insert_value(array, const_int3, 4, "insert")
        .is_err_and(|e| e == BuilderError::ExtractOutOfRange));

    assert!(builder.build_extract_value(array, 0, "extract").unwrap().is_int_value());
    assert!(builder.build_extract_value(array, 1, "extract").unwrap().is_int_value());
    assert!(builder.build_extract_value(array, 2, "extract").unwrap().is_int_value());
    assert!(builder
        .build_extract_value(array, 3, "extract")
        .is_err_and(|e| e == BuilderError::ExtractOutOfRange));

    let struct_alloca = builder.build_alloca(struct_type, "struct_alloca").unwrap();
    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    let struct_value = builder
        .build_load(struct_alloca, "struct_load")
        .unwrap()
        .into_struct_value();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let struct_value = builder
        .build_load(struct_type, struct_alloca, "struct_load")
        .unwrap()
        .into_struct_value();

    assert!(builder
        .build_insert_value(struct_value, const_int2, 0, "insert")
        .unwrap()
        .is_struct_value());
    assert!(builder
        .build_insert_value(struct_value, const_float, 1, "insert")
        .unwrap()
        .is_struct_value());
    assert!(builder
        .build_insert_value(struct_value, const_float, 2, "insert")
        .is_err_and(|e| e == BuilderError::ExtractOutOfRange));
    assert!(builder
        .build_insert_value(struct_value, const_float, 3, "insert")
        .is_err_and(|e| e == BuilderError::ExtractOutOfRange));

    assert!(builder
        .build_extract_value(struct_value, 0, "extract")
        .unwrap()
        .is_int_value());
    assert!(builder
        .build_extract_value(struct_value, 1, "extract")
        .unwrap()
        .is_float_value());
    assert!(builder
        .build_extract_value(struct_value, 2, "extract")
        .is_err_and(|e| e == BuilderError::ExtractOutOfRange));
    assert!(builder
        .build_extract_value(struct_value, 3, "extract")
        .is_err_and(|e| e == BuilderError::ExtractOutOfRange));

    builder.build_return(None).unwrap();

    assert!(module.verify().is_ok());
}

#[test]
fn test_insert_element() {
    use inkwell::types::IntType;
    use inkwell::values::VectorValue;

    let context = Context::create();
    let module = context.create_module("vec");

    let fn_type = context.void_type().fn_type(&[], false);
    let fn_value = module.add_function("vec_fn", fn_type, None);
    let builder = context.create_builder();
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    #[llvm_versions(12..)]
    fn get_empty_vector_of(ty: IntType<'_>) -> VectorValue<'_> {
        ty.vec_type(4).get_poison()
    }
    #[llvm_versions(..12)]
    fn get_empty_vector_of(ty: IntType<'_>) -> VectorValue<'_> {
        ty.vec_type(4).get_undef()
    }

    let i8_ty = context.i8_type();
    let i32_ty = context.i32_type();
    let mut v = get_empty_vector_of(i8_ty);
    v = builder
        .build_insert_element(v, i8_ty.const_int(0, false), i32_ty.const_int(0, false), "v0")
        .unwrap();
    v = builder
        .build_insert_element(v, i8_ty.const_int(1, false), i32_ty.const_int(1, false), "v1")
        .unwrap();
    v = builder
        .build_insert_element(v, i8_ty.const_int(2, false), i32_ty.const_int(2, false), "v2")
        .unwrap();
    v = builder
        .build_insert_element(v, i8_ty.const_int(3, false), i32_ty.const_int(3, false), "v3")
        .unwrap();
    let _ = v;

    builder.build_return(None).unwrap();

    assert!(module.verify().is_ok());
}

fn is_alignment_ok(align: u32) -> bool {
    // This replicates the assertions LLVM runs.
    //
    // See https://github.com/TheDan64/inkwell/issues/168
    align > 0 && align.is_power_of_two() && (align as f64).log2() < 64.0
}

#[llvm_versions(8..)]
#[test]
fn test_alignment_bytes() {
    let verify_alignment = |alignment: u32| {
        let context = Context::create();
        let module = context.create_module("av");
        let result = run_memcpy_on(&context, &module, alignment);

        if is_alignment_ok(alignment) {
            assert!(
                result.is_ok() && module.verify().is_ok(),
                "alignment of {} was a power of 2 under 2^64, but did not verify for memcpy.",
                alignment
            );
        } else {
            assert!(result.is_err(), "alignment of {} was a power of 2 under 2^64, yet verification passed for memcpy when it should not have.", alignment);
        }

        let result = run_memmove_on(&context, &module, alignment);

        if is_alignment_ok(alignment) {
            assert!(
                result.is_ok() && module.verify().is_ok(),
                "alignment of {} was a power of 2 under 2^64, but did not verify for memmove.",
                alignment
            );
        } else {
            assert!(result.is_err(), "alignment of {} was a power of 2 under 2^64, yet verification passed for memmove when it should not have.", alignment);
        }
    };

    for alignment in 0..32 {
        verify_alignment(alignment);
    }

    verify_alignment(u32::MAX);
}

#[llvm_versions(8..)]
fn run_memcpy_on<'ctx>(
    context: &'ctx Context,
    module: &inkwell::module::Module<'ctx>,
    alignment: u32,
) -> Result<(), BuilderError> {
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let array_len = 4;
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let fn_type = i32_type.ptr_type(AddressSpace::default()).fn_type(&[], false);
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let fn_type = context.ptr_type(AddressSpace::default()).fn_type(&[], false);
    let fn_value = module.add_function("test_fn", fn_type, None);
    let builder = context.create_builder();
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let len_value = i64_type.const_int(array_len as u64, false);
    let element_type = i32_type;
    let array_ptr = builder.build_array_malloc(i32_type, len_value, "array_ptr").unwrap();

    // Initialize the array with the values [1, 2, 3, 4]
    for index in 0..4 {
        let index_val = i32_type.const_int(index, false);
        #[cfg(any(
            feature = "llvm4-0",
            feature = "llvm5-0",
            feature = "llvm6-0",
            feature = "llvm7-0",
            feature = "llvm8-0",
            feature = "llvm9-0",
            feature = "llvm10-0",
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0"
        ))]
        let elem_ptr = unsafe { builder.build_in_bounds_gep(array_ptr, &[index_val], "index") }.unwrap();
        #[cfg(any(
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        let elem_ptr = unsafe {
            builder
                .build_in_bounds_gep(element_type, array_ptr, &[index_val], "index")
                .unwrap()
        };
        let int_val = i32_type.const_int(index + 1, false);

        builder.build_store(elem_ptr, int_val).unwrap();
    }

    // Memcpy the first half of the array over the second half of the array.
    let elems_to_copy = 2;
    let bytes_to_copy = elems_to_copy * std::mem::size_of::<i32>();
    let size_val = i64_type.const_int(bytes_to_copy as u64, false);
    let index_val = i32_type.const_int(2, false);
    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    let dest_ptr = unsafe { builder.build_in_bounds_gep(array_ptr, &[index_val], "index") }.unwrap();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let dest_ptr = unsafe {
        builder
            .build_in_bounds_gep(element_type, array_ptr, &[index_val], "index")
            .unwrap()
    };

    builder.build_memcpy(dest_ptr, alignment, array_ptr, alignment, size_val)?;

    builder.build_return(Some(&array_ptr)).unwrap();

    Ok(())
}

#[llvm_versions(8..)]
#[test]
fn test_memcpy() {
    // 1. Allocate an array with a few elements.
    // 2. Memcpy from the first half of the array to the second half.
    // 3. Run the code in an execution engine and verify the array's contents.
    let context = Context::create();
    let module = context.create_module("av");

    assert!(run_memcpy_on(&context, &module, 8).is_ok());

    // Verify the module
    if let Err(errors) = module.verify() {
        panic!("Errors defining module: {:?}", errors);
    }

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    unsafe {
        let func = execution_engine
            .get_function::<unsafe extern "C" fn() -> *const i32>("test_fn")
            .unwrap();
        let actual: &[i32] = std::slice::from_raw_parts(func.call(), 4);

        assert_eq!(&[1, 2, 1, 2], actual);
    }
}

#[llvm_versions(8..)]
fn run_memmove_on<'ctx>(
    context: &'ctx Context,
    module: &inkwell::module::Module<'ctx>,
    alignment: u32,
) -> Result<(), BuilderError> {
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let array_len = 4;
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let fn_type = i32_type.ptr_type(AddressSpace::default()).fn_type(&[], false);
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let fn_type = context.ptr_type(AddressSpace::default()).fn_type(&[], false);
    let fn_value = module.add_function("test_fn", fn_type, None);
    let builder = context.create_builder();
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let len_value = i64_type.const_int(array_len as u64, false);
    let element_type = i32_type;
    let array_ptr = builder.build_array_malloc(i32_type, len_value, "array_ptr").unwrap();

    // Initialize the array with the values [1, 2, 3, 4]
    for index in 0..4 {
        let index_val = i32_type.const_int(index, false);
        #[cfg(any(
            feature = "llvm4-0",
            feature = "llvm5-0",
            feature = "llvm6-0",
            feature = "llvm7-0",
            feature = "llvm8-0",
            feature = "llvm9-0",
            feature = "llvm10-0",
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0"
        ))]
        let elem_ptr = unsafe { builder.build_in_bounds_gep(array_ptr, &[index_val], "index") }.unwrap();
        #[cfg(any(
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        let elem_ptr = unsafe {
            builder
                .build_in_bounds_gep(element_type, array_ptr, &[index_val], "index")
                .unwrap()
        };
        let int_val = i32_type.const_int(index + 1, false);

        builder.build_store(elem_ptr, int_val).unwrap();
    }

    // Memcpy the first half of the array over the second half of the array.
    let elems_to_copy = 2;
    let bytes_to_copy = elems_to_copy * std::mem::size_of::<i32>();
    let size_val = i64_type.const_int(bytes_to_copy as u64, false);
    let index_val = i32_type.const_int(2, false);
    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    let dest_ptr = unsafe { builder.build_in_bounds_gep(array_ptr, &[index_val], "index") }.unwrap();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let dest_ptr = unsafe {
        builder
            .build_in_bounds_gep(element_type, array_ptr, &[index_val], "index")
            .unwrap()
    };

    builder.build_memmove(dest_ptr, alignment, array_ptr, alignment, size_val)?;

    builder.build_return(Some(&array_ptr)).unwrap();

    Ok(())
}

#[llvm_versions(8..)]
#[test]
fn test_memmove() {
    // 1. Allocate an array with a few elements.
    // 2. Memmove from the first half of the array to the second half.
    // 3. Run the code in an execution engine and verify the array's contents.
    let context = Context::create();
    let module = context.create_module("av");

    assert!(run_memmove_on(&context, &module, 8).is_ok());

    // Verify the module
    if let Err(errors) = module.verify() {
        panic!("Errors defining module: {:?}", errors);
    }

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    unsafe {
        let func = execution_engine
            .get_function::<unsafe extern "C" fn() -> *const i32>("test_fn")
            .unwrap();
        let actual = std::slice::from_raw_parts(func.call(), 4);

        assert_eq!(&[1, 2, 1, 2], actual);
    }
}

#[llvm_versions(8..)]
fn run_memset_on<'ctx>(
    context: &'ctx Context,
    module: &inkwell::module::Module<'ctx>,
    alignment: u32,
) -> Result<(), BuilderError> {
    let i8_type = context.i8_type();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let array_len = 4;
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let fn_type = i32_type.ptr_type(AddressSpace::default()).fn_type(&[], false);
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let fn_type = context.ptr_type(AddressSpace::default()).fn_type(&[], false);
    let fn_value = module.add_function("test_fn", fn_type, None);
    let builder = context.create_builder();
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let len_value = i64_type.const_int(array_len as u64, false);
    let element_type = i32_type;
    let array_ptr = builder.build_array_malloc(i32_type, len_value, "array_ptr").unwrap();

    let elems_to_copy = 2;
    let bytes_to_copy = elems_to_copy * std::mem::size_of::<i32>();
    let size_val = i64_type.const_int(bytes_to_copy as u64, false);
    // Memset the first half of the array as 0
    let val = i8_type.const_zero();
    builder.build_memset(array_ptr, alignment, val, size_val)?;
    // Memset the second half of the array as -1
    let val = i8_type.const_all_ones();
    let index = i32_type.const_int(2, false);
    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    let part_2 = unsafe { builder.build_in_bounds_gep(array_ptr, &[index], "index") }.unwrap();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let part_2 = unsafe {
        builder
            .build_in_bounds_gep(element_type, array_ptr, &[index], "index")
            .unwrap()
    };
    builder.build_memset(part_2, alignment, val, size_val)?;
    builder.build_return(Some(&array_ptr)).unwrap();

    Ok(())
}

#[llvm_versions(8..)]
#[test]
fn test_memset() {
    // 1. Allocate an array with a few elements.
    // 2. Memmove from the first half of the array to the second half.
    // 3. Run the code in an execution engine and verify the array's contents.
    let context = Context::create();
    let module = context.create_module("av");

    assert!(run_memset_on(&context, &module, 8).is_ok());

    // Verify the module
    if let Err(errors) = module.verify() {
        panic!("Errors defining module: {:?}", errors);
    }

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();

    unsafe {
        let func = execution_engine
            .get_function::<unsafe extern "C" fn() -> *const i32>("test_fn")
            .unwrap();
        let actual = std::slice::from_raw_parts(func.call(), 4);

        assert_eq!(&[0, 0, -1, -1], actual);
    }
}

#[test]
fn test_bit_cast() {
    use inkwell::values::BasicValue;

    let context = Context::create();
    let module = context.create_module("bc");
    let void_type = context.void_type();
    let f32_type = context.f32_type();
    let i32_type = context.i32_type();
    let f64_type = context.f64_type();
    let i64_type = context.i64_type();
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let i32_ptr_type = i32_type.ptr_type(AddressSpace::default());
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let i32_ptr_type = context.ptr_type(AddressSpace::default());
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let i64_ptr_type = i64_type.ptr_type(AddressSpace::default());
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let i64_ptr_type = context.ptr_type(AddressSpace::default());
    let i32_vec_type = i32_type.vec_type(2);
    let arg_types = [
        i32_type.into(),
        f32_type.into(),
        i32_vec_type.into(),
        i32_ptr_type.into(),
        f64_type.into(),
    ];
    let fn_type = void_type.fn_type(&arg_types, false);
    let fn_value = module.add_function("bc", fn_type, None);
    let builder = context.create_builder();
    let entry = context.append_basic_block(fn_value, "entry");
    let i32_arg = fn_value.get_first_param().unwrap();
    let f32_arg = fn_value.get_nth_param(1).unwrap();
    let i32_vec_arg = fn_value.get_nth_param(2).unwrap();
    let i32_ptr_arg = fn_value.get_nth_param(3).unwrap();
    let f64_arg = fn_value.get_nth_param(4).unwrap();

    builder.position_at_end(entry);

    let cast = builder.build_bit_cast(i32_arg, f32_type, "i32tof32").unwrap();

    builder.build_bit_cast(f32_arg, f32_type, "f32tof32").unwrap();
    builder.build_bit_cast(i32_vec_arg, i64_type, "2xi32toi64").unwrap();
    builder.build_bit_cast(i32_ptr_arg, i64_ptr_type, "i32*toi64*").unwrap();

    builder.build_return(None).unwrap();

    assert!(module.verify().is_ok(), "{}", module.print_to_string().to_string());

    let first_iv = cast.as_instruction_value().unwrap();

    builder.position_before(&first_iv);
    builder.build_bit_cast(f64_arg, i64_type, "f64toi64").unwrap();

    assert!(module.verify().is_ok());
}

#[test]
fn test_atomicrmw() {
    let context = Context::create();
    let module = context.create_module("rmw");

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");
    let builder = context.create_builder();
    builder.position_at_end(entry);

    let i32_type = context.i32_type();
    let i31_type = context.custom_width_int_type(31);
    let i4_type = context.custom_width_int_type(4);

    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let ptr_value = i32_type.ptr_type(AddressSpace::default()).get_undef();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let ptr_value = context.ptr_type(AddressSpace::default()).get_undef();
    let zero_value = i32_type.const_zero();
    let result = builder.build_atomicrmw(AtomicRMWBinOp::Add, ptr_value, zero_value, AtomicOrdering::Unordered);
    assert!(result.is_ok());

    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    {
        let i64_type = context.i64_type();
        let ptr_value = i64_type.ptr_type(AddressSpace::default()).get_undef();
        let zero_value = i32_type.const_zero();
        let result = builder.build_atomicrmw(AtomicRMWBinOp::Add, ptr_value, zero_value, AtomicOrdering::Unordered);
        assert!(result.is_err());
    }

    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let ptr_value = i31_type.ptr_type(AddressSpace::default()).get_undef();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let ptr_value = context.ptr_type(AddressSpace::default()).get_undef();
    let zero_value = i31_type.const_zero();
    let result = builder.build_atomicrmw(AtomicRMWBinOp::Add, ptr_value, zero_value, AtomicOrdering::Unordered);
    assert!(result.is_err());

    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let ptr_value = i4_type.ptr_type(AddressSpace::default()).get_undef();
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let ptr_value = context.ptr_type(AddressSpace::default()).get_undef();
    let zero_value = i4_type.const_zero();
    let result = builder.build_atomicrmw(AtomicRMWBinOp::Add, ptr_value, zero_value, AtomicOrdering::Unordered);
    assert!(result.is_err());
}

#[test]
fn test_cmpxchg() {
    let context = Context::create();
    let module = context.create_module("cmpxchg");

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");
    let builder = context.create_builder();
    builder.position_at_end(entry);

    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let i32_ptr_type = i32_type.ptr_type(AddressSpace::default());
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let i32_ptr_type = context.ptr_type(AddressSpace::default());
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let i32_ptr_ptr_type = i32_ptr_type.ptr_type(AddressSpace::default());
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let i32_ptr_ptr_type = context.ptr_type(AddressSpace::default());

    let ptr_value = i32_ptr_type.get_undef();
    let zero_value = i32_type.const_zero();
    let neg_one_value = i32_type.const_all_ones();
    let result = builder.build_cmpxchg(
        ptr_value,
        zero_value,
        neg_one_value,
        AtomicOrdering::Monotonic,
        AtomicOrdering::Monotonic,
    );
    assert!(result.is_ok());

    let ptr_value = i32_ptr_type.get_undef();
    let zero_value = i32_type.const_zero();
    let neg_one_value = i32_type.const_all_ones();
    let result = builder.build_cmpxchg(
        ptr_value,
        zero_value,
        neg_one_value,
        AtomicOrdering::Unordered,
        AtomicOrdering::Monotonic,
    );
    assert!(result.is_err());

    let ptr_value = i32_ptr_type.get_undef();
    let zero_value = i32_type.const_zero();
    let neg_one_value = i32_type.const_all_ones();
    let result = builder.build_cmpxchg(
        ptr_value,
        zero_value,
        neg_one_value,
        AtomicOrdering::Monotonic,
        AtomicOrdering::Unordered,
    );
    assert!(result.is_err());

    let ptr_value = i32_ptr_type.get_undef();
    let zero_value = i32_type.const_zero();
    let neg_one_value = i32_type.const_all_ones();
    let result = builder.build_cmpxchg(
        ptr_value,
        zero_value,
        neg_one_value,
        AtomicOrdering::Monotonic,
        AtomicOrdering::Release,
    );
    assert!(result.is_err());

    let ptr_value = i32_ptr_type.get_undef();
    let zero_value = i32_type.const_zero();
    let neg_one_value = i32_type.const_all_ones();
    let result = builder.build_cmpxchg(
        ptr_value,
        zero_value,
        neg_one_value,
        AtomicOrdering::Monotonic,
        AtomicOrdering::AcquireRelease,
    );
    assert!(result.is_err());

    let ptr_value = i32_ptr_type.get_undef();
    let zero_value = i32_type.const_zero();
    let neg_one_value = i32_type.const_all_ones();
    let result = builder.build_cmpxchg(
        ptr_value,
        zero_value,
        neg_one_value,
        AtomicOrdering::Monotonic,
        AtomicOrdering::SequentiallyConsistent,
    );
    assert!(result.is_err());

    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    {
        let ptr_value = i32_ptr_ptr_type.get_undef();
        let zero_value = i32_type.const_zero();
        let neg_one_value = i32_type.const_all_ones();
        let result = builder.build_cmpxchg(
            ptr_value,
            zero_value,
            neg_one_value,
            AtomicOrdering::Monotonic,
            AtomicOrdering::Monotonic,
        );
        assert!(result.is_err());
    }

    let ptr_value = i32_ptr_type.get_undef();
    let zero_value = i64_type.const_zero();
    let neg_one_value = i32_type.const_all_ones();
    let result = builder.build_cmpxchg(
        ptr_value,
        zero_value,
        neg_one_value,
        AtomicOrdering::Monotonic,
        AtomicOrdering::Monotonic,
    );
    assert!(result.is_err());

    let ptr_value = i32_ptr_type.get_undef();
    let zero_value = i32_type.const_zero();
    let neg_one_value = i64_type.const_all_ones();
    let result = builder.build_cmpxchg(
        ptr_value,
        zero_value,
        neg_one_value,
        AtomicOrdering::Monotonic,
        AtomicOrdering::Monotonic,
    );
    assert!(result.is_err());

    let ptr_value = i32_ptr_ptr_type.get_undef();
    let zero_value = i32_ptr_type.const_zero();
    let neg_one_value = i32_ptr_type.const_zero();
    let result = builder.build_cmpxchg(
        ptr_value,
        zero_value,
        neg_one_value,
        AtomicOrdering::Monotonic,
        AtomicOrdering::Monotonic,
    );
    assert!(result.is_ok());

    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    {
        let ptr_value = i32_ptr_type.get_undef();
        let zero_value = i32_ptr_type.const_zero();
        let neg_one_value = i32_ptr_type.const_zero();
        let result = builder.build_cmpxchg(
            ptr_value,
            zero_value,
            neg_one_value,
            AtomicOrdering::Monotonic,
            AtomicOrdering::Monotonic,
        );
        assert!(result.is_err());
    }
}

#[test]
fn test_safe_struct_gep() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("struct_gep");
    let void_type = context.void_type();
    let i32_ty = context.i32_type();
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let i32_ptr_ty = i32_ty.ptr_type(AddressSpace::default());
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let i32_ptr_ty = context.ptr_type(AddressSpace::default());
    let field_types = &[i32_ty.into(), i32_ty.into()];
    let struct_ty = context.struct_type(field_types, false);
    #[cfg(not(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    )))]
    let struct_ptr_ty = struct_ty.ptr_type(AddressSpace::default());
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    let struct_ptr_ty = context.ptr_type(AddressSpace::default());
    let fn_type = void_type.fn_type(&[i32_ptr_ty.into(), struct_ptr_ty.into()], false);
    let fn_value = module.add_function("", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let i32_ptr = fn_value.get_first_param().unwrap().into_pointer_value();
    let struct_ptr = fn_value.get_last_param().unwrap().into_pointer_value();

    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    {
        assert!(builder.build_struct_gep(i32_ptr, 0, "struct_gep").is_err());
        assert!(builder.build_struct_gep(i32_ptr, 10, "struct_gep").is_err());
        assert!(builder.build_struct_gep(struct_ptr, 0, "struct_gep").is_ok());
        assert!(builder.build_struct_gep(struct_ptr, 1, "struct_gep").is_ok());
        assert!(builder.build_struct_gep(struct_ptr, 2, "struct_gep").is_err());
    }
    #[cfg(any(
        feature = "llvm15-0",
        feature = "llvm16-0",
        feature = "llvm17-0",
        feature = "llvm18-0"
    ))]
    {
        assert!(builder.build_struct_gep(i32_ty, i32_ptr, 0, "struct_gep").is_err());
        assert!(builder.build_struct_gep(i32_ty, i32_ptr, 10, "struct_gep").is_err());
        assert!(builder.build_struct_gep(struct_ty, struct_ptr, 0, "struct_gep").is_ok());
        assert!(builder.build_struct_gep(struct_ty, struct_ptr, 1, "struct_gep").is_ok());
        assert!(builder
            .build_struct_gep(struct_ty, struct_ptr, 2, "struct_gep")
            .is_err());
    }
}
