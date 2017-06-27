extern crate llvm_sys;

use self::llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction, LLVMVerifyFunction};
use self::llvm_sys::core::{LLVMContextCreate, LLVMCreateBuilderInContext, LLVMModuleCreateWithNameInContext, LLVMContextDispose, LLVMDisposeBuilder, LLVMVoidTypeInContext, LLVMDumpModule, LLVMInt1TypeInContext, LLVMInt8TypeInContext, LLVMInt16TypeInContext, LLVMInt32Type, LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMBuildRet, LLVMBuildRetVoid, LLVMPositionBuilderAtEnd, LLVMBuildCall, LLVMBuildStore, LLVMPointerType, LLVMStructTypeInContext, LLVMAddFunction, LLVMFunctionType, LLVMSetValueName, LLVMGetValueName, LLVMCreatePassManager, LLVMBuildExtractValue, LLVMAppendBasicBlockInContext, LLVMBuildLoad, LLVMBuildGEP, LLVMBuildCondBr, LLVMBuildICmp, LLVMBuildCast, LLVMGetNamedFunction, LLVMBuildAdd, LLVMBuildSub, LLVMBuildMul, LLVMConstInt, LLVMGetFirstParam, LLVMGetNextParam, LLVMCountParams, LLVMDisposePassManager, LLVMCreateFunctionPassManagerForModule, LLVMInitializeFunctionPassManager, LLVMDisposeMessage, LLVMArrayType, LLVMGetReturnType, LLVMTypeOf, LLVMGetElementType, LLVMBuildNeg, LLVMBuildNot, LLVMGetNextBasicBlock, LLVMGetFirstBasicBlock, LLVMGetLastBasicBlock, LLVMGetInsertBlock, LLVMGetBasicBlockParent, LLVMConstReal, LLVMConstArray, LLVMBuildBr, LLVMBuildPhi, LLVMAddIncoming, LLVMBuildAlloca, LLVMBuildMalloc, LLVMBuildArrayMalloc, LLVMBuildArrayAlloca, LLVMGetUndef, LLVMSetDataLayout, LLVMGetBasicBlockTerminator, LLVMInsertIntoBuilder, LLVMIsABasicBlock, LLVMIsAFunction, LLVMIsFunctionVarArg, LLVMDumpType, LLVMPrintValueToString, LLVMPrintTypeToString, LLVMInsertBasicBlock, LLVMInsertBasicBlockInContext, LLVMGetParam, LLVMGetTypeKind, LLVMIsConstant, LLVMVoidType, LLVMSetLinkage, LLVMBuildInsertValue, LLVMIsNull, LLVMBuildIsNull, LLVMIsAConstantArray, LLVMIsAConstantDataArray, LLVMBuildPointerCast, LLVMSetGlobalConstant, LLVMSetInitializer, LLVMAddGlobal, LLVMFloatTypeInContext, LLVMDoubleTypeInContext, LLVMStructGetTypeAtIndex, LLVMMoveBasicBlockAfter, LLVMMoveBasicBlockBefore, LLVMGetTypeByName, LLVMBuildFree, LLVMGetParamTypes, LLVMGetBasicBlocks, LLVMIsUndef, LLVMBuildAnd, LLVMBuildOr, LLVMBuildSDiv, LLVMBuildUDiv, LLVMBuildFAdd, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildXor, LLVMBuildFCmp, LLVMBuildFNeg, LLVMBuildFSub, LLVMBuildUnreachable, LLVMBuildFence, LLVMGetPointerAddressSpace, LLVMIsAConstantPointerNull, LLVMCountParamTypes, LLVMFP128TypeInContext, LLVMIntTypeInContext, LLVMBuildIsNotNull, LLVMConstNamedStruct, LLVMStructCreateNamed, LLVMAlignOf, LLVMTypeIsSized, LLVMGetTypeContext, LLVMStructSetBody};
use self::llvm_sys::execution_engine::{LLVMGetExecutionEngineTargetData, LLVMCreateExecutionEngineForModule, LLVMExecutionEngineRef, LLVMRunFunction, LLVMRunFunctionAsMain, LLVMDisposeExecutionEngine, LLVMLinkInInterpreter, LLVMGetFunctionAddress, LLVMLinkInMCJIT, LLVMAddModule};
use self::llvm_sys::LLVMLinkage::LLVMCommonLinkage;
use self::llvm_sys::prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef, LLVMBasicBlockRef, LLVMPassManagerRef};
use self::llvm_sys::target::{LLVMOpaqueTargetData, LLVMTargetDataRef, LLVM_InitializeNativeTarget, LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeAsmParser, LLVMCopyStringRepOfTargetData, LLVMAddTargetData, LLVM_InitializeNativeDisassembler, LLVMSizeOfTypeInBits};
use self::llvm_sys::transforms::scalar::{LLVMAddMemCpyOptPass};
use self::llvm_sys::{LLVMOpcode, LLVMIntPredicate, LLVMTypeKind, LLVMRealPredicate, LLVMAtomicOrdering};

use std::ffi::{CString, CStr};
use std::fmt;
use std::mem::{transmute, uninitialized, zeroed};
use std::os::raw::c_char;

// Misc Notes
// Always pass a c_string.as_ptr() call into the function call directly and never
// before hand. Seems to make a huge difference (stuff stops working) otherwise

pub struct Context {
    context: LLVMContextRef,
}

// From Docs: A single context is not thread safe.
// However, different contexts can execute on different threads simultaneously.
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

pub struct Builder {
    builder: LLVMBuilderRef,
}

impl Builder {
    fn new(builder: LLVMBuilderRef) -> Self {
        assert!(!builder.is_null());

        Builder {
            builder: builder
        }
    }

    pub fn build_return(&self, value: Option<Value>) -> Value {
        // let value = unsafe {
        //     value.map_or(LLVMBuildRetVoid(self.builder), |value| LLVMBuildRet(self.builder, value.value))
        // };

        let value = unsafe { match value {
            Some(v) => LLVMBuildRet(self.builder, v.value),
            None => LLVMBuildRetVoid(self.builder),
        }};

        Value::new(value)
    }

    pub fn build_call<V: Into<Value> + Copy>(&self, function: &FunctionValue, args: &Vec<V>, name: &str) -> Value {
        // LLVM gets upset when void calls are named because they don't return anything
        let name = unsafe {
            match LLVMGetTypeKind(LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(function.fn_value)))) {
                LLVMTypeKind::LLVMVoidTypeKind => "",
                _ => name,
            }
        };

        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        // REVIEW: Had to make Value Copy + Clone to get this to work...
        // Is this safe, given Value is a raw ptr wrapper?
        // I suppose in theory LLVM should never delete the values in the scope of this call, but still
        let arg_values: Vec<Value> = args.iter().map(|val| (*val).into()).collect();

        // WARNING: transmute will no longer work correctly if Value gains more fields
        // We're avoiding reallocation by telling rust Vec<Value> is identical to Vec<LLVMValueRef>
        let mut args: Vec<LLVMValueRef> = unsafe {
            transmute(arg_values)
        };

        let value = unsafe {
            LLVMBuildCall(self.builder, function.fn_value, args.as_mut_ptr(), args.len() as u32, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_gep<V: Into<Value> + Copy>(&self, ptr: &Value, ordered_indexes: &Vec<V>, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        // TODO: Assert vec values are all i32 => Result? Might not always be desirable
        // REVIEW: Had to make Value Copy + Clone to get this to work...
        // Is this safe, given Value is a raw ptr wrapper?
        // I suppose in theory LLVM should never delete the values in the scope of this call, but still
        let index_values: Vec<Value> = ordered_indexes.iter().map(|val| (*val).into()).collect();

        // WARNING: transmute will no longer work correctly if Value gains more fields
        // We're avoiding reallocation by telling rust Vec<Value> is identical to Vec<LLVMValueRef>
        let mut index_values: Vec<LLVMValueRef> = unsafe {
            transmute(index_values)
        };

        let value = unsafe {
            LLVMBuildGEP(self.builder, ptr.value, index_values.as_mut_ptr(), index_values.len() as u32, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_phi(&self, type_: &Type, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPhi(self.builder, type_.type_, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_store(&self, value: &Value, ptr: &Value) -> Value {
        let value = unsafe {
            LLVMBuildStore(self.builder, value.value, ptr.value)
        };

        Value::new(value)
    }

    pub fn build_load(&self, ptr: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildLoad(self.builder, ptr.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_stack_allocation(&self, type_: &Type, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAlloca(self.builder, type_.type_, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_heap_allocation(&self, type_: &Type, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMalloc(self.builder, type_.type_, c_string.as_ptr())
        };

        Value::new(value)
    }

    // TODO: Rename to "build_heap_allocated_aRray" + stack version?
    pub fn build_array_heap_allocation<V: Into<Value> + Copy>(&self, type_: &Type, size: &V, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayMalloc(self.builder, type_.type_, (*size).into().value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_stack_allocated_array<V: Into<Value> + Copy>(&self, type_: &Type, size: &V, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayAlloca(self.builder, type_.type_, (*size).into().value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_free(&self, ptr: &Value) -> Value { // REVIEW: Why does free return? Seems like original pointer? Ever useful?
        let val = unsafe {
            LLVMBuildFree(self.builder, ptr.value)
        };

        Value::new(val)
    }

    pub fn insert_instruction(&self, value: Value) {
        unsafe {
            LLVMInsertIntoBuilder(self.builder, value.value);
        }
    }

    pub fn get_insert_block(&self) -> BasicBlock {
        let bb = unsafe {
            LLVMGetInsertBlock(self.builder)
        };

        BasicBlock::new(bb)
    }

    pub fn build_int_div(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        // TODO: Support signed

        let value = unsafe {
            LLVMBuildUDiv(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_float_div(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFDiv(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_int_add(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAdd(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_float_add(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFAdd(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_xor(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildXor(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_and(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAnd(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_or(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildOr(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_int_sub(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSub(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_float_sub(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFSub(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_int_mul(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMul(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }


    /// REVIEW: Untested
    pub fn build_float_mul(&self, left_value: &Value, right_value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFMul(self.builder, left_value.value, right_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }


    /// REVIEW: Untested
    pub fn build_cast(&self, op: LLVMOpcode, from_value: &Value, to_type: &Type, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildCast(self.builder, op, from_value.value, to_type.type_, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_pointer_cast(&self, from: &Value, to: &Type, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPointerCast(self.builder, from.value, to.type_, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_int_compare(&self, op: LLVMIntPredicate, left_val: &Value, right_val: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildICmp(self.builder, op, left_val.value, right_val.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_float_compare(&self, op: LLVMRealPredicate, left_val: &Value, right_val: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFCmp(self.builder, op, left_val.value, right_val.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_unconditional_branch(&self, destination_block: &BasicBlock) -> Value {
        let value = unsafe {
            LLVMBuildBr(self.builder, destination_block.basic_block)
        };

        Value::new(value)
    }

    pub fn build_conditional_branch(&self, comparison: &Value, then_block: &BasicBlock, else_block: &BasicBlock) -> Value {
        let value = unsafe {
            LLVMBuildCondBr(self.builder, comparison.value, then_block.basic_block, else_block.basic_block)
        };

        Value::new(value)
    }

    /// REVIEW: Combine with float neg?
    /// REVIEW: Untested
    pub fn build_neg(&self, value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNeg(self.builder, value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Combine with int neg?
    /// REVIEW: Untested
    pub fn build_float_neg(&self, value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFNeg(self.builder, value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_not(&self, value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNot(self.builder, value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn position_at_end(&self, basic_block: &BasicBlock) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, basic_block.basic_block);
        }
    }

    pub fn build_extract_value<V: AsRef<LLVMValueRef>>(&self, value: &V, index: u32, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExtractValue(self.builder, *value.as_ref(), index, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_insert_value(&self, value: &Value, ptr: &Value, index: u32, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildInsertValue(self.builder, value.value, ptr.value, index, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_unreachable(&self) -> Value {
        let val = unsafe {
            LLVMBuildUnreachable(self.builder)
        };

        Value::new(val)
    }

    /// REVIEW: Untested
    // TODO: Better name for num?
    pub fn build_fence(&self, atmoic_ordering: LLVMAtomicOrdering, num: i32, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildFence(self.builder, atmoic_ordering, num, c_string.as_ptr())
        };

        Value::new(val)
    }

    pub fn build_is_null(&self, value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNull(self.builder, value.value, c_string.as_ptr())
        };

        Value::new(val)
    }

    pub fn build_is_not_null(&self, value: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNotNull(self.builder, value.value, c_string.as_ptr())
        };

        Value::new(val)
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
        }
    }
}

pub struct Module {
    module: LLVMModuleRef,
}

impl Module {
    fn new(module: LLVMModuleRef) -> Self {
        assert!(!module.is_null());

        Module {
            module: module
        }
    }

    pub fn add_function(&self, name: &str, return_type: FunctionType) -> FunctionValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMAddFunction(self.module, c_string.as_ptr(), return_type.fn_type)
        };

        // unsafe {
        //     LLVMSetLinkage(value, LLVMCommonLinkage);
        // }

        FunctionValue::new(value)
    }

    pub fn get_function(&self, name: &str) -> Option<FunctionValue> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMGetNamedFunction(self.module, c_string.as_ptr())
        };

        if value.is_null() {
            return None;
        }

        Some(FunctionValue::new(value))
    }

    pub fn get_type(&self, name: &str) -> Option<Type> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let type_ = unsafe {
            LLVMGetTypeByName(self.module, c_string.as_ptr())
        };

        if type_.is_null() {
            return None;
        }

        Some(Type::new(type_))
    }

    pub fn create_execution_engine(&self, jit_mode: bool) -> Result<ExecutionEngine, String> {
        let mut execution_engine = unsafe { uninitialized() };
        let mut err_str = unsafe { zeroed() };

        if jit_mode {
            unsafe {
                LLVMLinkInMCJIT();
            }
        }

        // TODO: Check that these calls are even needed
        let code = unsafe {
            LLVM_InitializeNativeTarget()
        };

        if code == 1 {
            return Err("Unknown error in initializing native target".into());
        }

        let code = unsafe {
            LLVM_InitializeNativeAsmPrinter()
        };

        if code == 1 {
            return Err("Unknown error in initializing native asm printer".into());
        }

        let code = unsafe {
            LLVM_InitializeNativeAsmParser()
        };

        if code == 1 { // REVIEW: Does parser need to go before printer?
            return Err("Unknown error in initializing native asm parser".into());
        }

        let code = unsafe {
            LLVM_InitializeNativeDisassembler()
        };

        if code == 1 {
            return Err("Unknown error in initializing native disassembler".into());
        }

        unsafe {
            LLVMLinkInInterpreter();
        }

        let code = unsafe {
            LLVMCreateExecutionEngineForModule(&mut execution_engine, self.module, &mut err_str) // Should take ownership of module
        };

        if code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(err_str);

                rust_str
            };

            return Err(rust_str);
        }

        Ok(ExecutionEngine::new(execution_engine, jit_mode))
    }

    pub fn create_function_pass_manager(&self) -> PassManager {
        let pass_manager = unsafe {
            LLVMCreateFunctionPassManagerForModule(self.module)
        };

        PassManager::new(pass_manager)
    }

    pub fn add_global(&self, type_: &Type, init_value: &Option<Value>, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMAddGlobal(self.module, type_.type_, c_string.as_ptr())
        };

        if let &Some(ref init_val) = init_value {
            unsafe {
                LLVMSetInitializer(value, init_val.value)
            }
        }

        Value::new(value)
    }

    pub fn verify(&self, print: bool) -> bool {
        let err_str: *mut *mut i8 = unsafe { zeroed() };

        let action = if print == true {
            LLVMVerifierFailureAction::LLVMPrintMessageAction
        } else {
            LLVMVerifierFailureAction::LLVMReturnStatusAction
        };

        let code = unsafe {
            LLVMVerifyModule(self.module, action, err_str)
        };

        if code == 1 && !err_str.is_null() {
            unsafe {
                if print {
                    let rust_str = CStr::from_ptr(*err_str).to_str().unwrap();

                    println!("{}", rust_str); // FIXME: Should probably be stderr?
                }

                LLVMDisposeMessage(*err_str);
            }
        }

        code == 0
    }

    pub fn set_data_layout(&self, data_layout: DataLayout) {
        unsafe {
            LLVMSetDataLayout(self.module, data_layout.data_layout)
        }
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpModule(self.module);
        }
    }
}

// REVIEW: Drop for Module? There's a LLVM method, but I read context dispose takes care of it...

pub struct ExecutionEngine {
    execution_engine: LLVMExecutionEngineRef,
    jit_mode: bool,
}

impl ExecutionEngine {
    fn new(execution_engine: LLVMExecutionEngineRef, jit_mode: bool) -> ExecutionEngine {
        assert!(!execution_engine.is_null());

        ExecutionEngine {
            execution_engine: execution_engine,
            jit_mode: jit_mode,
        }
    }

    pub fn add_module(&mut self, module: &Module) {
        unsafe {
            LLVMAddModule(self.execution_engine, module.module)
        }
    }

    /// WARNING: The returned address *will* be invalid if the EE drops first
    pub fn get_function_address(&self, fn_name: &str) -> Result<u64, String> {

        if !self.jit_mode {
            return Err("ExecutionEngineError: Cannot use get_function_address in non jit_mode".into());
        }

        let c_string = CString::new(fn_name).expect("Conversion to CString failed unexpectedly");

        let address = unsafe {
            LLVMGetFunctionAddress(self.execution_engine, c_string.as_ptr())
        };

        if address == 0 {
            return Err(format!("ExecutionEngineError: Could not find function {}", fn_name));
        }

        Ok(address)
    }

    pub fn get_target_data(&self) -> TargetData {
        let target_data = unsafe {
            LLVMGetExecutionEngineTargetData(self.execution_engine)
        };

        TargetData::new(target_data)
    }

    pub fn run_function(&self, function: FunctionValue) {
        let mut args = vec![]; // TODO: Support args

        unsafe {
            LLVMRunFunction(self.execution_engine, function.fn_value, args.len() as u32, args.as_mut_ptr()); // REVIEW: usize to u32 ok??
        }
    }

    pub fn run_function_as_main(&self, function: FunctionValue) {
        let args = vec![]; // TODO: Support argc, argv
        let env_p = vec![]; // REVIEW: No clue what this is

        unsafe {
            LLVMRunFunctionAsMain(self.execution_engine, function.fn_value, args.len() as u32, args.as_ptr(), env_p.as_ptr()); // REVIEW: usize to u32 cast ok??
        }
    }
}

impl Drop for ExecutionEngine {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeExecutionEngine(self.execution_engine);
        }
    }
}

pub struct TargetData {
    target_data: LLVMTargetDataRef,
}

impl TargetData {
    fn new(target_data: LLVMTargetDataRef) -> TargetData {
        assert!(!target_data.is_null());

        TargetData {
            target_data: target_data
        }
    }

    pub fn get_data_layout(&self) -> DataLayout {
        let data_layout = unsafe {
            LLVMCopyStringRepOfTargetData(self.target_data)
        };

        DataLayout::new(data_layout)
    }

    pub fn get_bit_size(&self, type_: &Type) -> u64 {
        unsafe {
            LLVMSizeOfTypeInBits(self.target_data, type_.type_)
        }
    }
}

pub struct DataLayout {
    data_layout: *mut c_char,
}

impl DataLayout {
    fn new(data_layout: *mut c_char) -> DataLayout {
        assert!(!data_layout.is_null());

        DataLayout {
            data_layout: data_layout
        }
    }
}

impl Drop for DataLayout {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMessage(self.data_layout)
        }
    }
}

pub struct PassManager {
    pass_manager: LLVMPassManagerRef,
}

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

pub struct Type {
    type_: LLVMTypeRef,
}

impl Type {
    fn new(type_: LLVMTypeRef) -> Type {
        assert!(!type_.is_null());

        Type {
            type_: type_
        }
    }

    pub fn dump_type(&self) {
        unsafe {
            LLVMDumpType(self.type_);
        }
    }

    pub fn ptr_type(&self, address_space: u32) -> Type {
        let type_ = unsafe {
            LLVMPointerType(self.type_, address_space)
        };

        Type::new(type_)
    }

    pub fn fn_type(&self, param_types: &mut Vec<Type>, is_var_args: bool) -> FunctionType {
        // WARNING: transmute will no longer work correctly if Type gains more fields
        // We're avoiding reallocation by telling rust Vec<Type> is identical to Vec<LLVMTypeRef>
        let mut param_types: &mut Vec<LLVMTypeRef> = unsafe {
            transmute(param_types)
        };

        let fn_type = unsafe {
            LLVMFunctionType(self.type_, param_types.as_mut_ptr(), param_types.len() as u32, is_var_args as i32) // REVIEW: safe to cast usize to u32?
        };

        FunctionType::new(fn_type)
    }

    pub fn array_type(&self, size: u32) -> Type {
        let type_ = unsafe {
            LLVMArrayType(self.type_, size)
        };

        Type::new(type_)
    }

    pub fn const_int(&self, value: u64, sign_extend: bool) -> Value {
        // REVIEW: What if type is void??

        let value = unsafe {
            LLVMConstInt(self.type_, value, sign_extend as i32)
        };

        Value::new(value)
    }

    pub fn const_float(&self, value: f64) -> Value {
        // REVIEW: What if type is void??

        let value = unsafe {
            LLVMConstReal(self.type_, value)
        };

        Value::new(value)
    }

    pub fn const_array(&self, values: Vec<Value>) -> Value {
        // WARNING: transmute will no longer work correctly if Type gains more fields
        // We're avoiding reallocation by telling rust Vec<Type> is identical to Vec<LLVMTypeRef>
        let mut values: Vec<LLVMValueRef> = unsafe {
            transmute(values)
        };

        let value = unsafe {
            LLVMConstArray(self.type_, values.as_mut_ptr(), values.len() as u32)
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn get_undef(&self) -> Value {
        let value = unsafe {
            LLVMGetUndef(self.type_)
        };

        Value::new(value)
    }

    /// LLVM 3.7+
    /// REVIEW: Untested
    pub fn get_type_at_struct_index(&self, index: u32) -> Option<Type> {
        // REVIEW: This should only be used on Struct Types, so add a StructType?
        let type_ = unsafe {
            LLVMStructGetTypeAtIndex(self.type_, index)
        };

        if type_.is_null() {
            return None;
        }

        Some(Type::new(type_))
    }

    pub fn get_kind(&self) -> LLVMTypeKind {
        unsafe {
            LLVMGetTypeKind(self.type_)
        }
    }

    /// REVIEW: Untested
    pub fn get_alignment(&self) -> Value {
        let val = unsafe {
            LLVMAlignOf(self.type_)
        };

        Value::new(val)
    }

    /// REVIEW: Untested
    pub fn const_struct(&self, value: &mut Value, num: u32) -> Value {
        // REVIEW: What if not a struct? Need StructType?
        // TODO: Better name for num. What's it for?
        let val = unsafe {
            LLVMConstNamedStruct(self.type_, &mut value.value, num)
        };

        Value::new(val)
    }

    // FIXME: Not approved by the FDA, may cause segfaults
    // If multiple Context objects are created, one is bound to be `Drop`ped at end of a scope
    // Should create only a Context reference to avoid `Drop`
    fn get_context(&self) -> Context { // REVIEW: Option<Context>? I believe types can be context-less (maybe not, it might auto assign the global context (if any??))
        let context = unsafe {
            LLVMGetTypeContext(self.type_)
        };

        Context::new(context)
    }

    /// REVIEW: Untested
    pub fn is_sized(&self) -> bool {
        unsafe {
            LLVMTypeIsSized(self.type_) == 1
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(self.type_))
        };
        write!(f, "Type {{\n    address: {:?}\n    llvm_type: {:?}\n}}", self.type_, llvm_type)
    }
}

pub struct FunctionValue {
    fn_value: LLVMValueRef,
}

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

pub struct FunctionType {
    fn_type: LLVMTypeRef,
}

impl FunctionType {
    fn new(fn_type: LLVMTypeRef) -> FunctionType {
        assert!(!fn_type.is_null());

        FunctionType {
            fn_type: fn_type
        }
    }

    /// REVIEW: Untested
    pub fn is_var_arg(&self) -> bool {
        unsafe {
            LLVMIsFunctionVarArg(self.fn_type) == 1
        }
    }

    /// FIXME: Not working correctly
    fn get_param_types(&self) -> Vec<Type> {
        let count = self.count_param_types();
        let raw_vec = unsafe { uninitialized() };

        unsafe {
            LLVMGetParamTypes(self.fn_type, raw_vec);

            transmute(Vec::from_raw_parts(raw_vec, count as usize, count as usize))
        }
    }

    pub fn count_param_types(&self) -> u32 {
        unsafe {
            LLVMCountParamTypes(self.fn_type)
        }
    }
}

impl fmt::Debug for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(self.fn_type))
        };

        write!(f, "FunctionType {{\n    address: {:?}\n    llvm_type: {:?}\n}}", self.fn_type, llvm_type)
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

#[derive(Clone, Copy)]
pub struct Value {
    value: LLVMValueRef,
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

pub struct BasicBlock {
    basic_block: LLVMBasicBlockRef,
}

impl BasicBlock {
    fn new(basic_block: LLVMBasicBlockRef) -> BasicBlock {
        // TODO: debug mode only assertions
        {
            assert!(!basic_block.is_null());

            unsafe {
                assert!(!LLVMIsABasicBlock(basic_block as LLVMValueRef).is_null()) // NOTE: There is a LLVMBasicBlockAsValue but it might be the same as casting
            }
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
