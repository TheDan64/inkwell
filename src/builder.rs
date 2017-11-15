use either::Either;
use llvm_sys::core::{LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildArrayAlloca, LLVMBuildArrayMalloc, LLVMBuildBr, LLVMBuildCall, LLVMBuildCast, LLVMBuildCondBr, LLVMBuildExtractValue, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFence, LLVMBuildFMul, LLVMBuildFNeg, LLVMBuildFree, LLVMBuildFSub, LLVMBuildGEP, LLVMBuildICmp, LLVMBuildInsertValue, LLVMBuildIsNotNull, LLVMBuildIsNull, LLVMBuildLoad, LLVMBuildMalloc, LLVMBuildMul, LLVMBuildNeg, LLVMBuildNot, LLVMBuildOr, LLVMBuildPhi, LLVMBuildPointerCast, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildSub, LLVMBuildUDiv, LLVMBuildUnreachable, LLVMBuildXor, LLVMDisposeBuilder, LLVMGetElementType, LLVMGetInsertBlock, LLVMGetReturnType, LLVMGetTypeKind, LLVMInsertIntoBuilder, LLVMPositionBuilderAtEnd, LLVMTypeOf, LLVMSetTailCall, LLVMBuildExtractElement, LLVMBuildInsertElement, LLVMBuildIntToPtr, LLVMBuildPtrToInt, LLVMInsertIntoBuilderWithName, LLVMClearInsertionPosition, LLVMCreateBuilder, LLVMPositionBuilder, LLVMPositionBuilderBefore, LLVMBuildAggregateRet, LLVMBuildStructGEP, LLVMBuildInBoundsGEP, LLVMBuildPtrDiff, LLVMBuildNSWAdd, LLVMBuildNUWAdd, LLVMBuildNSWSub, LLVMBuildNUWSub, LLVMBuildNSWMul, LLVMBuildNUWMul, LLVMBuildSDiv, LLVMBuildSRem, LLVMBuildURem, LLVMBuildFRem, LLVMBuildNSWNeg, LLVMBuildNUWNeg, LLVMBuildFPToUI, LLVMBuildFPToSI, LLVMBuildSIToFP, LLVMBuildUIToFP, LLVMBuildFPTrunc, LLVMBuildFPExt, LLVMBuildIntCast, LLVMBuildFPCast, LLVMBuildSExtOrBitCast, LLVMBuildZExtOrBitCast, LLVMBuildTruncOrBitCast, LLVMBuildSwitch, LLVMAddCase, LLVMBuildShl, LLVMBuildAShr, LLVMBuildLShr, LLVMBuildGlobalString, LLVMBuildGlobalStringPtr, LLVMBuildExactSDiv, LLVMBuildTrunc, LLVMBuildSExt, LLVMBuildZExt};
use llvm_sys::prelude::{LLVMBuilderRef, LLVMValueRef};
use llvm_sys::{LLVMOpcode, LLVMTypeKind, LLVMAtomicOrdering};

use {IntPredicate, FloatPredicate};
use basic_block::BasicBlock;
use values::{AggregateValue, AsValueRef, BasicValue, BasicValueEnum, PhiValue, FunctionValue, FloatValue, IntValue, PointerValue, VectorValue, InstructionValue, GlobalValue};
use types::{AsTypeRef, BasicType, PointerType, IntType, FloatType};

use std::ffi::CString;

pub struct Builder {
    builder: LLVMBuilderRef,
}

impl Builder {
    pub(crate) fn new(builder: LLVMBuilderRef) -> Self {
        assert!(!builder.is_null());

        Builder {
            builder: builder
        }
    }

    pub fn create() -> Self {
        let builder = unsafe {
            LLVMCreateBuilder()
        };

        Builder::new(builder)
    }

    // REVIEW: Would probably make this API a bit simpler by taking Into<Option<&BasicValue>>
    // So that you could just do build_return(&value) or build_return(None)
    // Is that frowned upon?
    pub fn build_return(&self, value: Option<&BasicValue>) -> InstructionValue {
        // let value = unsafe {
        //     value.map_or(LLVMBuildRetVoid(self.builder), |value| LLVMBuildRet(self.builder, value.value))
        // };

        let value = unsafe {
            match value {
                Some(v) => LLVMBuildRet(self.builder, v.as_value_ref()),
                None => LLVMBuildRetVoid(self.builder),
            }
        };

        InstructionValue::new(value)
    }

    pub fn build_aggregate_return(&self, values: &[&BasicValue]) -> InstructionValue {
        let mut args: Vec<LLVMValueRef> = values.iter()
                                                .map(|val| val.as_value_ref())
                                                .collect();
        let value = unsafe {
            LLVMBuildAggregateRet(self.builder, args.as_mut_ptr(), args.len() as u32)
        };

        InstructionValue::new(value)
    }

    pub fn build_call(&self, function: &FunctionValue, args: &[&BasicValue], name: &str, tail_call: bool) -> Either<BasicValueEnum, InstructionValue> {
        // LLVM gets upset when void calls are named because they don't return anything
        let name = unsafe {
            match LLVMGetTypeKind(LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(function.as_value_ref())))) {
                LLVMTypeKind::LLVMVoidTypeKind => "",
                _ => name,
            }
        };

        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let mut args: Vec<LLVMValueRef> = args.iter()
                                              .map(|val| val.as_value_ref())
                                              .collect();
        let value = unsafe {
            LLVMBuildCall(self.builder, function.as_value_ref(), args.as_mut_ptr(), args.len() as u32, c_string.as_ptr())
        };

        if tail_call {
            unsafe {
                LLVMSetTailCall(value, true as i32)
            }
        }

        unsafe {
            match LLVMGetTypeKind(LLVMTypeOf(value)) {
                LLVMTypeKind::LLVMVoidTypeKind => Either::Right(InstructionValue::new(value)),
                _ => Either::Left(BasicValueEnum::new(value)),
            }
        }
    }

    // REVIEW: Doesn't GEP work on array too?
    pub fn build_gep(&self, ptr: &PointerValue, ordered_indexes: &[&IntValue], name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter()
                                                                 .map(|val| val.as_value_ref())
                                                                 .collect();
        let value = unsafe {
            LLVMBuildGEP(self.builder, ptr.as_value_ref(), index_values.as_mut_ptr(), index_values.len() as u32, c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // REVIEW: Doesn't GEP work on array too?
    pub fn build_in_bounds_gep(&self, ptr: &PointerValue, ordered_indexes: &[&IntValue], name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter()
                                                                 .map(|val| val.as_value_ref())
                                                                 .collect();
        let value = unsafe {
            LLVMBuildInBoundsGEP(self.builder, ptr.as_value_ref(), index_values.as_mut_ptr(), index_values.len() as u32, c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // REVIEW: Shouldn't this take a StructValue? Or does it still need to be PointerValue<StructValue>?
    pub fn build_struct_gep(&self, ptr: &PointerValue, index: u32, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildStructGEP(self.builder, ptr.as_value_ref(), index, c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    pub fn build_ptr_diff(&self, lhs_ptr: &PointerValue, rhs_ptr: &PointerValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPtrDiff(self.builder, lhs_ptr.as_value_ref(), rhs_ptr.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_phi(&self, type_: &BasicType, name: &str) -> PhiValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPhi(self.builder, type_.as_type_ref(), c_string.as_ptr())
        };

        PhiValue::new(value)
    }

    pub fn build_store(&self, ptr: &PointerValue, value: &BasicValue) -> InstructionValue {
        let value = unsafe {
            LLVMBuildStore(self.builder, value.as_value_ref(), ptr.as_value_ref())
        };

        InstructionValue::new(value)
    }

    pub fn build_load(&self, ptr: &PointerValue, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildLoad(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    // TODOC: Stack allocation
    pub fn build_alloca(&self, type_: &BasicType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAlloca(self.builder, type_.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // TODOC: Stack allocation
    pub fn build_array_alloca(&self, type_: &BasicType, size: &IntValue, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayAlloca(self.builder, type_.as_type_ref(), size.as_value_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // TODOC: Heap allocation
    // FIXME: Not working
    pub fn build_malloc(&self, type_: &BasicType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMalloc(self.builder, type_.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // TODOC: Heap allocation
    // FIXME: Not working
    pub fn build_array_malloc(&self, type_: &BasicType, size: &IntValue, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayMalloc(self.builder, type_.as_type_ref(), size.as_value_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // SubType: <P>(&self, ptr: &PointerValue<P>) -> InstructionValue {
    pub fn build_free(&self, ptr: &PointerValue) -> InstructionValue {
        let val = unsafe {
            LLVMBuildFree(self.builder, ptr.as_value_ref())
        };

        InstructionValue::new(val)
    }

    pub fn insert_instruction(&self, instruction: &InstructionValue, name: Option<&str>) {
        match name {
            Some(name) => {
                let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

                unsafe {
                    LLVMInsertIntoBuilderWithName(self.builder, instruction.as_value_ref(), c_string.as_ptr())
                }
            },
            None => unsafe {
                LLVMInsertIntoBuilder(self.builder, instruction.as_value_ref());
            }
        }
    }

    pub fn get_insert_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetInsertBlock(self.builder)
        };

        BasicBlock::new(bb)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I: IntSubType>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    //     if I::sign() == Unsigned { LLVMBuildUDiv() } else { LLVMBuildSDiv() }
    pub fn build_int_unsigned_div(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildUDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_signed_div(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_exact_signed_div(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExactSDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_unsigned_rem(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildURem(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }


    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_signed_rem(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSRem(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_int_s_extend(&self, int_value: &IntValue, int_type: &IntType, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSExt(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_int_s_extend_or_bit_cast(&self, int_value: &IntValue, int_type: &IntType, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSExtOrBitCast(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_int_z_extend(&self, int_value: &IntValue, int_type: &IntType, name: &str) -> IntValue {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildZExt(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_int_z_extend_or_bit_cast(&self, int_value: &IntValue, int_type: &IntType, name: &str) -> IntValue {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildZExtOrBitCast(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_int_truncate(&self, int_value: &IntValue, int_type: &IntType, name: &str) -> IntValue {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildTrunc(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_int_truncate_or_bit_cast(&self, int_value: &IntValue, int_type: &IntType, name: &str) -> IntValue {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildTruncOrBitCast(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_float_rem(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFRem(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    // REVIEW: Consolidate these two casts into one via subtypes
    pub fn build_float_to_unsigned_int(&self, float: &FloatValue, int_type: &IntType, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPToUI(self.builder, float.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_float_to_signed_int(&self, float: &FloatValue, int_type: &IntType, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPToSI(self.builder, float.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Consolidate these two casts into one via subtypes
    pub fn build_unsigned_int_to_float(&self, int: &IntValue, float_type: &FloatType, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildUIToFP(self.builder, int.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    pub fn build_signed_int_to_float(&self, int: &FloatValue, float_type: &FloatType, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSIToFP(self.builder, int.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    pub fn build_float_trunc(&self, float: &FloatValue, float_type: &FloatType, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPTrunc(self.builder, float.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    pub fn build_float_ext(&self, float: &FloatValue, float_type: &FloatType, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPExt(self.builder, float.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    pub fn build_float_cast(&self, float: &FloatValue, float_type: &FloatType, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPCast(self.builder, float.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    // SubType: <L, R>(&self, lhs: &IntValue<L>, rhs: &IntType<R>, name: &str) -> IntValue<R> {
    pub fn build_int_cast(&self, int: &IntValue, int_type: &IntType, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildIntCast(self.builder, int.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_float_div(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_add(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_add via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nsw_add(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_add via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nuw_add(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // SubType: <F>(&self, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name: &str) -> FloatValue<F> {
    pub fn build_float_add(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    // SubType: (&self, lhs: &IntValue<bool>, rhs: &IntValue<bool>, name: &str) -> IntValue<bool> {
    pub fn build_xor(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildXor(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // SubType: (&self, lhs: &IntValue<bool>, rhs: &IntValue<bool>, name: &str) -> IntValue<bool> {
    pub fn build_and(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAnd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // SubType: (&self, lhs: &IntValue<bool>, rhs: &IntValue<bool>, name: &str) -> IntValue<bool> {
    pub fn build_or(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildOr(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    /// Builds an `IntValue` containing the result of a logical left shift instruction.
    ///
    /// # Example
    /// A logical left shift is an operation in which an integer value's bits are shifted left by N number of positions.
    ///
    /// ```rust,no_run
    /// assert_eq!(0b0000_0001 << 0, 0b0000_0001);
    /// assert_eq!(0b0000_0001 << 1, 0b0000_0010);
    /// assert_eq!(0b0000_0011 << 2, 0b0000_1100);
    /// ```
    ///
    /// In Rust, a function that could do this for 8bit values looks like:
    ///
    /// ```rust,no_run
    /// fn left_shift(value: u8, n: u8) -> u8 {
    ///     value << n
    /// }
    /// ```
    ///
    /// And in Inkwell, the corresponding function would look roughly like:
    ///
    /// ```rust,no_run
    /// use inkwell::context::Context;
    ///
    /// // Setup
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let builder = context.create_builder();
    /// let i8_type = context.i8_type();
    /// let fn_type = i8_type.fn_type(&[&i8_type, &i8_type], false);
    ///
    /// // Function Definition
    /// let function = module.add_function("left_shift", &fn_type, None);
    /// let value = function.get_first_param().unwrap().into_int_value();
    /// let n = function.get_nth_param(1).unwrap().into_int_value();
    /// let entry_block = function.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_block);
    ///
    /// let shift = builder.build_left_shift(&value, &n, "left_shift"); // value << n
    ///
    /// builder.build_return(Some(&shift));
    /// ```
    pub fn build_left_shift(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildShl(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    /// Builds an `IntValue` containing the result of a right shift instruction.
    ///
    /// # Example
    /// A right shift is an operation in which an integer value's bits are shifted right by N number of positions.
    /// It may either be logical and have its leftmost N bit(s) filled with zeros or sign extended and filled with ones
    /// if the leftmost bit was one.
    ///
    /// ```rust,no_run
    /// // Logical Right Shift
    /// assert_eq!(0b1100_0000 >> 2, 0b0011_0000);
    /// assert_eq!(0b0000_0010 >> 1, 0b0000_0001);
    /// assert_eq!(0b0000_1100 >> 2, 0b0000_0011);
    ///
    /// // Sign Extended Right Shift
    /// assert_eq!(0b0100_0000i8 >> 2, 0b0001_0000);
    /// assert_eq!(0b1110_0000i8 >> 1, 0b1111_0000);
    /// assert_eq!(0b1100_0000i8 >> 2, 0b1111_0000);
    /// ```
    ///
    /// In Rust, functions that could do this for 8bit values look like:
    ///
    /// ```rust,no_run
    /// fn logical_right_shift(value: u8, n: u8) -> u8 {
    ///     value >> n
    /// }
    ///
    /// fn sign_extended_right_shift(value: i8, n: u8) -> i8 {
    ///     value >> n
    /// }
    /// ```
    /// Notice that, in Rust (and most other languages), whether or not a value is sign extended depends wholly on whether
    /// or not the type is signed (ie an i8 is a signed 8 bit value). LLVM does not make this distinction for you.
    ///
    /// In Inkwell, the corresponding functions would look roughly like:
    ///
    /// ```rust,no_run
    /// use inkwell::context::Context;
    ///
    /// // Setup
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let builder = context.create_builder();
    /// let i8_type = context.i8_type();
    /// let fn_type = i8_type.fn_type(&[&i8_type, &i8_type], false);
    ///
    /// // Function Definition
    /// let function = module.add_function("right_shift", &fn_type, None);
    /// let value = function.get_first_param().unwrap().into_int_value();
    /// let n = function.get_nth_param(1).unwrap().into_int_value();
    /// let entry_block = function.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_block);
    ///
    /// // Whether or not your right shift is sign extended (true) or logical (false) depends
    /// // on the boolean input parameter:
    /// let shift = builder.build_right_shift(&value, &n, false, "right_shift"); // value >> n
    ///
    /// builder.build_return(Some(&shift));
    /// ```
    pub fn build_right_shift(&self, lhs: &IntValue, rhs: &IntValue, sign_extend: bool, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            if sign_extend {
                LLVMBuildAShr(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
            } else {
                LLVMBuildLShr(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
            }
        };

        IntValue::new(value)
    }

    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_sub(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_sub via flag param
    pub fn build_int_nsw_sub(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_sub via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nuw_sub(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // SubType: <F>(&self, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name: &str) -> FloatValue<F> {
    pub fn build_float_sub(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_mul(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_mul via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nsw_mul(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_mul via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nuw_mul(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // SubType: <F>(&self, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name: &str) -> FloatValue<F> {
    pub fn build_float_mul(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    pub fn build_cast(&self, op: LLVMOpcode, from_value: &BasicValue, to_type: &BasicType, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildCast(self.builder, op, from_value.as_value_ref(), to_type.as_type_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    // SubType: <F, T>(&self, from: &PointerValue<F>, to: &PointerType<T>, name: &str) -> PointerValue<T> {
    pub fn build_pointer_cast(&self, from: &PointerValue, to: &PointerType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPointerCast(self.builder, from.as_value_ref(), to.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // SubType: <I>(&self, op, lhs: &IntValue<I>, rhs: &IntValue<I>, name) -> IntValue<bool> { ?
    pub fn build_int_compare(&self, op: IntPredicate, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildICmp(self.builder, op.as_llvm_predicate(), lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // SubType: <F>(&self, op, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name) -> IntValue<bool> { ?
    pub fn build_float_compare(&self, op: FloatPredicate, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFCmp(self.builder, op.as_llvm_predicate(), lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_unconditional_branch(&self, destination_block: &BasicBlock) -> InstructionValue {
        let value = unsafe {
            LLVMBuildBr(self.builder, destination_block.basic_block)
        };

        InstructionValue::new(value)
    }

    pub fn build_conditional_branch(&self, comparison: &IntValue, then_block: &BasicBlock, else_block: &BasicBlock) -> InstructionValue {
        let value = unsafe {
            LLVMBuildCondBr(self.builder, comparison.as_value_ref(), then_block.basic_block, else_block.basic_block)
        };

        InstructionValue::new(value)
    }

    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<I> {
    pub fn build_int_neg(&self, value: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_neg via flag and subtypes
    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<I> {
    pub fn build_int_nsw_neg(&self, value: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<I> {
    pub fn build_int_nuw_neg(&self, value: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // SubType: <F>(&self, value: &FloatValue<F>, name) -> FloatValue<F> {
    pub fn build_float_neg(&self, value: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<bool> { ?
    pub fn build_not(&self, value: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNot(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // REVIEW: What if instruction and basic_block are completely unrelated?
    // It'd be great if we could get the BB from the instruction behind the scenes
    pub fn position_at(&self, basic_block: &BasicBlock, instruction: &InstructionValue) {
        unsafe {
            LLVMPositionBuilder(self.builder, basic_block.basic_block, instruction.as_value_ref())
        }
    }

    pub fn position_before(&self, instruction: &InstructionValue) {
        unsafe {
            LLVMPositionBuilderBefore(self.builder, instruction.as_value_ref())
        }
    }

    pub fn position_at_end(&self, basic_block: &BasicBlock) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, basic_block.basic_block);
        }
    }

    // REVIEW: How does LLVM treat out of bound index? Maybe we should return an Option?
    // or is that only in bounds GEP
    // REVIEW: Should this be AggregatePointerValue?
    pub fn build_extract_value(&self, value: &AggregateValue, index: u32, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExtractValue(self.builder, value.as_value_ref(), index, c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    // REVIEW: Should this be AggregatePointerValue instead of just PointerValue?
    pub fn build_insert_value(&self, value: &BasicValue, ptr: &PointerValue, index: u32, name: &str) -> InstructionValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildInsertValue(self.builder, value.as_value_ref(), ptr.as_value_ref(), index, c_string.as_ptr())
        };

        InstructionValue::new(value)
    }

    pub fn build_extract_element(&self, vector: &VectorValue, index: &IntValue, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExtractElement(self.builder, vector.as_value_ref(), index.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    pub fn build_insert_element(&self, vector: &VectorValue, element: &BasicValue, index: &IntValue, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildInsertElement(self.builder, vector.as_value_ref(), element.as_value_ref(), index.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    pub fn build_unreachable(&self) -> InstructionValue {
        let val = unsafe {
            LLVMBuildUnreachable(self.builder)
        };

        InstructionValue::new(val)
    }

    // REVIEW: Not sure if this should return InstructionValue or an actual value
    // TODO: Better name for num?
    pub fn build_fence(&self, atmoic_ordering: LLVMAtomicOrdering, num: i32, name: &str) -> InstructionValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildFence(self.builder, atmoic_ordering, num, c_string.as_ptr())
        };

        InstructionValue::new(val)
    }

    // SubType: <P>(&self, ptr: &PointerValue<P>, name) -> IntValue<bool> {
    pub fn build_is_null(&self, ptr: &PointerValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNull(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(val)
    }

    // SubType: <P>(&self, ptr: &PointerValue<P>, name) -> IntValue<bool> {
    pub fn build_is_not_null(&self, ptr: &PointerValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNotNull(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(val)
    }

    // SubType: <I, P>(&self, int: &IntValue<I>, ptr_type: &PointerType<P>, name) -> PointerValue<P> {
    pub fn build_int_to_ptr(&self, int: &IntValue, ptr_type: &PointerType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildIntToPtr(self.builder, int.as_value_ref(), ptr_type.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // SubType: <I, P>(&self, ptr: &PointerValue<P>, int_type: &IntType<I>, name) -> IntValue<I> {
    pub fn build_ptr_to_int(&self, ptr: &PointerValue, int_type: &IntType, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPtrToInt(self.builder, ptr.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn clear_insertion_position(&self) {
        unsafe {
            LLVMClearInsertionPosition(self.builder)
        }
    }

    // REVIEW: Returning InstructionValue is the safe move here; but if the value means something
    // (IE the result of the switch) it should probably return BasicValueEnum?
    // SubTypes: I think value and case values must be the same subtype (maybe). Case value might need to be constants
    pub fn build_switch(&self, value: &IntValue, else_block: &BasicBlock, cases: &[(&IntValue, &BasicBlock)]) -> InstructionValue {
        let switch_value = unsafe {
            LLVMBuildSwitch(self.builder, value.as_value_ref(), else_block.basic_block, cases.len() as u32)
        };

        for &(value, basic_block) in cases {
            unsafe {
                LLVMAddCase(switch_value, value.as_value_ref(), basic_block.basic_block)
            }
        }

        InstructionValue::new(switch_value)
    }

    pub fn build_global_string(&self, value: &str, name: &str) -> GlobalValue {
        let c_string_value = CString::new(value).expect("Conversion to CString failed unexpectedly");
        let c_string_name = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = unsafe {
            LLVMBuildGlobalString(self.builder, c_string_value.as_ptr(), c_string_name.as_ptr())
        };

        GlobalValue::new(value)
    }

    pub fn build_global_string_ptr(&self, value: &str, name: &str) -> GlobalValue {
        let c_string_value = CString::new(value).expect("Conversion to CString failed unexpectedly");
        let c_string_name = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = unsafe {
            LLVMBuildGlobalStringPtr(self.builder, c_string_value.as_ptr(), c_string_name.as_ptr())
        };

        GlobalValue::new(value)
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
        }
    }
}
