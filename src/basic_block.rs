use llvm_sys::core::{LLVMGetBasicBlockParent, LLVMGetBasicBlockTerminator, LLVMGetNextBasicBlock, LLVMInsertBasicBlock, LLVMIsABasicBlock, LLVMIsConstant, LLVMMoveBasicBlockAfter, LLVMMoveBasicBlockBefore, LLVMPrintTypeToString, LLVMPrintValueToString, LLVMTypeOf, LLVMDeleteBasicBlock};
use llvm_sys::prelude::{LLVMValueRef, LLVMBasicBlockRef};

use values::{BasicValueEnum, FunctionValue};

use std::fmt;
use std::ffi::{CStr, CString};

pub struct BasicBlock {
    pub(crate) basic_block: LLVMBasicBlockRef,
}

impl BasicBlock {
    pub(crate) fn new(basic_block: LLVMBasicBlockRef) -> BasicBlock {
        assert!(!basic_block.is_null());

        unsafe {
            assert!(!LLVMIsABasicBlock(basic_block as LLVMValueRef).is_null()) // NOTE: There is a LLVMBasicBlockAsValue but it might be the same as casting
        }

        BasicBlock {
            basic_block: basic_block
        }
    }

    pub fn get_parent(&self) -> FunctionValue {
        let value = unsafe {
            LLVMGetBasicBlockParent(self.basic_block)
        };

        FunctionValue::new(value)
    }

    pub fn get_next_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetNextBasicBlock(self.basic_block)
        };

        if bb.is_null() {
            return None;
        }

        Some(BasicBlock::new(bb))
    }

    // REVIEW: What if terminator is an instuction?
    pub fn get_terminator(&self) -> Option<BasicValueEnum> {
        let value = unsafe {
            LLVMGetBasicBlockTerminator(self.basic_block)
        };

        if value.is_null() {
            return None;
        }

        Some(BasicValueEnum::new(value))
    }

    pub fn move_before(&self, basic_block: &BasicBlock) {
        unsafe {
            LLVMMoveBasicBlockBefore(self.basic_block, basic_block.basic_block)
        }
    }

    pub fn move_after(&self, basic_block: &BasicBlock) {
        unsafe {
            LLVMMoveBasicBlockAfter(self.basic_block, basic_block.basic_block)
        }
    }

    pub fn prepend_basic_block(&self, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMInsertBasicBlock(self.basic_block, c_string.as_ptr())
        };

        BasicBlock::new(bb)
    }

    // REVIEW: Could potentially be unsafe if there are existing references. Might need a global ref counter
    // keeping private for now
    fn delete(self) {
        unsafe {
            LLVMDeleteBasicBlock(self.basic_block)
        }
    }
}

impl fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.basic_block as LLVMValueRef))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.basic_block as LLVMValueRef)))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.basic_block as LLVMValueRef) == 1
        };

        write!(f, "BasicBlock {{\n    address: {:?}\n    is_const: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", self.basic_block, is_const, llvm_value, llvm_type)
    }
}

#[test]
fn test_get_basic_blocks() {
    use context::Context;

    let context = Context::create();
    let module = context.create_module("test");

    let bool_type = context.bool_type();
    let fn_type = bool_type.fn_type(&[], false);

    let function = module.add_function("testing", &fn_type);

    assert_eq!(function.get_name(), &*CString::new("testing").unwrap());
    assert_eq!(function.get_return_type().into_int_type().get_bit_width(), 1);

    assert!(function.get_last_basic_block().is_none());
    assert_eq!(function.get_basic_blocks().len(), 0);

    let basic_block = context.append_basic_block(&function, "entry");

    let last_basic_block = function.get_last_basic_block()
                                   .expect("Did not find expected basic block");

    assert_eq!(last_basic_block.basic_block, basic_block.basic_block);

    let basic_blocks = function.get_basic_blocks();

    assert_eq!(basic_blocks.len(), 1);
    assert_eq!(basic_blocks[0].basic_block, basic_block.basic_block);
}

#[test]
fn test_basic_block_ordering() {
    use context::Context;

    let context = Context::create();
    let module = context.create_module("test");

    let i128_type = context.i128_type();
    let fn_type = i128_type.fn_type(&[], false);

    let function = module.add_function("testing", &fn_type);

    let basic_block = context.append_basic_block(&function, "entry");
    let basic_block4 = context.insert_basic_block_after(&basic_block, "block4");
    let basic_block2 = context.insert_basic_block_after(&basic_block, "block2");
    let basic_block3 = context.prepend_basic_block(&basic_block4, "block3");

    let basic_blocks = function.get_basic_blocks();

    assert_eq!(basic_blocks.len(), 4);
    assert_eq!(basic_blocks[0].basic_block, basic_block.basic_block);
    assert_eq!(basic_blocks[1].basic_block, basic_block2.basic_block);
    assert_eq!(basic_blocks[2].basic_block, basic_block3.basic_block);
    assert_eq!(basic_blocks[3].basic_block, basic_block4.basic_block);

    basic_block3.move_before(&basic_block2);
    basic_block.move_after(&basic_block4);

    let basic_block5 = basic_block.prepend_basic_block("block5");
    let basic_blocks = function.get_basic_blocks();

    assert_eq!(basic_blocks.len(), 5);
    assert_eq!(basic_blocks[0].basic_block, basic_block3.basic_block);
    assert_eq!(basic_blocks[1].basic_block, basic_block2.basic_block);
    assert_eq!(basic_blocks[2].basic_block, basic_block4.basic_block);
    assert_eq!(basic_blocks[3].basic_block, basic_block5.basic_block);
    assert_eq!(basic_blocks[4].basic_block, basic_block.basic_block);
}
