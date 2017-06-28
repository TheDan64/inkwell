use llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction, LLVMVerifyFunction};
use llvm_sys::core::{LLVMContextCreate, LLVMCreateBuilderInContext, LLVMModuleCreateWithNameInContext, LLVMContextDispose, LLVMDisposeBuilder, LLVMVoidTypeInContext, LLVMDumpModule, LLVMInt1TypeInContext, LLVMInt8TypeInContext, LLVMInt16TypeInContext, LLVMInt32Type, LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMBuildRet, LLVMBuildRetVoid, LLVMPositionBuilderAtEnd, LLVMBuildCall, LLVMBuildStore, LLVMPointerType, LLVMStructTypeInContext, LLVMAddFunction, LLVMFunctionType, LLVMSetValueName, LLVMGetValueName, LLVMCreatePassManager, LLVMBuildExtractValue, LLVMAppendBasicBlockInContext, LLVMBuildLoad, LLVMBuildGEP, LLVMBuildCondBr, LLVMBuildICmp, LLVMBuildCast, LLVMGetNamedFunction, LLVMBuildAdd, LLVMBuildSub, LLVMBuildMul, LLVMConstInt, LLVMGetFirstParam, LLVMGetNextParam, LLVMCountParams, LLVMDisposePassManager, LLVMCreateFunctionPassManagerForModule, LLVMInitializeFunctionPassManager, LLVMDisposeMessage, LLVMArrayType, LLVMGetReturnType, LLVMTypeOf, LLVMGetElementType, LLVMBuildNeg, LLVMBuildNot, LLVMGetNextBasicBlock, LLVMGetFirstBasicBlock, LLVMGetLastBasicBlock, LLVMGetInsertBlock, LLVMGetBasicBlockParent, LLVMConstReal, LLVMConstArray, LLVMBuildBr, LLVMBuildPhi, LLVMAddIncoming, LLVMBuildAlloca, LLVMBuildMalloc, LLVMBuildArrayMalloc, LLVMBuildArrayAlloca, LLVMGetUndef, LLVMSetDataLayout, LLVMGetBasicBlockTerminator, LLVMInsertIntoBuilder, LLVMIsABasicBlock, LLVMIsAFunction, LLVMIsFunctionVarArg, LLVMDumpType, LLVMPrintValueToString, LLVMPrintTypeToString, LLVMInsertBasicBlock, LLVMInsertBasicBlockInContext, LLVMGetParam, LLVMGetTypeKind, LLVMIsConstant, LLVMVoidType, LLVMSetLinkage, LLVMBuildInsertValue, LLVMIsNull, LLVMBuildIsNull, LLVMIsAConstantArray, LLVMIsAConstantDataArray, LLVMBuildPointerCast, LLVMSetGlobalConstant, LLVMSetInitializer, LLVMAddGlobal, LLVMFloatTypeInContext, LLVMDoubleTypeInContext, LLVMStructGetTypeAtIndex, LLVMMoveBasicBlockAfter, LLVMMoveBasicBlockBefore, LLVMGetTypeByName, LLVMBuildFree, LLVMGetParamTypes, LLVMGetBasicBlocks, LLVMIsUndef, LLVMBuildAnd, LLVMBuildOr, LLVMBuildSDiv, LLVMBuildUDiv, LLVMBuildFAdd, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildXor, LLVMBuildFCmp, LLVMBuildFNeg, LLVMBuildFSub, LLVMBuildUnreachable, LLVMBuildFence, LLVMGetPointerAddressSpace, LLVMIsAConstantPointerNull, LLVMCountParamTypes, LLVMFP128TypeInContext, LLVMIntTypeInContext, LLVMBuildIsNotNull, LLVMConstNamedStruct, LLVMStructCreateNamed, LLVMAlignOf, LLVMTypeIsSized, LLVMGetTypeContext, LLVMStructSetBody};
use llvm_sys::execution_engine::{LLVMGetExecutionEngineTargetData, LLVMCreateExecutionEngineForModule, LLVMExecutionEngineRef, LLVMRunFunction, LLVMRunFunctionAsMain, LLVMDisposeExecutionEngine, LLVMLinkInInterpreter, LLVMGetFunctionAddress, LLVMLinkInMCJIT, LLVMAddModule};
use llvm_sys::LLVMLinkage::LLVMCommonLinkage;
use llvm_sys::prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef, LLVMBasicBlockRef, LLVMPassManagerRef};
use llvm_sys::target::{LLVMOpaqueTargetData, LLVMTargetDataRef, LLVM_InitializeNativeTarget, LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeAsmParser, LLVMCopyStringRepOfTargetData, LLVMAddTargetData, LLVM_InitializeNativeDisassembler, LLVMSizeOfTypeInBits};
use llvm_sys::transforms::scalar::{LLVMAddMemCpyOptPass};
use llvm_sys::{LLVMOpcode, LLVMIntPredicate, LLVMTypeKind, LLVMRealPredicate, LLVMAtomicOrdering};

use std::ffi::{CString, CStr};
use std::fmt;
use std::mem::transmute;

use {BasicBlock, FunctionValue, Type, Value};

impl FunctionValue {
    fn new(value: LLVMValueRef) -> FunctionValue {
        // TODO: Debug mode only assertions:
        {
            assert!(!value.is_null());

            unsafe {
                assert!(!LLVMIsAFunction(value).is_null())
            }
        }

        FunctionValue {
            fn_value: value
        }
    }

    pub fn verify(&self, print: bool) {
        let action = if print == true {
            LLVMVerifierFailureAction::LLVMPrintMessageAction
        } else {
            LLVMVerifierFailureAction::LLVMReturnStatusAction
        };

        let code = unsafe {
            LLVMVerifyFunction(self.fn_value, action)
        };

        if code == 1 {
            panic!("LLVMGenError")
        }
    }

    pub fn get_first_param(&self) -> Option<ParamValue> {
        let param = unsafe {
            LLVMGetFirstParam(self.fn_value)
        };

        if param.is_null() {
            return None;
        }

        Some(ParamValue::new(param))
    }

    pub fn get_first_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetFirstBasicBlock(self.fn_value)
        };

        if bb.is_null() {
            return None;
        }

        Some(BasicBlock::new(bb))
    }

    pub fn get_nth_param(&self, nth: u32) -> Option<ParamValue> {
        let count = self.count_params();

        if nth + 1 > count {
            return None;
        }

        let param = unsafe {
            LLVMGetParam(self.fn_value, nth)
        };

        Some(ParamValue::new(param))
    }

    pub fn count_params(&self) -> u32 {
        unsafe {
            LLVMCountParams(self.fn_value)
        }
    }

    /// REVIEW: Untested
    pub fn get_basic_blocks(&self) -> Vec<BasicBlock> {
        let mut blocks = vec![];

        unsafe {
            LLVMGetBasicBlocks(self.fn_value, blocks.as_mut_ptr());

            transmute(blocks)
        }
    }

    pub fn get_return_type(&self) -> Type {
        let type_ = unsafe {
            LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(self.fn_value)))
        };

        Type::new(type_)
    }

    pub fn params(&self) -> ParamValueIter {
        ParamValueIter {
            param_iter_value: self.fn_value,
            start: true,
        }
    }

    pub fn get_last_basic_block(&self) -> BasicBlock {
        let bb = unsafe {
            LLVMGetLastBasicBlock(self.fn_value)
        };

        BasicBlock::new(bb)
    }
}

impl fmt::Debug for FunctionValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.fn_value))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.fn_value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.fn_value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.fn_value) == 1
        };
        let is_null = unsafe {
            LLVMIsNull(self.fn_value) == 1
        };

        write!(f, "FunctionValue {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.fn_value, is_const, is_null, llvm_value, llvm_type)
    }
}

impl AsRef<LLVMValueRef> for FunctionValue {
    fn as_ref(&self) -> &LLVMValueRef {
        &self.fn_value
    }
}

pub struct ParamValue {
    param_value: LLVMValueRef,
}

impl ParamValue {
    fn new(param_value: LLVMValueRef) -> ParamValue {
        assert!(!param_value.is_null());

        ParamValue {
            param_value: param_value
        }
    }

    pub fn set_name(&mut self, name: &str) {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetValueName(self.param_value, c_string.as_ptr())
        }
    }

    pub fn as_value(&self) -> Value {
        // REVIEW: Would like to not have this in favor of function params being smarter about what they accept
        // Also letting another variable have access to inner raw ptr is risky

        Value::new(self.param_value)
    }
}

impl AsRef<LLVMValueRef> for ParamValue {
    fn as_ref(&self) -> &LLVMValueRef {
        &self.param_value
    }
}

impl fmt::Debug for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.param_value))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.param_value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.param_value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.param_value) == 1
        };
        let is_null = unsafe {
            LLVMIsNull(self.param_value) == 1
        };

        write!(f, "FunctionValue {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.param_value, is_const, is_null, llvm_value, llvm_type)
    }
}

pub struct ParamValueIter {
    param_iter_value: LLVMValueRef,
    start: bool,
}

impl Iterator for ParamValueIter {
    type Item = ParamValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == true {
            let first_value = unsafe {
                LLVMGetFirstParam(self.param_iter_value)
            };

            if first_value.is_null() {
                return None;
            }

            self.start = false;

            self.param_iter_value = first_value;

            return Some(ParamValue::new(first_value));
        }

        let next_value = unsafe {
            LLVMGetNextParam(self.param_iter_value)
        };

        if next_value.is_null() {
            return None;
        }

        self.param_iter_value = next_value;

        Some(ParamValue::new(next_value))
    }
}

impl Value {
    fn new(value: LLVMValueRef) -> Value {
        assert!(!value.is_null());

        Value {
            value: value
        }
    }

    pub fn set_global_constant(&self, num: i32) { // REVIEW: Need better name for this arg
        unsafe {
            LLVMSetGlobalConstant(self.value, num)
        }
    }

    fn set_name(&mut self, name: &str) {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetValueName(self.value, c_string.as_ptr());
        }
    }

    pub fn get_name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetValueName(self.value))
        }
    }

    pub fn add_incoming(&self, mut incoming_values: &mut Value, mut incoming_basic_block: &mut BasicBlock, count: u32) { // REVIEW: PhiValue (self) only?
        unsafe {
            LLVMAddIncoming(self.value, &mut incoming_values.value, &mut incoming_basic_block.basic_block, count);
        }
    }

    /// REVIEW: Untested
    pub fn is_undef(&self) -> bool {
        unsafe {
            LLVMIsUndef(self.value) == 1
        }
    }

    pub fn get_type(&self) -> Type {
        let type_ = unsafe {
            LLVMTypeOf(self.value)
        };

        Type::new(type_)
    }

    pub fn get_type_kind(&self) -> LLVMTypeKind {
        self.get_type().get_kind()
    }

    pub fn is_pointer(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMPointerTypeKind => true,
            _ => false,
        }
    }
    pub fn is_int(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMIntegerTypeKind => true,
            _ => false,
        }
    }

    pub fn is_f32(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMFloatTypeKind => true,
            _ => false,
        }
    }

    pub fn is_f64(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMDoubleTypeKind => true,
            _ => false,
        }
    }

    pub fn is_f128(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMFP128TypeKind => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMHalfTypeKind => true,
            LLVMTypeKind::LLVMFloatTypeKind => true,
            LLVMTypeKind::LLVMDoubleTypeKind => true,
            LLVMTypeKind::LLVMX86_FP80TypeKind => true,
            LLVMTypeKind::LLVMFP128TypeKind => true,
            LLVMTypeKind::LLVMPPC_FP128TypeKind => true,
            _ => false,
        }
    }

    pub fn is_struct(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMStructTypeKind => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMArrayTypeKind => true,
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMVoidTypeKind => true,
            _ => false,
        }
    }
}

impl From<u64> for Value {
    fn from(int: u64) -> Value {
        let type_ = unsafe {
            LLVMInt32Type()
        };

        Type::new(type_).const_int(int, false)
    }
}

impl AsRef<LLVMValueRef> for Value {
    fn as_ref(&self) -> &LLVMValueRef {
        &self.value
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.value))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.value) == 1
        };
        let is_null = unsafe {
            LLVMIsNull(self.value) == 1
        };
        let is_const_array = unsafe {
            !LLVMIsAConstantArray(self.value).is_null()
        };
        let is_const_data_array = unsafe {
            !LLVMIsAConstantDataArray(self.value).is_null()
        };

        write!(f, "Value {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_const_array: {:?}\n    is_const_data_array: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.value, is_const, is_const_array, is_const_data_array, is_null, llvm_value, llvm_type)
    }
}

// Case for separate Value structs:
// LLVMValueRef can be a value (ie int)
// LLVMValueRef can be a function
// LLVMValueRef can be a function param
// LLVMValueRef can be a comparison_op
