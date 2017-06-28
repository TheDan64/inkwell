use llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction, LLVMVerifyFunction};
use llvm_sys::core::{LLVMContextCreate, LLVMCreateBuilderInContext, LLVMModuleCreateWithNameInContext, LLVMContextDispose, LLVMDisposeBuilder, LLVMVoidTypeInContext, LLVMDumpModule, LLVMInt1TypeInContext, LLVMInt8TypeInContext, LLVMInt16TypeInContext, LLVMInt32Type, LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMBuildRet, LLVMBuildRetVoid, LLVMPositionBuilderAtEnd, LLVMBuildCall, LLVMBuildStore, LLVMPointerType, LLVMStructTypeInContext, LLVMAddFunction, LLVMFunctionType, LLVMSetValueName, LLVMGetValueName, LLVMCreatePassManager, LLVMBuildExtractValue, LLVMAppendBasicBlockInContext, LLVMBuildLoad, LLVMBuildGEP, LLVMBuildCondBr, LLVMBuildICmp, LLVMBuildCast, LLVMGetNamedFunction, LLVMBuildAdd, LLVMBuildSub, LLVMBuildMul, LLVMConstInt, LLVMGetFirstParam, LLVMGetNextParam, LLVMCountParams, LLVMDisposePassManager, LLVMCreateFunctionPassManagerForModule, LLVMInitializeFunctionPassManager, LLVMDisposeMessage, LLVMArrayType, LLVMGetReturnType, LLVMTypeOf, LLVMGetElementType, LLVMBuildNeg, LLVMBuildNot, LLVMGetNextBasicBlock, LLVMGetFirstBasicBlock, LLVMGetLastBasicBlock, LLVMGetInsertBlock, LLVMGetBasicBlockParent, LLVMConstReal, LLVMConstArray, LLVMBuildBr, LLVMBuildPhi, LLVMAddIncoming, LLVMBuildAlloca, LLVMBuildMalloc, LLVMBuildArrayMalloc, LLVMBuildArrayAlloca, LLVMGetUndef, LLVMSetDataLayout, LLVMGetBasicBlockTerminator, LLVMInsertIntoBuilder, LLVMIsABasicBlock, LLVMIsAFunction, LLVMIsFunctionVarArg, LLVMDumpType, LLVMPrintValueToString, LLVMPrintTypeToString, LLVMInsertBasicBlock, LLVMInsertBasicBlockInContext, LLVMGetParam, LLVMGetTypeKind, LLVMIsConstant, LLVMVoidType, LLVMSetLinkage, LLVMBuildInsertValue, LLVMIsNull, LLVMBuildIsNull, LLVMIsAConstantArray, LLVMIsAConstantDataArray, LLVMBuildPointerCast, LLVMSetGlobalConstant, LLVMSetInitializer, LLVMAddGlobal, LLVMFloatTypeInContext, LLVMDoubleTypeInContext, LLVMStructGetTypeAtIndex, LLVMMoveBasicBlockAfter, LLVMMoveBasicBlockBefore, LLVMGetTypeByName, LLVMBuildFree, LLVMGetParamTypes, LLVMGetBasicBlocks, LLVMIsUndef, LLVMBuildAnd, LLVMBuildOr, LLVMBuildSDiv, LLVMBuildUDiv, LLVMBuildFAdd, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildXor, LLVMBuildFCmp, LLVMBuildFNeg, LLVMBuildFSub, LLVMBuildUnreachable, LLVMBuildFence, LLVMGetPointerAddressSpace, LLVMIsAConstantPointerNull, LLVMCountParamTypes, LLVMFP128TypeInContext, LLVMIntTypeInContext, LLVMBuildIsNotNull, LLVMConstNamedStruct, LLVMStructCreateNamed, LLVMAlignOf, LLVMTypeIsSized, LLVMGetTypeContext, LLVMStructSetBody};
use llvm_sys::execution_engine::{LLVMGetExecutionEngineTargetData, LLVMCreateExecutionEngineForModule, LLVMExecutionEngineRef, LLVMRunFunction, LLVMRunFunctionAsMain, LLVMDisposeExecutionEngine, LLVMLinkInInterpreter, LLVMGetFunctionAddress, LLVMLinkInMCJIT, LLVMAddModule};
use llvm_sys::LLVMLinkage::LLVMCommonLinkage;
use llvm_sys::prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef, LLVMBasicBlockRef, LLVMPassManagerRef};
use llvm_sys::target::{LLVMOpaqueTargetData, LLVMTargetDataRef, LLVM_InitializeNativeTarget, LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeAsmParser, LLVMCopyStringRepOfTargetData, LLVMAddTargetData, LLVM_InitializeNativeDisassembler, LLVMSizeOfTypeInBits};
use llvm_sys::transforms::scalar::{LLVMAddMemCpyOptPass};
use llvm_sys::{LLVMOpcode, LLVMIntPredicate, LLVMTypeKind, LLVMRealPredicate, LLVMAtomicOrdering};

use {PassManager, TargetData};

impl PassManager {
    fn new(pass_manager: LLVMPassManagerRef) -> PassManager {
        assert!(!pass_manager.is_null());

        PassManager {
            pass_manager: pass_manager
        }
    }

    pub fn initialize(&self) -> bool {
        // return true means some pass modified the module, not an error occurred
        unsafe {
            LLVMInitializeFunctionPassManager(self.pass_manager) == 1
        }
    }

    pub fn add_optimize_memcpy_pass(&self) {
        unsafe {
            LLVMAddMemCpyOptPass(self.pass_manager)
        }
    }

    pub fn add_target_data(&self, target_data: TargetData) {
        unsafe {
            LLVMAddTargetData(target_data.target_data, self.pass_manager)
        }
    }
}

impl Drop for PassManager {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposePassManager(self.pass_manager)
        }
    }
}
