use std::convert::TryInto;

use inkwell::context::Context;
use inkwell::values::{FloatValue, IntValue, PhiValue, PointerValue};
use inkwell::AddressSpace;

#[test]
fn test_phi_conversion() {
    let context = Context::create();
    let module = context.create_module("phi");
    let builder = context.create_builder();

    let fn_type = context.void_type().fn_type(&[], false);
    let function = module.add_function("do_stuff", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    // test that conversion succeeds
    let bool_type = context.bool_type();
    let expect_phi_name = "phi_node";
    let phi = builder.build_phi(bool_type, expect_phi_name).unwrap();
    let instruction = phi.as_instruction();

    let phi_from_instruction: PhiValue = instruction.try_into().unwrap();
    let name = phi_from_instruction.get_name().to_str().unwrap();
    assert_eq!(name, expect_phi_name);

    // test that conversion fails
    let ret_instruction = builder.build_return(None).unwrap();
    let phi_from_instruction: Result<PhiValue, _> = ret_instruction.try_into();
    assert!(phi_from_instruction.is_err());
}

#[test]
fn test_conversion_to_int_value() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    // Create a function whose the first parameter is of IntType
    let i64_type = context.i64_type();
    let fn_type = context.void_type().fn_type(&[i64_type.into()], false);
    let function = module.add_function("testing", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    // Create an IntType instruction
    let int_arg = function.get_nth_param(0).unwrap().into_int_value();
    let int_const = i64_type.const_int(1, false);
    let int_instr = builder
        .build_int_add(int_arg, int_const, "add")
        .unwrap()
        .as_instruction()
        .unwrap();

    // Test the instruction conversion to an IntValue
    let int_conversion: Result<IntValue, _> = int_instr.try_into();
    assert!(int_conversion.is_ok());

    // Test the instruction conversion to other LLVM Values
    let float_conversion: Result<FloatValue, _> = int_instr.try_into();
    assert!(float_conversion.is_err());
    let ptr_conversion: Result<PointerValue, _> = int_instr.try_into();
    assert!(ptr_conversion.is_err());
}

#[test]
fn test_conversion_to_float_value() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    // Create a function whose the first parameter is of IntType
    let f16_type = context.f16_type();
    let fn_type = context.void_type().fn_type(&[f16_type.into()], false);
    let function = module.add_function("testing", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    // Create a FloatType instruction
    let float_arg = function.get_nth_param(0).unwrap().into_float_value();
    let float_const = f16_type.const_float(1.2);
    let float_instr = builder
        .build_float_add(float_arg, float_const, "add")
        .unwrap()
        .as_instruction()
        .unwrap();

    // Test the instruction conversion to a FloatValue
    let float_conversion: Result<FloatValue, _> = float_instr.try_into();
    assert!(float_conversion.is_ok());

    // Test the instruction conversion to other LLVM Values
    let int_conversion: Result<IntValue, _> = float_instr.try_into();
    assert!(int_conversion.is_err());
    let phi_conversion: Result<PhiValue, _> = float_instr.try_into();
    assert!(phi_conversion.is_err());
}

#[test]
fn test_conversion_to_pointer_value() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    // Create a function whose the first parameter is of IntType
    let fn_type = context.void_type().fn_type(&[], false);
    let function = module.add_function("testing", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);

    // Create a PointerType instruction
    #[cfg(feature = "typed-pointers")]
    let i64_ptr_type = context.i64_type().ptr_type(AddressSpace::default());
    #[cfg(not(feature = "typed-pointers"))]
    let i64_ptr_type = context.ptr_type(AddressSpace::default());
    let alloca_instr = builder
        .build_alloca(i64_ptr_type, "alloca")
        .unwrap()
        .as_instruction()
        .unwrap();

    // Test the instruction conversion to a FloatValue
    let ptr_conversion: Result<PointerValue, _> = alloca_instr.try_into();
    assert!(ptr_conversion.is_ok());

    // Test the instruction conversion to other LLVM Values
    let int_conversion: Result<IntValue, _> = alloca_instr.try_into();
    assert!(int_conversion.is_err());
    let float_conversion: Result<FloatValue, _> = alloca_instr.try_into();
    assert!(float_conversion.is_err());
}
