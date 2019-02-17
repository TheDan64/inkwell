extern crate inkwell;

use self::inkwell::AddressSpace;
use self::inkwell::context::Context;
use self::inkwell::values::{BasicValue, InstructionOpcode::*};

#[test]
fn test_operands() {
    let context = Context::create();
    let module = context.create_module("ivs");
    let builder = context.create_builder();
    let void_type = context.void_type();
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

    assert_eq!(store_instruction.get_opcode(), Store);
    assert_eq!(free_instruction.get_opcode(), Call);
    assert_eq!(return_instruction.get_opcode(), Return);

    assert!(arg1.as_instruction_value().is_none());

    // Test operands
    assert_eq!(store_instruction.get_num_operands(), 2);
    assert_eq!(free_instruction.get_num_operands(), 2);

    let store_operand0 = store_instruction.get_operand(0).unwrap();
    let store_operand1 = store_instruction.get_operand(1).unwrap();

    assert_eq!(store_operand0.left().unwrap(), f32_val); // f32 const
    assert_eq!(store_operand1.left().unwrap(), arg1); // f32* arg1
    assert!(store_instruction.get_operand(2).is_none());
    assert!(store_instruction.get_operand(3).is_none());
    assert!(store_instruction.get_operand(4).is_none());

    let free_operand0 = free_instruction.get_operand(0).unwrap().left().unwrap();
    let free_operand1 = free_instruction.get_operand(1).unwrap().left().unwrap();
    let free_operand0_instruction = free_operand0.as_instruction_value().unwrap();

    assert!(free_operand0.is_pointer_value()); // (implictly casted) i8* arg1
    assert!(free_operand1.is_pointer_value()); // Free function ptr
    assert_eq!(free_operand0_instruction.get_opcode(), BitCast);
    assert_eq!(free_operand0_instruction.get_operand(0).unwrap().left().unwrap(), arg1);
    assert!(free_operand0_instruction.get_operand(1).is_none());
    assert!(free_operand0_instruction.get_operand(2).is_none());
    assert!(free_instruction.get_operand(2).is_none());
    assert!(free_instruction.get_operand(3).is_none());
    assert!(free_instruction.get_operand(4).is_none());

    assert!(module.verify().is_ok());

    assert!(free_instruction.set_operand(0, arg1));

    // Module is no longer valid because free takes an i8* not f32*
    assert!(module.verify().is_err());

    assert!(free_instruction.set_operand(0, free_operand0));

    assert!(module.verify().is_ok());

    // No-op, free only has two (0-1) operands
    assert!(!free_instruction.set_operand(2, free_operand0));

    assert!(module.verify().is_ok());

    assert_eq!(return_instruction.get_num_operands(), 0);
    assert!(return_instruction.get_operand(0).is_none());
    assert!(return_instruction.get_operand(1).is_none());
    assert!(return_instruction.get_operand(2).is_none());

    // Test Uses
    let bitcast_use_value = free_operand0_instruction
        .get_first_use()
        .unwrap()
        .get_used_value()
        .left()
        .unwrap();
    let free_call_param = free_instruction.get_operand(0).unwrap().left().unwrap();

    assert_eq!(bitcast_use_value, free_call_param);

    // These instructions/calls don't return any ir value so they aren't used anywhere
    assert!(store_instruction.get_first_use().is_none());
    assert!(free_instruction.get_first_use().is_none());
    assert!(return_instruction.get_first_use().is_none());

    // arg1 (%0) has two uses:
    //   store float 0x400921FB60000000, float* %0
    //   %1 = bitcast float* %0 to i8*
    let arg1_first_use = arg1.get_first_use().unwrap();
    let arg1_second_use = arg1_first_use.get_next_use().unwrap();

    // However their operands are used
    let store_operand_use0 = store_instruction.get_operand_use(0).unwrap();
    let store_operand_use1 = store_instruction.get_operand_use(1).unwrap();

    assert!(store_operand_use0.get_next_use().is_none());
    assert!(store_operand_use1.get_next_use().is_none());
    assert_eq!(store_operand_use1, arg1_second_use);

    assert_eq!(store_operand_use0.get_user(), store_instruction);
    assert_eq!(store_operand_use1.get_user(), store_instruction);
    assert_eq!(store_operand_use0.get_used_value().left().unwrap(), f32_val);
    assert_eq!(store_operand_use1.get_used_value().left().unwrap(), arg1);

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

    assert!(module.verify().is_ok());
}

#[test]
fn test_basic_block_operand() {
    let context = Context::create();
    let module = context.create_module("ivs");
    let builder = context.create_builder();
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let function = module.add_function("bb_op", fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");
    let basic_block2 = context.append_basic_block(&function, "exit");

    builder.position_at_end(&basic_block);

    let branch_instruction = builder.build_unconditional_branch(&basic_block2);
    let bb_operand = branch_instruction.get_operand(0).unwrap().right().unwrap();

    assert_eq!(bb_operand, basic_block2);

    let bb_operand_use = branch_instruction.get_operand_use(0).unwrap();

    assert_eq!(bb_operand_use.get_used_value().right().unwrap(), basic_block2);

    builder.position_at_end(&basic_block2);
    builder.build_return(None);

    assert!(module.verify().is_ok());
}

#[test]
fn test_get_next_use() {
    let context = Context::create();
    let module = context.create_module("ivs");
    let builder = context.create_builder();
    let f32_type = context.f32_type();
    let fn_type = f32_type.fn_type(&[f32_type.into()], false);
    let function = module.add_function("take_f32", fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let arg1 = function.get_first_param().unwrap().into_float_value();
    let f32_val = f32_type.const_float(::std::f64::consts::PI);
    let add_pi0 = builder.build_float_add(arg1, f32_val, "add_pi");
    let add_pi1 = builder.build_float_add(add_pi0, f32_val, "add_pi");

    builder.build_return(Some(&add_pi1));

    // f32_val constant appears twice, so there are two uses (first, next)
    let first_use = f32_val.get_first_use().unwrap();

    assert_eq!(first_use.get_user(), add_pi1.as_instruction_value().unwrap());
    assert_eq!(first_use.get_next_use().map(|x| x.get_user()), add_pi0.as_instruction_value());
    assert!(arg1.get_first_use().is_some());
    assert!(module.verify().is_ok());
}


#[test]
fn test_instructions() {
    let context = Context::create();
    let module = context.create_module("testing");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);

    let function = module.add_function("free_f32", fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    let arg1 = function.get_first_param().unwrap().into_pointer_value();

    assert!(arg1.get_first_use().is_none());

    let f32_val = f32_type.const_float(::std::f64::consts::PI);

    let store_instruction = builder.build_store(arg1, f32_val);
    let ptr_val = builder.build_ptr_to_int(arg1, i64_type, "ptr_val");
    let ptr = builder.build_int_to_ptr(ptr_val, f32_ptr_type, "ptr");
    let free_instruction = builder.build_free(arg1);
    let return_instruction = builder.build_return(None);

    assert_eq!(store_instruction.get_opcode(), Store);
    assert_eq!(ptr_val.as_instruction().unwrap().get_opcode(), PtrToInt);
    assert_eq!(ptr.as_instruction().unwrap().get_opcode(), IntToPtr);
    assert_eq!(free_instruction.get_opcode(), Call);
    assert_eq!(return_instruction.get_opcode(), Return);

    // test instruction cloning
    let instruction_clone = return_instruction.clone();

    assert_eq!(instruction_clone.get_opcode(), return_instruction.get_opcode());
    assert_ne!(instruction_clone, return_instruction);

    // test copying
    let instruction_clone_copy = instruction_clone;

    assert_eq!(instruction_clone, instruction_clone_copy);
}
