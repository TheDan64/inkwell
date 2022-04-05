use std::convert::TryInto;

use inkwell::context::Context;
use inkwell::values::{
    BasicValue, BasicValueEnum, InstructionOpcode::*, InstructionValue, PhiValue,
};
use inkwell::{AddressSpace, AtomicOrdering, AtomicRMWBinOp, FloatPredicate, IntPredicate};

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
    let int_value = context.i8_type().const_zero();
    let expect_phi_name = "phi_node";
    let phi = builder.build_phi(bool_type, expect_phi_name);
    let instruction = phi.as_instruction();

    let phi_from_instruction: PhiValue = instruction.try_into().unwrap();
    let name = phi_from_instruction.get_name().to_str().unwrap();
    assert_eq!(name, expect_phi_name);

    // test that conversion fails
    let ret_instruction = builder.build_return(None);
    let phi_from_instruction: Result<PhiValue, _> = ret_instruction.try_into();
    assert!(phi_from_instruction.is_err());
}
