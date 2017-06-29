use llvm_sys::core::{LLVMGetBasicBlockParent, LLVMGetBasicBlockTerminator, LLVMGetNextBasicBlock, LLVMInsertBasicBlock, LLVMIsABasicBlock, LLVMIsConstant, LLVMMoveBasicBlockAfter, LLVMMoveBasicBlockBefore, LLVMPrintTypeToString, LLVMPrintValueToString, LLVMTypeOf};
use llvm_sys::prelude::{LLVMValueRef, LLVMBasicBlockRef};

use values::{FunctionValue, Value};

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

    pub fn get_terminator(&self) -> Option<Value> {
        let value = unsafe {
            LLVMGetBasicBlockTerminator(self.basic_block)
        };

        if value.is_null() {
            return None;
        }

        Some(Value::new(value))
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
