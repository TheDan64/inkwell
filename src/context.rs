use llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction, LLVMVerifyFunction};
use llvm_sys::core::{LLVMContextCreate, LLVMCreateBuilderInContext, LLVMModuleCreateWithNameInContext, LLVMContextDispose, LLVMDisposeBuilder, LLVMVoidTypeInContext, LLVMDumpModule, LLVMInt1TypeInContext, LLVMInt8TypeInContext, LLVMInt16TypeInContext, LLVMInt32Type, LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMBuildRet, LLVMBuildRetVoid, LLVMPositionBuilderAtEnd, LLVMBuildCall, LLVMBuildStore, LLVMPointerType, LLVMStructTypeInContext, LLVMAddFunction, LLVMFunctionType, LLVMSetValueName, LLVMGetValueName, LLVMCreatePassManager, LLVMBuildExtractValue, LLVMAppendBasicBlockInContext, LLVMBuildLoad, LLVMBuildGEP, LLVMBuildCondBr, LLVMBuildICmp, LLVMBuildCast, LLVMGetNamedFunction, LLVMBuildAdd, LLVMBuildSub, LLVMBuildMul, LLVMConstInt, LLVMGetFirstParam, LLVMGetNextParam, LLVMCountParams, LLVMDisposePassManager, LLVMCreateFunctionPassManagerForModule, LLVMInitializeFunctionPassManager, LLVMDisposeMessage, LLVMArrayType, LLVMGetReturnType, LLVMTypeOf, LLVMGetElementType, LLVMBuildNeg, LLVMBuildNot, LLVMGetNextBasicBlock, LLVMGetFirstBasicBlock, LLVMGetLastBasicBlock, LLVMGetInsertBlock, LLVMGetBasicBlockParent, LLVMConstReal, LLVMConstArray, LLVMBuildBr, LLVMBuildPhi, LLVMAddIncoming, LLVMBuildAlloca, LLVMBuildMalloc, LLVMBuildArrayMalloc, LLVMBuildArrayAlloca, LLVMGetUndef, LLVMSetDataLayout, LLVMGetBasicBlockTerminator, LLVMInsertIntoBuilder, LLVMIsABasicBlock, LLVMIsAFunction, LLVMIsFunctionVarArg, LLVMDumpType, LLVMPrintValueToString, LLVMPrintTypeToString, LLVMInsertBasicBlock, LLVMInsertBasicBlockInContext, LLVMGetParam, LLVMGetTypeKind, LLVMIsConstant, LLVMVoidType, LLVMSetLinkage, LLVMBuildInsertValue, LLVMIsNull, LLVMBuildIsNull, LLVMIsAConstantArray, LLVMIsAConstantDataArray, LLVMBuildPointerCast, LLVMSetGlobalConstant, LLVMSetInitializer, LLVMAddGlobal, LLVMFloatTypeInContext, LLVMDoubleTypeInContext, LLVMStructGetTypeAtIndex, LLVMMoveBasicBlockAfter, LLVMMoveBasicBlockBefore, LLVMGetTypeByName, LLVMBuildFree, LLVMGetParamTypes, LLVMGetBasicBlocks, LLVMIsUndef, LLVMBuildAnd, LLVMBuildOr, LLVMBuildSDiv, LLVMBuildUDiv, LLVMBuildFAdd, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildXor, LLVMBuildFCmp, LLVMBuildFNeg, LLVMBuildFSub, LLVMBuildUnreachable, LLVMBuildFence, LLVMGetPointerAddressSpace, LLVMIsAConstantPointerNull, LLVMCountParamTypes, LLVMFP128TypeInContext, LLVMIntTypeInContext, LLVMBuildIsNotNull, LLVMConstNamedStruct, LLVMStructCreateNamed, LLVMAlignOf, LLVMTypeIsSized, LLVMGetTypeContext, LLVMStructSetBody};
use llvm_sys::execution_engine::{LLVMGetExecutionEngineTargetData, LLVMCreateExecutionEngineForModule, LLVMExecutionEngineRef, LLVMRunFunction, LLVMRunFunctionAsMain, LLVMDisposeExecutionEngine, LLVMLinkInInterpreter, LLVMGetFunctionAddress, LLVMLinkInMCJIT, LLVMAddModule};
use llvm_sys::LLVMLinkage::LLVMCommonLinkage;
use llvm_sys::prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef, LLVMBasicBlockRef, LLVMPassManagerRef};
use llvm_sys::target::{LLVMOpaqueTargetData, LLVMTargetDataRef, LLVM_InitializeNativeTarget, LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeAsmParser, LLVMCopyStringRepOfTargetData, LLVMAddTargetData, LLVM_InitializeNativeDisassembler, LLVMSizeOfTypeInBits};
use llvm_sys::transforms::scalar::{LLVMAddMemCpyOptPass};
use llvm_sys::{LLVMOpcode, LLVMIntPredicate, LLVMTypeKind, LLVMRealPredicate, LLVMAtomicOrdering};

// From Docs: A single context is not thread safe.
// However, different contexts can execute on different threads simultaneously.
pub struct Context {
    context: LLVMContextRef,
}

impl Context {
    pub fn create() -> Self {
        let context = unsafe {
            LLVMContextCreate()
        };

        Context::new(context)
    }

    fn new(context: LLVMContextRef) -> Self {
        assert!(!context.is_null());

        Context {
            context: context
        }
    }

    pub fn create_builder(&self) -> Builder {
        let builder = unsafe {
            LLVMCreateBuilderInContext(self.context)
        };

        Builder::new(builder)
    }

    pub fn create_module(&self, name: &str) -> Module {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let module = unsafe {
            LLVMModuleCreateWithNameInContext(c_string.as_ptr(), self.context)
        };

        Module::new(module)
    }

    pub fn void_type(&self) -> Type {
        let void_type = unsafe {
            LLVMVoidTypeInContext(self.context)
        };

        Type::new(void_type)
    }

    pub fn bool_type(&self) -> Type {
        let bool_type = unsafe {
            LLVMInt1TypeInContext(self.context)
        };

        Type::new(bool_type)
    }

    pub fn i8_type(&self) -> Type {
        let i8_type = unsafe {
            LLVMInt8TypeInContext(self.context)
        };

        Type::new(i8_type)
    }

    pub fn i16_type(&self) -> Type {
        let i16_type = unsafe {
            LLVMInt16TypeInContext(self.context)
        };

        Type::new(i16_type)
    }

    pub fn i32_type(&self) -> Type {
        let i32_type = unsafe {
            LLVMInt32TypeInContext(self.context)
        };

        Type::new(i32_type)
    }

    pub fn i64_type(&self) -> Type {
        let i64_type = unsafe {
            LLVMInt64TypeInContext(self.context)
        };

        Type::new(i64_type)
    }

    pub fn i128_type(&self) -> Type {
        // REVIEW: The docs says there's a LLVMInt128TypeInContext, but
        // it might only be in a newer version

        let i128_type = unsafe {
            LLVMIntTypeInContext(self.context, 128)
        };

        Type::new(i128_type)
    }

    pub fn custom_width_int_type(&self, bits: u32) -> Type {
        let int_type = unsafe {
            LLVMIntTypeInContext(self.context, bits)
        };

        Type::new(int_type)
    }

    pub fn f32_type(&self) -> Type {
        let f32_type = unsafe {
            LLVMFloatTypeInContext(self.context)
        };

        Type::new(f32_type)
    }

    pub fn f64_type(&self) -> Type {
        let f64_type = unsafe {
            LLVMDoubleTypeInContext(self.context)
        };

        Type::new(f64_type)
    }

    pub fn f128_type(&self) -> Type {
        let f128_type = unsafe {
            LLVMFP128TypeInContext(self.context)
        };

        Type::new(f128_type)
    }

    pub fn struct_type(&self, field_types: Vec<Type>, packed: bool, name: &str) -> Type { // REVIEW: StructType?
        // WARNING: transmute will no longer work correctly if Type gains more fields
        // We're avoiding reallocation by telling rust Vec<Type> is identical to Vec<LLVMTypeRef>
        let mut field_types: Vec<LLVMTypeRef> = unsafe {
            transmute(field_types)
        };

        let struct_type = if name.len() == 0 {
            unsafe {
                LLVMStructTypeInContext(self.context, field_types.as_mut_ptr(), field_types.len() as u32, packed as i32)
            }
        } else {
            let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

            unsafe {
                let struct_type = LLVMStructCreateNamed(self.context, c_string.as_ptr());

                LLVMStructSetBody(struct_type, field_types.as_mut_ptr(), field_types.len() as u32, packed as i32);

                struct_type
            }
        };

        Type::new(struct_type)
    }

    pub fn append_basic_block(&self, function: &FunctionValue, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMAppendBasicBlockInContext(self.context, function.fn_value, c_string.as_ptr())
        };

        BasicBlock::new(bb)
    }

    pub fn insert_basic_block_after(&self, basic_block: &BasicBlock, name: &str) -> BasicBlock {
        match basic_block.get_next_basic_block() {
            Some(next_basic_block) => self.prepend_basic_block(&next_basic_block, name),
            None => {
                let parent_fn = basic_block.get_parent();

                self.append_basic_block(&parent_fn, name)
            },
        }
    }

    pub fn prepend_basic_block(&self, basic_block: &BasicBlock, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMInsertBasicBlockInContext(self.context, basic_block.basic_block, c_string.as_ptr())
        };

        BasicBlock::new(bb)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.context);
        }
    }
}
