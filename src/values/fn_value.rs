use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMViewFunctionCFG, LLVMViewFunctionCFGOnly};
use llvm_sys::core::{LLVMIsAFunction, LLVMIsConstant, LLVMGetLinkage, LLVMTypeOf, LLVMGetPreviousFunction, LLVMGetNextFunction, LLVMGetParam, LLVMCountParams, LLVMGetLastParam, LLVMCountBasicBlocks, LLVMGetFirstParam, LLVMGetNextParam, LLVMGetBasicBlocks, LLVMGetReturnType, LLVMAppendBasicBlock, LLVMDeleteFunction, LLVMGetElementType, LLVMGetLastBasicBlock, LLVMGetFirstBasicBlock, LLVMGetEntryBasicBlock, LLVMGetIntrinsicID, LLVMGetFunctionCallConv, LLVMSetFunctionCallConv, LLVMGetGC, LLVMSetGC};
#[cfg(not(feature = "llvm3-6"))]
use llvm_sys::core::{LLVMGetPersonalityFn, LLVMSetPersonalityFn};
use llvm_sys::prelude::{LLVMValueRef, LLVMBasicBlockRef};

use std::ffi::{CStr, CString};
use std::mem::forget;
use std::fmt;

use basic_block::BasicBlock;
use module::Linkage;
use types::{BasicTypeEnum, FunctionType};
use values::traits::AsValueRef;
use values::{BasicValueEnum, Value, MetadataValue};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FunctionValue {
    fn_value: Value,
}

impl FunctionValue {
    pub(crate) fn new(value: LLVMValueRef) -> Option<Self> {
        if value.is_null() {
            return None;
        }

        unsafe {
            assert!(!LLVMIsAFunction(value).is_null())
        }

        Some(FunctionValue { fn_value: Value::new(value) })
    }

    pub fn get_linkage(&self) -> Linkage {
        let linkage = unsafe {
            LLVMGetLinkage(self.as_value_ref())
        };

        Linkage::new(linkage)
    }

    pub fn is_null(&self) -> bool {
        self.fn_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.fn_value.is_undef()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.fn_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.fn_value.print_to_stderr()
    }

    // TODO: Maybe support LLVMAbortProcessAction?
    // FIXME: Better error returns, code 1 is error
    pub fn verify(&self, print: bool) -> bool {
        let action = if print {
            LLVMVerifierFailureAction::LLVMPrintMessageAction
        } else {
            LLVMVerifierFailureAction::LLVMReturnStatusAction
        };

        let code = unsafe {
            LLVMVerifyFunction(self.fn_value.value, action)
        };

        code != 1
    }

    // REVIEW: If there's a demand, could easily create a module.get_functions() -> Iterator
    pub fn get_next_function(&self) -> Option<Self> {
        let function = unsafe {
            LLVMGetNextFunction(self.as_value_ref())
        };

        FunctionValue::new(function)
    }

    pub fn get_previous_function(&self) -> Option<Self> {
        let function = unsafe {
            LLVMGetPreviousFunction(self.as_value_ref())
        };

        FunctionValue::new(function)
    }

    pub fn get_first_param(&self) -> Option<BasicValueEnum> {
        let param = unsafe {
            LLVMGetFirstParam(self.as_value_ref())
        };

        if param.is_null() {
            return None;
        }

        Some(BasicValueEnum::new(param))
    }

    pub fn get_last_param(&self) -> Option<BasicValueEnum> {
        let param = unsafe {
            LLVMGetLastParam(self.as_value_ref())
        };

        if param.is_null() {
            return None;
        }

        Some(BasicValueEnum::new(param))
    }

    // REVIEW: Odd behavior, a function with no BBs returns a ptr
    // that isn't actually a basic_block and seems to get corrupted
    // Should check filed LLVM bugs - maybe just return None since
    // we can catch it with "LLVMIsABasicBlock"
    pub fn get_entry_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetEntryBasicBlock(self.as_value_ref())
        };

        BasicBlock::new(bb)
    }

    // REVIEW: Odd behavior, a function with no BBs returns a ptr
    // that isn't actually a basic_block and seems to get corrupted
    // Should check filed LLVM bugs - maybe just return None since
    // we can catch it with "LLVMIsABasicBlock"
    pub fn get_first_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetFirstBasicBlock(self.as_value_ref())
        };

        BasicBlock::new(bb)
    }

    // TODOC: This applies the global context to the basic block. To keep the existing context
    // prefer context.append_basic_block()
    pub fn append_basic_block(&self, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMAppendBasicBlock(self.as_value_ref(), c_string.as_ptr())
        };

        BasicBlock::new(bb).expect("Appending basic block should never fail")
    }

    pub fn get_nth_param(&self, nth: u32) -> Option<BasicValueEnum> {
        let count = self.count_params();

        if nth + 1 > count {
            return None;
        }

        let param = unsafe {
            LLVMGetParam(self.as_value_ref(), nth)
        };

        Some(BasicValueEnum::new(param))
    }

    pub fn count_params(&self) -> u32 {
        unsafe {
            LLVMCountParams(self.fn_value.value)
        }
    }

    pub fn count_basic_blocks(&self) -> u32 {
        unsafe {
            LLVMCountBasicBlocks(self.as_value_ref())
        }
    }

    pub fn get_basic_blocks(&self) -> Vec<BasicBlock> {
        let count = self.count_basic_blocks();
        let mut raw_vec: Vec<LLVMBasicBlockRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetBasicBlocks(self.as_value_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| BasicBlock::new(*val).unwrap()).collect()
    }

    pub fn get_return_type(&self) -> BasicTypeEnum {
        let type_ = unsafe {
            LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(self.fn_value.value)))
        };

        BasicTypeEnum::new(type_)
    }

    pub fn params(&self) -> ParamValueIter {
        ParamValueIter {
            param_iter_value: self.fn_value.value,
            start: true,
        }
    }

    pub fn get_last_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetLastBasicBlock(self.fn_value.value)
        };

        BasicBlock::new(bb)
    }

    pub fn get_name(&self) -> &CStr {
        self.fn_value.get_name()
    }

    pub fn view_function_config(&self) {
        unsafe {
            LLVMViewFunctionCFG(self.as_value_ref())
        }
    }

    pub fn view_function_config_only(&self) {
        unsafe {
            LLVMViewFunctionCFGOnly(self.as_value_ref())
        }
    }

    // FIXME: Look for ways to prevent use after delete
    pub unsafe fn delete(self) {
        // unsafe {
        LLVMDeleteFunction(self.as_value_ref())
        // }
    }

    pub fn get_type(&self) -> FunctionType {
        FunctionType::new(self.fn_value.get_type())
    }

    pub fn has_metadata(&self) -> bool {
        self.fn_value.has_metadata()
    }

    pub fn get_metadata(&self, kind_id: u32) -> Option<MetadataValue> {
        self.fn_value.get_metadata(kind_id)
    }

    pub fn set_metadata(&self, metadata: &MetadataValue, kind_id: u32) {
        self.fn_value.set_metadata(metadata, kind_id)
    }

    // TODOC: How this works as an exception handler
    // TODO: LLVM 3.9+
    // pub fn has_personality_function(&self) -> bool {
    //     unsafe {
    //         LLVMHasPersonalityFunction(self.as_value_ref())
    //     }
    // }

    #[cfg(not(feature = "llvm3-6"))]
    pub fn get_personality_function(&self) -> Option<FunctionValue> {
        let value = unsafe {
            LLVMGetPersonalityFn(self.as_value_ref())
        };

        FunctionValue::new(value)
    }

    #[cfg(not(feature = "llvm3-6"))]
    pub fn set_personality_function(&self, personality_fn: &FunctionValue) {
        unsafe {
            LLVMSetPersonalityFn(self.as_value_ref(), personality_fn.as_value_ref())
        }
    }

    pub fn get_intrinsic_id(&self) -> u32 {
        unsafe {
            LLVMGetIntrinsicID(self.as_value_ref())
        }
    }

    pub fn get_call_conventions(&self) -> u32 {
        unsafe {
            LLVMGetFunctionCallConv(self.as_value_ref())
        }
    }

    pub fn set_call_conventions(&self, call_conventions: u32) {
        unsafe {
            LLVMSetFunctionCallConv(self.as_value_ref(), call_conventions)
        }
    }

    pub fn get_gc(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetGC(self.as_value_ref()))
        }
    }

    pub fn set_gc(&self, gc: &str) {
        let c_string = CString::new(gc).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetGC(self.as_value_ref(), c_string.as_ptr())
        }
    }

    pub fn replace_all_uses_with(&self, other: &FunctionValue) {
        self.fn_value.replace_all_uses_with(other.as_value_ref())
    }
}

impl AsValueRef for FunctionValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.fn_value.value
    }
}

impl fmt::Debug for FunctionValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = self.print_to_string();
        let llvm_type = self.get_type();
        let name = self.get_name();
        let is_const = unsafe {
            LLVMIsConstant(self.fn_value.value) == 1
        };
        let is_null = self.is_null();

        write!(f, "FunctionValue {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.as_value_ref(), is_const, is_null, llvm_value, llvm_type.print_to_string())
    }
}

pub struct ParamValueIter {
    param_iter_value: LLVMValueRef,
    start: bool,
}

impl Iterator for ParamValueIter {
    type Item = BasicValueEnum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start {
            let first_value = unsafe {
                LLVMGetFirstParam(self.param_iter_value)
            };

            if first_value.is_null() {
                return None;
            }

            self.start = false;

            self.param_iter_value = first_value;

            return Some(BasicValueEnum::new(first_value));
        }

        let next_value = unsafe {
            LLVMGetNextParam(self.param_iter_value)
        };

        if next_value.is_null() {
            return None;
        }

        self.param_iter_value = next_value;

        Some(BasicValueEnum::new(next_value))
    }
}
