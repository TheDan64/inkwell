extern crate inkwell;

use self::inkwell::AddressSpace;
use self::inkwell::context::Context;
use self::inkwell::values::{BasicValue, InstructionOpcode};

#[test]
fn test_operands() {
    let context = Context::create();
    let module = context.create_module("ivs");
    let builder = context.create_builder();
    let void_type = context.void_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);

    let function = module.add_function("take_f32_ptr", fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let arg1 = function.get_first_param().unwrap().into_pointer_value();
    let f32_val = f32_type.const_float(::std::f64::consts::PI);
    let store_instruction = builder.build_store(arg1, f32_val);
    let free_instruction = builder.build_free(arg1);
    let return_instruction = builder.build_return(None);

    assert!(arg1.as_instruction_value().is_none());

    // Test operands
    assert_eq!(store_instruction.get_num_operands(), 2);
    assert_eq!(free_instruction.get_num_operands(), 2);

    let store_operand0 = store_instruction.get_operand(0).unwrap();
    let store_operand1 = store_instruction.get_operand(1).unwrap();

    assert_eq!(store_operand0, f32_val); // f32 const
    assert_eq!(store_operand1, arg1); // f32* arg1
    assert!(store_instruction.get_operand(2).is_none());
    assert!(store_instruction.get_operand(3).is_none());
    assert!(store_instruction.get_operand(4).is_none());

    let free_operand0 = free_instruction.get_operand(0).unwrap();
    let free_operand1 = free_instruction.get_operand(1).unwrap();
    let free_operand0_instruction = free_operand0.as_instruction_value().unwrap();

    assert!(free_operand0.is_pointer_value()); // (implictly casted) i8* arg1
    assert!(free_operand1.is_pointer_value()); // Free function ptr
    assert_eq!(free_operand0_instruction.get_opcode(), InstructionOpcode::BitCast);
    assert_eq!(free_operand0_instruction.get_operand(0).unwrap(), arg1);
    // assert_eq!(free_operand0_instruction.get_operand(1).unwrap(), arg1);
    assert!(free_instruction.get_operand(2).is_none());
    assert!(free_instruction.get_operand(3).is_none());
    assert!(free_instruction.get_operand(4).is_none());

    assert!(module.verify().is_ok());

    free_instruction.set_operand(0, arg1);

    // Module is no longer valid because free takes an i8* not f32*
    assert!(module.verify().is_err());

    free_instruction.set_operand(0, free_operand0);

    assert!(module.verify().is_ok());

    // No-op, free only has two operands
    free_instruction.set_operand(2, free_operand0);

    assert!(module.verify().is_ok());

    assert_eq!(return_instruction.get_num_operands(), 0);
    assert!(return_instruction.get_operand(0).is_none());
    assert!(return_instruction.get_operand(1).is_none());
    assert!(return_instruction.get_operand(2).is_none());

    // Test Uses
    // These instructions/calls don't return any ir value so they aren't used anywhere
    // TODO: Test on instruction that is used
    assert!(store_instruction.get_first_use().is_none());
    assert!(free_instruction.get_first_use().is_none());
    assert!(return_instruction.get_first_use().is_none());

    // However their operands are used
    let store_operand_use0 = store_instruction.get_operand_use(0).unwrap();
    let store_operand_use1 = store_instruction.get_operand_use(1).unwrap();

    // in "store float 0x400921FB60000000, float* %0"
    // The const float is only used once, so it has no subsequent use
    // However the 2nd operand %0 is used in the subsequent (implicit) bitcast
    // TODO: Test with successful next use
    assert!(store_operand_use0.get_next_use().is_none());
    assert!(store_operand_use1.get_next_use().is_none()); // REVIEW: Why is this none?

    assert_eq!(store_operand_use0.get_user(), store_instruction);
    assert_eq!(store_operand_use1.get_user(), store_instruction);
    assert_eq!(store_operand_use0.get_used_value(), f32_val);
    assert_eq!(store_operand_use1.get_used_value(), arg1);

    assert!(store_instruction.get_operand_use(2).is_none());
    assert!(store_instruction.get_operand_use(3).is_none());
    assert!(store_instruction.get_operand_use(4).is_none());
    assert!(store_instruction.get_operand_use(5).is_none());
    assert!(store_instruction.get_operand_use(6).is_none());

    let free_operand_use0 = free_instruction.get_operand_use(0).unwrap();
    let free_operand_use1 = free_instruction.get_operand_use(1).unwrap();

    assert!(free_operand_use0.get_next_use().is_none());
    assert!(free_operand_use1.get_next_use().is_none());






    assert!(free_instruction.get_operand_use(2).is_none());
    assert!(free_instruction.get_operand_use(3).is_none());
    assert!(free_instruction.get_operand_use(4).is_none());
    assert!(free_instruction.get_operand_use(5).is_none());
    assert!(free_instruction.get_operand_use(6).is_none());


    assert!(false, "\n{}", module.print_to_string().to_string());
}
