use inkwell::context::Context;
use inkwell::values::InstructionOpcode;

#[test]
fn test_basic_block_ordering() {
    let context = Context::create();
    let module = context.create_module("test");
    let builder = context.create_builder();

    assert!(builder.get_insert_block().is_none());

    let i128_type = context.i128_type();
    let fn_type = i128_type.fn_type(&[], false);

    let function = module.add_function("testing", fn_type, None);

    assert!(function.get_first_basic_block().is_none());

    let basic_block = context.append_basic_block(function, "entry");
    let basic_block4 = context.insert_basic_block_after(basic_block, "block4");
    let basic_block2 = context.insert_basic_block_after(basic_block, "block2");
    let basic_block3 = context.prepend_basic_block(basic_block4, "block3");

    for basic_blocks in [function.get_basic_blocks(), function.get_basic_block_iter().collect()] {
        assert_eq!(basic_blocks.len(), 4);
        assert_eq!(basic_blocks[0], basic_block);
        assert_eq!(basic_blocks[1], basic_block2);
        assert_eq!(basic_blocks[2], basic_block3);
        assert_eq!(basic_blocks[3], basic_block4);
    }

    assert!(basic_block3.move_before(basic_block2).is_ok());
    assert!(basic_block.move_after(basic_block4).is_ok());

    let basic_block5 = context.prepend_basic_block(basic_block, "block5");

    for basic_blocks in [function.get_basic_blocks(), function.get_basic_block_iter().collect()] {
        assert_eq!(basic_blocks.len(), 5);
        assert_eq!(basic_blocks[0], basic_block3);
        assert_eq!(basic_blocks[1], basic_block2);
        assert_eq!(basic_blocks[2], basic_block4);
        assert_eq!(basic_blocks[3], basic_block5);
        assert_eq!(basic_blocks[4], basic_block);

        assert_ne!(basic_blocks[0], basic_block);
        assert_ne!(basic_blocks[1], basic_block3);
        assert_ne!(basic_blocks[2], basic_block2);
        assert_ne!(basic_blocks[3], basic_block4);
        assert_ne!(basic_blocks[4], basic_block5);
    }

    context.append_basic_block(function, "block6");

    let bb1 = function.get_first_basic_block().unwrap();
    let bb4 = basic_block5.get_previous_basic_block().unwrap();

    assert_eq!(bb1, basic_block3);
    assert_eq!(bb4, basic_block4);
    assert!(basic_block3.get_previous_basic_block().is_none());

    unsafe {
        assert!(bb4.delete().is_ok());
    }

    let bb2 = basic_block5.get_previous_basic_block().unwrap();

    assert_eq!(bb2, basic_block2);

    let bb3 = function.get_first_basic_block().unwrap();

    assert_eq!(bb3, basic_block3);
    assert_eq!(basic_block.get_name().to_str(), Ok("entry"));
    assert_eq!(basic_block2.get_name().to_str(), Ok("block2"));
    assert_eq!(basic_block3.get_name().to_str(), Ok("block3"));
    assert_eq!(basic_block5.get_name().to_str(), Ok("block5"));
}

#[test]
fn test_get_basic_blocks() {
    let context = Context::create();
    let module = context.create_module("test");

    let bool_type = context.bool_type();
    let fn_type = bool_type.fn_type(&[], false);

    let function = module.add_function("testing", fn_type, None);

    assert_eq!(function.get_name().to_str(), Ok("testing"));
    assert_eq!(fn_type.get_return_type().unwrap().into_int_type().get_bit_width(), 1);

    assert!(function.get_last_basic_block().is_none());
    assert_eq!(function.get_basic_blocks().len(), 0);
    assert_eq!(function.get_basic_block_iter().count(), 0);

    let basic_block = context.append_basic_block(function, "entry");

    let last_basic_block = function
        .get_last_basic_block()
        .expect("Did not find expected basic block");

    assert_eq!(last_basic_block, basic_block);

    for basic_blocks in [function.get_basic_blocks(), function.get_basic_block_iter().collect()] {
        assert_eq!(basic_blocks.len(), 1);
        assert_eq!(basic_blocks[0], basic_block);
    }
}

#[test]
fn test_get_terminator() {
    let context = Context::create();
    let module = context.create_module("test");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("testing", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    // REVIEW: What's the difference between a terminator and last instruction?
    assert!(basic_block.get_terminator().is_none());
    assert!(basic_block.get_first_instruction().is_none());
    assert!(basic_block.get_last_instruction().is_none());

    builder.build_return(None).unwrap();

    assert_eq!(
        basic_block.get_terminator().unwrap().get_opcode(),
        InstructionOpcode::Return
    );
    assert_eq!(
        basic_block.get_first_instruction().unwrap().get_opcode(),
        InstructionOpcode::Return
    );
    assert_eq!(
        basic_block.get_last_instruction().unwrap().get_opcode(),
        InstructionOpcode::Return
    );
    assert_eq!(basic_block.get_last_instruction(), basic_block.get_terminator());
}

#[test]
fn test_no_parent() {
    let context = Context::create();
    let module = context.create_module("test");

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("testing", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    let basic_block2 = context.append_basic_block(function, "next");

    assert_eq!(basic_block.get_parent().unwrap(), function);
    assert_eq!(basic_block.get_next_basic_block().unwrap(), basic_block2);

    assert!(basic_block.remove_from_function().is_ok());

    assert!(basic_block.get_next_basic_block().is_none());
    assert!(basic_block2.get_previous_basic_block().is_none());

    assert!(basic_block.remove_from_function().is_err());
    assert!(basic_block.remove_from_function().is_err());

    assert!(basic_block.get_parent().is_none());
}

#[test]
fn test_rauw() {
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_val = module.add_function("my_fn", fn_type, None);
    let entry = context.append_basic_block(fn_val, "entry");
    let bb1 = context.append_basic_block(fn_val, "bb1");
    let bb2 = context.append_basic_block(fn_val, "bb2");
    builder.position_at_end(entry);
    let branch_inst = builder.build_unconditional_branch(bb1).unwrap();

    bb1.replace_all_uses_with(&bb1); // no-op
    bb1.replace_all_uses_with(&bb2);

    assert_eq!(branch_inst.get_operand(0).unwrap().right().unwrap(), bb2);
}

#[test]
fn test_get_first_use() {
    let context = Context::create();
    let module = context.create_module("ivs");
    let builder = context.create_builder();
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_val = module.add_function("my_fn", fn_type, None);
    let entry = context.append_basic_block(fn_val, "entry");
    let bb1 = context.append_basic_block(fn_val, "bb1");
    let bb2 = context.append_basic_block(fn_val, "bb2");
    builder.position_at_end(entry);
    let branch_inst = builder.build_unconditional_branch(bb1).unwrap();

    assert!(bb2.get_first_use().is_none());
    assert!(bb1.get_first_use().is_some());
    assert_eq!(bb1.get_first_use().unwrap().get_user(), branch_inst);
    assert!(bb1.get_first_use().unwrap().get_next_use().is_none());
}

#[test]
fn test_get_address() {
    let context = Context::create();
    let module = context.create_module("my_mod");
    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_val = module.add_function("my_fn", fn_type, None);
    let entry_bb = context.append_basic_block(fn_val, "entry");
    let next_bb = context.append_basic_block(fn_val, "next");

    assert!(unsafe { entry_bb.get_address() }.is_none());
    assert!(unsafe { next_bb.get_address() }.is_some());
}
