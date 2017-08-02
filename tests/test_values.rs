extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::module::Linkage::*;
use self::inkwell::values::InstructionOpcode::*;

#[test]
fn test_linkage() {
    let context = Context::create();
    let module = context.create_module("testing");

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("free_f32", &fn_type, None);

    assert_eq!(function.get_linkage(), ExternalLinkage);
}

#[test]
fn test_instructions() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f32_ptr_type = f32_type.ptr_type(0);
    let fn_type = void_type.fn_type(&[&f32_ptr_type], false);

    let function = module.add_function("free_f32", &fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let arg1 = function.get_first_param().unwrap().into_pointer_value();

    let f32_val = f32_type.const_float(3.14);

    let store_instruction = builder.build_store(&arg1, &f32_val);
    let ptr_val = builder.build_ptr_to_int(&arg1, &i64_type, "ptr_val");
    let ptr = builder.build_int_to_ptr(&ptr_val, &f32_ptr_type, "ptr");
    let free_instruction = builder.build_free(&arg1);
    let return_instruction = builder.build_return(None);

    assert_eq!(store_instruction.get_opcode(), Store);
    assert_eq!(ptr_val.as_instruction().unwrap().get_opcode(), PtrToInt);
    assert_eq!(ptr.as_instruction().unwrap().get_opcode(), IntToPtr);
    assert_eq!(free_instruction.get_opcode(), Call);
    assert_eq!(return_instruction.get_opcode(), Return);
}

#[test]
fn test_tail_call() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("do_nothing", &fn_type, None);

    let call_instruction = builder.build_call(&function, &[], "to_infinity_and_beyond", false);

    assert_eq!(call_instruction.right().unwrap().is_tail_call(), false);

    let call_instruction = builder.build_call(&function, &[], "to_infinity_and_beyond", true);

    assert_eq!(call_instruction.right().unwrap().is_tail_call(), true);
}

#[test]
fn test_const_null_ptr() {
    let context = Context::create();
    let void_type = context.void_type();
    let bool_type = context.bool_type();
    let i8_type = context.i8_type();
    let i16_type = context.i16_type();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let i128_type = context.i128_type();
    let f16_type = context.f16_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let f128_type = context.f128_type();

    assert!(void_type.const_null_ptr().is_null());
    assert!(bool_type.const_null_ptr().is_null());
    assert!(i8_type.const_null_ptr().is_null());
    assert!(i16_type.const_null_ptr().is_null());
    assert!(i32_type.const_null_ptr().is_null());
    assert!(i64_type.const_null_ptr().is_null());
    assert!(i128_type.const_null_ptr().is_null());
    assert!(f16_type.const_null_ptr().is_null());
    assert!(f32_type.const_null_ptr().is_null());
    assert!(f64_type.const_null_ptr().is_null());
    assert!(f128_type.const_null_ptr().is_null());
}
