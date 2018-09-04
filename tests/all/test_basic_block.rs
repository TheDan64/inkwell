extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::values::InstructionOpcode;

use std::ffi::CString;

#[test]
fn test_basic_block_ordering() {
    let context = Context::create();
    let module = context.create_module("test");
    let builder = context.create_builder();

    assert!(builder.get_insert_block().is_none());

    let i128_type = context.i128_type();
    let fn_type = i128_type.fn_type(&[], false);

    let function = module.add_function("testing", fn_type, None);

    // REVIEW: Possibly LLVM bug - gives a basic block ptr that isn't
    // actually a basic block instead of returning nullptr. Simplest solution
    // may be to just return None if LLVMIsABB doesn't pass
    // assert!(function.get_entry_basic_block().is_none());
    // assert!(function.get_first_basic_block().is_none());

    let basic_block = context.append_basic_block(&function, "entry");
    let basic_block4 = context.insert_basic_block_after(&basic_block, "block4");
    let basic_block2 = context.insert_basic_block_after(&basic_block, "block2");
    let basic_block3 = context.prepend_basic_block(&basic_block4, "block3");

    let basic_blocks = function.get_basic_blocks();

    assert_eq!(basic_blocks.len(), 4);
    assert_eq!(basic_blocks[0], basic_block);
    assert_eq!(basic_blocks[1], basic_block2);
    assert_eq!(basic_blocks[2], basic_block3);
    assert_eq!(basic_blocks[3], basic_block4);

    basic_block3.move_before(&basic_block2);
    basic_block.move_after(&basic_block4);

    let basic_block5 = basic_block.prepend_basic_block("block5");
    let basic_blocks = function.get_basic_blocks();

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

    function.append_basic_block("block6");

    let bb1 = function.get_entry_basic_block().unwrap();
    let bb4 = basic_block5.get_previous_basic_block().unwrap();

    assert_eq!(bb1, basic_block3);
    assert_eq!(bb4, basic_block4);
    assert!(basic_block3.get_previous_basic_block().is_none());

    unsafe {
        bb4.delete();
    }

    let bb2 = basic_block5.get_previous_basic_block().unwrap();

    assert_eq!(bb2, basic_block2);

    let bb3 = function.get_first_basic_block().unwrap();

    assert_eq!(bb3, basic_block3);

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
    {
        assert_eq!(*basic_block.get_name(), *CString::new("entry").unwrap());
        assert_eq!(*basic_block2.get_name(), *CString::new("block2").unwrap());
        assert_eq!(*basic_block3.get_name(), *CString::new("block3").unwrap());
        assert_eq!(*basic_block5.get_name(), *CString::new("block5").unwrap());
    }
}

#[test]
fn test_get_basic_blocks() {
    let context = Context::create();
    let module = context.create_module("test");

    let bool_type = context.bool_type();
    let fn_type = bool_type.fn_type(&[], false);

    let function = module.add_function("testing", fn_type, None);

    assert_eq!(function.get_name(), &*CString::new("testing").unwrap());
    assert_eq!(function.get_return_type().into_int_type().get_bit_width(), 1);

    assert!(function.get_last_basic_block().is_none());
    assert_eq!(function.get_basic_blocks().len(), 0);

    let basic_block = context.append_basic_block(&function, "entry");

    let last_basic_block = function.get_last_basic_block()
                                   .expect("Did not find expected basic block");

    assert_eq!(last_basic_block, basic_block);

    let basic_blocks = function.get_basic_blocks();

    assert_eq!(basic_blocks.len(), 1);
    assert_eq!(basic_blocks[0], basic_block);
}

#[test]
fn test_get_terminator() {
    let context = Context::create();
    let module = context.create_module("test");
    let builder = context.create_builder();

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("testing", fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    builder.position_at_end(&basic_block);

    // REVIEW: What's the difference between a terminator and last instruction?
    assert!(basic_block.get_terminator().is_none());
    assert!(basic_block.get_first_instruction().is_none());
    assert!(basic_block.get_last_instruction().is_none());

    builder.build_return(None);

    assert_eq!(basic_block.get_terminator().unwrap().get_opcode(), InstructionOpcode::Return);
    assert_eq!(basic_block.get_first_instruction().unwrap().get_opcode(), InstructionOpcode::Return);
    assert_eq!(basic_block.get_last_instruction().unwrap().get_opcode(), InstructionOpcode::Return);
    assert_eq!(basic_block.get_last_instruction(), basic_block.get_terminator());
}

#[test]
fn test_no_parent() {
    let context = Context::create();
    let module = context.create_module("test");

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("testing", fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");
    let basic_block2 = context.append_basic_block(&function, "next");

    assert_eq!(basic_block.get_parent().unwrap(), function);
    assert_eq!(basic_block.get_next_basic_block().unwrap(), basic_block2);

    // TODO: Test if this method is unsafe if parent function was hard deleted
    basic_block.remove_from_function();

    assert!(basic_block.get_next_basic_block().is_none());
    assert!(basic_block2.get_previous_basic_block().is_none());

    // The problem here is that calling the function more than once becomes UB
    // for some reason so we have to manually check for the parent function (in the call)
    // until we have SubTypes to solve this at compile time by doing something like:
    // impl BasicBlock<HasParent> { fn remove_from_function(self) -> BasicBlock<Orphan> }
    // though having to take ownership does raise some flags for when you just want to make
    // everything borrowable. I'm not sure it's possible to swap in place since they have
    // different subtypes
    basic_block.remove_from_function();
    basic_block.remove_from_function();

    assert!(basic_block.get_parent().is_none());
}
