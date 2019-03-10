use either::{Either, Left, Right};
use llvm_sys::core::{LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildArrayAlloca, LLVMBuildArrayMalloc, LLVMBuildBr, LLVMBuildCall, LLVMBuildCast, LLVMBuildCondBr, LLVMBuildExtractValue, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFence, LLVMBuildFMul, LLVMBuildFNeg, LLVMBuildFree, LLVMBuildFSub, LLVMBuildGEP, LLVMBuildICmp, LLVMBuildInsertValue, LLVMBuildIsNotNull, LLVMBuildIsNull, LLVMBuildLoad, LLVMBuildMalloc, LLVMBuildMul, LLVMBuildNeg, LLVMBuildNot, LLVMBuildOr, LLVMBuildPhi, LLVMBuildPointerCast, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildSub, LLVMBuildUDiv, LLVMBuildUnreachable, LLVMBuildXor, LLVMDisposeBuilder, LLVMGetElementType, LLVMGetInsertBlock, LLVMGetReturnType, LLVMGetTypeKind, LLVMInsertIntoBuilder, LLVMPositionBuilderAtEnd, LLVMTypeOf, LLVMBuildExtractElement, LLVMBuildInsertElement, LLVMBuildIntToPtr, LLVMBuildPtrToInt, LLVMInsertIntoBuilderWithName, LLVMClearInsertionPosition, LLVMCreateBuilder, LLVMPositionBuilder, LLVMPositionBuilderBefore, LLVMBuildAggregateRet, LLVMBuildStructGEP, LLVMBuildInBoundsGEP, LLVMBuildPtrDiff, LLVMBuildNSWAdd, LLVMBuildNUWAdd, LLVMBuildNSWSub, LLVMBuildNUWSub, LLVMBuildNSWMul, LLVMBuildNUWMul, LLVMBuildSDiv, LLVMBuildSRem, LLVMBuildURem, LLVMBuildFRem, LLVMBuildNSWNeg, LLVMBuildNUWNeg, LLVMBuildFPToUI, LLVMBuildFPToSI, LLVMBuildSIToFP, LLVMBuildUIToFP, LLVMBuildFPTrunc, LLVMBuildFPExt, LLVMBuildIntCast, LLVMBuildFPCast, LLVMBuildSExtOrBitCast, LLVMBuildZExtOrBitCast, LLVMBuildTruncOrBitCast, LLVMBuildSwitch, LLVMAddCase, LLVMBuildShl, LLVMBuildAShr, LLVMBuildLShr, LLVMBuildGlobalString, LLVMBuildGlobalStringPtr, LLVMBuildExactSDiv, LLVMBuildTrunc, LLVMBuildSExt, LLVMBuildZExt, LLVMBuildSelect, LLVMBuildAddrSpaceCast, LLVMBuildBitCast, LLVMBuildShuffleVector, LLVMBuildVAArg, LLVMBuildIndirectBr, LLVMAddDestination};
use llvm_sys::prelude::{LLVMBuilderRef, LLVMValueRef};
use llvm_sys::{LLVMTypeKind};

use {AtomicOrdering, IntPredicate, FloatPredicate};
use basic_block::BasicBlock;
use values::{AggregateValue, AggregateValueEnum, AsValueRef, BasicValue, BasicValueEnum, PhiValue, FunctionValue, IntValue, PointerValue, VectorValue, InstructionValue, GlobalValue, IntMathValue, FloatMathValue, PointerMathValue, InstructionOpcode, CallSiteValue};
use types::{AsTypeRef, BasicType, IntMathType, FloatMathType, PointerType, PointerMathType};

use std::ffi::CString;

#[derive(Debug)]
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

    /// Creates a `Builder` belonging to the global `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::builder::Builder;
    ///
    /// let builder = Builder::create();
    /// ```
    pub fn create() -> Self {
        let builder = unsafe {
            LLVMCreateBuilder()
        };

        Builder::new(builder)
    }

    // REVIEW: Would probably make this API a bit simpler by taking Into<Option<impl BasicValue>>
    // So that you could just do build_return(value) or build_return(None)
    // Is that frowned upon?
    // TODO: Option<impl BasicValue>
    pub fn build_return(&self, value: Option<&BasicValue>) -> InstructionValue {
        let value = unsafe {
            value.map_or_else(|| LLVMBuildRetVoid(self.builder), |value| LLVMBuildRet(self.builder, value.as_value_ref()))
        };

        InstructionValue::new(value)
    }

    pub fn build_aggregate_return(&self, values: &[BasicValueEnum]) -> InstructionValue {
        let mut args: Vec<LLVMValueRef> = values.iter()
                                                .map(|val| val.as_value_ref())
                                                .collect();
        let value = unsafe {
            LLVMBuildAggregateRet(self.builder, args.as_mut_ptr(), args.len() as u32)
        };

        InstructionValue::new(value)
    }

    pub fn build_call<F>(&self, function: F, args: &[BasicValueEnum], name: &str) -> CallSiteValue
    where
        F: Into<FunctionOrPointerValue>,
    {
        let fn_val_ref = match function.into() {
            Left(val) => val.as_value_ref(),
            Right(val) => val.as_value_ref(),
        };

        // LLVM gets upset when void return calls are named because they don't return anything
        let name = unsafe {
            match LLVMGetTypeKind(LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(fn_val_ref)))) {
                LLVMTypeKind::LLVMVoidTypeKind => "",
                _ => name,
            }
        };

        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let mut args: Vec<LLVMValueRef> = args.iter()
                                              .map(|val| val.as_value_ref())
                                              .collect();
        let value = unsafe {
            LLVMBuildCall(self.builder, fn_val_ref, args.as_mut_ptr(), args.len() as u32, c_string.as_ptr())
        };

        CallSiteValue::new(value)
    }

    // REVIEW: Doesn't GEP work on array too?
    /// GEP is very likely to segfault if indexes are used incorrectly, and is therefore an unsafe function. Maybe we can change this in the future.
    pub unsafe fn build_gep(&self, ptr: PointerValue, ordered_indexes: &[IntValue], name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter()
                                                                 .map(|val| val.as_value_ref())
                                                                 .collect();
        let value = LLVMBuildGEP(self.builder, ptr.as_value_ref(), index_values.as_mut_ptr(), index_values.len() as u32, c_string.as_ptr());

        PointerValue::new(value)
    }

    // REVIEW: Doesn't GEP work on array too?
    // REVIEW: This could be merge in with build_gep via a in_bounds: bool param
    /// GEP is very likely to segfault if indexes are used incorrectly, and is therefore an unsafe function. Maybe we can change this in the future.
    pub unsafe fn build_in_bounds_gep(&self, ptr: PointerValue, ordered_indexes: &[IntValue], name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter()
                                                                 .map(|val| val.as_value_ref())
                                                                 .collect();
        let value = LLVMBuildInBoundsGEP(self.builder, ptr.as_value_ref(), index_values.as_mut_ptr(), index_values.len() as u32, c_string.as_ptr());

        PointerValue::new(value)
    }

    // REVIEW: Shouldn't this take a StructValue? Or does it still need to be PointerValue<StructValue>?
    // I think it's the latter. This might not be as unsafe as regular GEP. Should check to see if it lets us
    // go OOB. Maybe we could use the PointerValue<StructValue>'s struct info to do bounds checking...
    /// GEP is very likely to segfault if indexes are used incorrectly, and is therefore an unsafe function. Maybe we can change this in the future.
    pub unsafe fn build_struct_gep(&self, ptr: PointerValue, index: u32, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = LLVMBuildStructGEP(self.builder, ptr.as_value_ref(), index, c_string.as_ptr());

        PointerValue::new(value)
    }

    pub fn build_ptr_diff(&self, lhs_ptr: PointerValue, rhs_ptr: PointerValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPtrDiff(self.builder, lhs_ptr.as_value_ref(), rhs_ptr.as_value_ref(), c_string.as_ptr())
        };

        IntValue::new(value)
    }

    // SubTypes: Maybe this should return PhiValue<T>? That way we could force incoming values to be of T::Value?
    // That is, assuming LLVM complains about different phi types.. which I imagine it would. But this would get
    // tricky with VoidType since it has no instance value?
    // TODOC: Phi Instruction(s) must be first instruction(s) in a BasicBlock.
    // REVIEW: Not sure if we can enforce the above somehow via types.
    pub fn build_phi<T: BasicType>(&self, type_: T, name: &str) -> PhiValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPhi(self.builder, type_.as_type_ref(), c_string.as_ptr())
        };

        PhiValue::new(value)
    }

    pub fn build_store<V: BasicValue>(&self, ptr: PointerValue, value: V) -> InstructionValue {
        let value = unsafe {
            LLVMBuildStore(self.builder, value.as_value_ref(), ptr.as_value_ref())
        };

        InstructionValue::new(value)
    }

    pub fn build_load(&self, ptr: PointerValue, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildLoad(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    // TODOC: Stack allocation
    pub fn build_alloca<T: BasicType>(&self, ty: T, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAlloca(self.builder, ty.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // TODOC: Stack allocation
    pub fn build_array_alloca<T: BasicType>(&self, ty: T, size: IntValue, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayAlloca(self.builder, ty.as_type_ref(), size.as_value_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // TODOC: Heap allocation
    pub fn build_malloc<T: BasicType>(&self, ty: T, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMalloc(self.builder, ty.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // TODOC: Heap allocation
    pub fn build_array_malloc<T: BasicType>(&self, ty: T, size: IntValue, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayMalloc(self.builder, ty.as_type_ref(), size.as_value_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // SubType: <P>(&self, ptr: PointerValue<P>) -> InstructionValue {
    pub fn build_free(&self, ptr: PointerValue) -> InstructionValue {
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
    pub fn build_int_unsigned_div<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildUDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_signed_div<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_exact_signed_div<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExactSDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_unsigned_rem<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildURem(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }


    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_signed_rem<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSRem(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_int_s_extend<T: IntMathValue>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSExt(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Does this need vector support?
    pub fn build_address_space_cast(&self, ptr_val: PointerValue, ptr_type: PointerType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAddrSpaceCast(self.builder, ptr_val.as_value_ref(), ptr_type.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    /// Builds a bitcast instruction. A bitcast reinterprets the bits of one value
    /// into a value of another type which has the same bit width.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("bc");
    /// let void_type = context.void_type();
    /// let f32_type = context.f32_type();
    /// let i32_type = context.i32_type();
    /// let arg_types = [i32_type.into()];
    /// let fn_type = void_type.fn_type(&arg_types, false);
    /// let fn_value = module.add_function("bc", fn_type, None);
    /// let builder = context.create_builder();
    /// let entry = fn_value.append_basic_block("entry");
    /// let i32_arg = fn_value.get_first_param().unwrap().into_int_value();
    ///
    /// builder.position_at_end(&entry);
    ///
    /// builder.build_bitcast(i32_arg, f32_type, "i32tof32");
    /// builder.build_return(None);
    ///
    /// assert!(module.verify().is_ok());
    /// ```
    pub fn build_bitcast<T, V>(&self, val: V, ty: T, name: &str) -> BasicValueEnum
    where
        T: BasicType,
        V: BasicValue,
    {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildBitCast(self.builder, val.as_value_ref(), ty.as_type_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    pub fn build_int_s_extend_or_bit_cast<T: IntMathValue>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSExtOrBitCast(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_int_z_extend<T: IntMathValue>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildZExt(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_int_z_extend_or_bit_cast<T: IntMathValue>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildZExtOrBitCast(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_int_truncate<T: IntMathValue>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildTrunc(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_int_truncate_or_bit_cast<T: IntMathValue>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildTruncOrBitCast(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_float_rem<T: FloatMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFRem(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Consolidate these two casts into one via subtypes
    pub fn build_float_to_unsigned_int<T: FloatMathValue>(&self, float: T, int_type: <T::BaseType as FloatMathType>::MathConvType, name: &str) -> <<T::BaseType as FloatMathType>::MathConvType as IntMathType>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPToUI(self.builder, float.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as FloatMathType>::MathConvType as IntMathType>::ValueType::new(value)
    }

    pub fn build_float_to_signed_int<T: FloatMathValue>(&self, float: T, int_type: <T::BaseType as FloatMathType>::MathConvType, name: &str) -> <<T::BaseType as FloatMathType>::MathConvType as IntMathType>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPToSI(self.builder, float.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as FloatMathType>::MathConvType as IntMathType>::ValueType::new(value)
    }

    // REVIEW: Consolidate these two casts into one via subtypes
    pub fn build_unsigned_int_to_float<T: IntMathValue>(&self, int: T, float_type: <T::BaseType as IntMathType>::MathConvType, name: &str) -> <<T::BaseType as IntMathType>::MathConvType as FloatMathType>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildUIToFP(self.builder, int.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as IntMathType>::MathConvType as FloatMathType>::ValueType::new(value)
    }

    pub fn build_signed_int_to_float<T: IntMathValue>(&self, int: T, float_type: <T::BaseType as IntMathType>::MathConvType, name: &str) -> <<T::BaseType as IntMathType>::MathConvType as FloatMathType>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSIToFP(self.builder, int.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as IntMathType>::MathConvType as FloatMathType>::ValueType::new(value)
    }

    pub fn build_float_trunc<T: FloatMathValue>(&self, float: T, float_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPTrunc(self.builder, float.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_float_ext<T: FloatMathValue>(&self, float: T, float_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPExt(self.builder, float.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_float_cast<T: FloatMathValue>(&self, float: T, float_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPCast(self.builder, float.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <L, R>(&self, lhs: &IntValue<L>, rhs: &IntType<R>, name: &str) -> IntValue<R> {
    pub fn build_int_cast<T: IntMathValue>(&self, int: T, int_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildIntCast(self.builder, int.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_float_div<T: FloatMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_add<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_add via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nsw_add<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_add via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nuw_add<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <F>(&self, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name: &str) -> FloatValue<F> {
    pub fn build_float_add<T: FloatMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: (&self, lhs: &IntValue<bool>, rhs: &IntValue<bool>, name: &str) -> IntValue<bool> {
    pub fn build_xor<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildXor(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: (&self, lhs: &IntValue<bool>, rhs: &IntValue<bool>, name: &str) -> IntValue<bool> {
    pub fn build_and<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAnd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: (&self, lhs: &IntValue<bool>, rhs: &IntValue<bool>, name: &str) -> IntValue<bool> {
    pub fn build_or<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildOr(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
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
    /// let fn_type = i8_type.fn_type(&[i8_type.into(), i8_type.into()], false);
    ///
    /// // Function Definition
    /// let function = module.add_function("left_shift", fn_type, None);
    /// let value = function.get_first_param().unwrap().into_int_value();
    /// let n = function.get_nth_param(1).unwrap().into_int_value();
    /// let entry_block = function.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_block);
    ///
    /// let shift = builder.build_left_shift(value, n, "left_shift"); // value << n
    ///
    /// builder.build_return(Some(&shift));
    /// ```
    pub fn build_left_shift<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildShl(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
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
    /// let fn_type = i8_type.fn_type(&[i8_type.into(), i8_type.into()], false);
    ///
    /// // Function Definition
    /// let function = module.add_function("right_shift", fn_type, None);
    /// let value = function.get_first_param().unwrap().into_int_value();
    /// let n = function.get_nth_param(1).unwrap().into_int_value();
    /// let entry_block = function.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_block);
    ///
    /// // Whether or not your right shift is sign extended (true) or logical (false) depends
    /// // on the boolean input parameter:
    /// let shift = builder.build_right_shift(value, n, false, "right_shift"); // value >> n
    ///
    /// builder.build_return(Some(&shift));
    /// ```
    pub fn build_right_shift<T: IntMathValue>(&self, lhs: T, rhs: T, sign_extend: bool, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            if sign_extend {
                LLVMBuildAShr(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
            } else {
                LLVMBuildLShr(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
            }
        };

        T::new(value)
    }

    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_sub<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_sub via flag param
    pub fn build_int_nsw_sub<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_sub via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nuw_sub<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <F>(&self, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name: &str) -> FloatValue<F> {
    pub fn build_float_sub<T: FloatMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_mul<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_mul via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nsw_mul<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_mul via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nuw_mul<T: IntMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <F>(&self, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name: &str) -> FloatValue<F> {
    pub fn build_float_mul<T: FloatMathValue>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_cast<T: BasicType, V: BasicValue>(&self, op: InstructionOpcode, from_value: V, to_type: T, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildCast(self.builder, op.as_llvm_opcode(), from_value.as_value_ref(), to_type.as_type_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    // SubType: <F, T>(&self, from: &PointerValue<F>, to: &PointerType<T>, name: &str) -> PointerValue<T> {
    pub fn build_pointer_cast<T: PointerMathValue>(&self, from: T, to: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPointerCast(self.builder, from.as_value_ref(), to.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <I>(&self, op, lhs: &IntValue<I>, rhs: &IntValue<I>, name) -> IntValue<bool> { ?
    // Note: we need a way to get an appropriate return type, since this method's return value
    // is always a bool (or vector of bools), not necessarily the same as the input value
    // See https://github.com/TheDan64/inkwell/pull/47#discussion_r197599297
    pub fn build_int_compare<T: IntMathValue>(&self, op: IntPredicate, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildICmp(self.builder, op.as_llvm_enum(), lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <F>(&self, op, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name) -> IntValue<bool> { ?
    // Note: see comment on build_int_compare regarding return value type
    pub fn build_float_compare<T: FloatMathValue>(&self, op: FloatPredicate, lhs: T, rhs: T, name: &str) -> <<T::BaseType as FloatMathType>::MathConvType as IntMathType>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFCmp(self.builder, op.as_llvm_enum(), lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        <<T::BaseType as FloatMathType>::MathConvType as IntMathType>::ValueType::new(value)
    }

    pub fn build_unconditional_branch(&self, destination_block: &BasicBlock) -> InstructionValue {
        let value = unsafe {
            LLVMBuildBr(self.builder, destination_block.basic_block)
        };

        InstructionValue::new(value)
    }

    pub fn build_conditional_branch(&self, comparison: IntValue, then_block: &BasicBlock, else_block: &BasicBlock) -> InstructionValue {
        let value = unsafe {
            LLVMBuildCondBr(self.builder, comparison.as_value_ref(), then_block.basic_block, else_block.basic_block)
        };

        InstructionValue::new(value)
    }

    pub fn build_indirect_branch<BV: BasicValue>(&self, address: BV, destinations: &[&BasicBlock]) -> InstructionValue {
        let value = unsafe {
            LLVMBuildIndirectBr(self.builder, address.as_value_ref(), destinations.len() as u32)
        };

        for destination in destinations {
            unsafe {
                LLVMAddDestination(value, destination.basic_block)
            }
        }

        InstructionValue::new(value)
    }

    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<I> {
    pub fn build_int_neg<T: IntMathValue>(&self, value: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_neg via flag and subtypes
    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<I> {
    pub fn build_int_nsw_neg<T: IntMathValue>(&self, value: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<I> {
    pub fn build_int_nuw_neg<T: IntMathValue>(&self, value: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <F>(&self, value: &FloatValue<F>, name) -> FloatValue<F> {
    pub fn build_float_neg<T: FloatMathValue>(&self, value: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<bool> { ?
    pub fn build_not<T: IntMathValue>(&self, value: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNot(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
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

    /// Builds an extract value instruction which extracts a `BasicValueEnum`
    /// from a struct or array.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("av");
    /// let void_type = context.void_type();
    /// let f32_type = context.f32_type();
    /// let i32_type = context.i32_type();
    /// let struct_type = context.struct_type(&[i32_type.into(), f32_type.into()], false);
    /// let array_type = i32_type.array_type(3);
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("av_fn", fn_type, None);
    /// let builder = context.create_builder();
    /// let entry = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry);
    ///
    /// let array_alloca = builder.build_alloca(array_type, "array_alloca");
    /// let array = builder.build_load(array_alloca, "array_load").into_array_value();
    /// let const_int1 = i32_type.const_int(2, false);
    /// let const_int2 = i32_type.const_int(5, false);
    /// let const_int3 = i32_type.const_int(6, false);
    ///
    /// assert!(builder.build_insert_value(array, const_int1, 0, "insert").is_some());
    /// assert!(builder.build_insert_value(array, const_int2, 1, "insert").is_some());
    /// assert!(builder.build_insert_value(array, const_int3, 2, "insert").is_some());
    /// assert!(builder.build_insert_value(array, const_int3, 3, "insert").is_none());
    ///
    /// assert!(builder.build_extract_value(array, 0, "extract").unwrap().is_int_value());
    /// assert!(builder.build_extract_value(array, 1, "extract").unwrap().is_int_value());
    /// assert!(builder.build_extract_value(array, 2, "extract").unwrap().is_int_value());
    /// assert!(builder.build_extract_value(array, 3, "extract").is_none());
    /// ```
    pub fn build_extract_value<AV: AggregateValue>(&self, agg: AV, index: u32, name: &str) -> Option<BasicValueEnum> {
        let size = match agg.as_aggregate_value_enum() {
            AggregateValueEnum::ArrayValue(av) => av.get_type().len(),
            AggregateValueEnum::StructValue(sv) => sv.get_type().count_fields(),
        };

        if index >= size {
            return None;
        }

        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExtractValue(self.builder, agg.as_value_ref(), index, c_string.as_ptr())
        };

        Some(BasicValueEnum::new(value))
    }

    /// Builds an insert value instruction which inserts a `BasicValue` into a struct
    /// or array.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("av");
    /// let void_type = context.void_type();
    /// let f32_type = context.f32_type();
    /// let i32_type = context.i32_type();
    /// let struct_type = context.struct_type(&[i32_type.into(), f32_type.into()], false);
    /// let array_type = i32_type.array_type(3);
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("av_fn", fn_type, None);
    /// let builder = context.create_builder();
    /// let entry = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry);
    ///
    /// let array_alloca = builder.build_alloca(array_type, "array_alloca");
    /// let array = builder.build_load(array_alloca, "array_load").into_array_value();
    /// let const_int1 = i32_type.const_int(2, false);
    /// let const_int2 = i32_type.const_int(5, false);
    /// let const_int3 = i32_type.const_int(6, false);
    ///
    /// assert!(builder.build_insert_value(array, const_int1, 0, "insert").is_some());
    /// assert!(builder.build_insert_value(array, const_int2, 1, "insert").is_some());
    /// assert!(builder.build_insert_value(array, const_int3, 2, "insert").is_some());
    /// assert!(builder.build_insert_value(array, const_int3, 3, "insert").is_none());
    /// ```
    pub fn build_insert_value<AV, BV>(&self, agg: AV, value: BV, index: u32, name: &str) -> Option<InstructionValue>
    where
        AV: AggregateValue,
        BV: BasicValue,
    {
        let size = match agg.as_aggregate_value_enum() {
            AggregateValueEnum::ArrayValue(av) => av.get_type().len(),
            AggregateValueEnum::StructValue(sv) => sv.get_type().count_fields(),
        };

        if index >= size {
            return None;
        }

        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildInsertValue(self.builder, agg.as_value_ref(), value.as_value_ref(), index, c_string.as_ptr())
        };

        Some(InstructionValue::new(value))
    }

    pub fn build_extract_element(&self, vector: VectorValue, index: IntValue, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExtractElement(self.builder, vector.as_value_ref(), index.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    pub fn build_insert_element<V: BasicValue>(&self, vector: VectorValue, element: V, index: IntValue, name: &str) -> BasicValueEnum {
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
    pub fn build_fence(&self, atomic_ordering: AtomicOrdering, num: i32, name: &str) -> InstructionValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildFence(self.builder, atomic_ordering.as_llvm_enum(), num, c_string.as_ptr())
        };

        InstructionValue::new(val)
    }

    // SubType: <P>(&self, ptr: &PointerValue<P>, name) -> IntValue<bool> {
    pub fn build_is_null<T: PointerMathValue>(&self, ptr: T, name: &str) -> <<T::BaseType as PointerMathType>::PtrConvType as IntMathType>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNull(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        <<T::BaseType as PointerMathType>::PtrConvType as IntMathType>::ValueType::new(val)
    }

    // SubType: <P>(&self, ptr: &PointerValue<P>, name) -> IntValue<bool> {
    pub fn build_is_not_null<T: PointerMathValue>(&self, ptr: T, name: &str) -> <<T::BaseType as PointerMathType>::PtrConvType as IntMathType>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNotNull(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        <<T::BaseType as PointerMathType>::PtrConvType as IntMathType>::ValueType::new(val)
    }

    // SubType: <I, P>(&self, int: &IntValue<I>, ptr_type: &PointerType<P>, name) -> PointerValue<P> {
    pub fn build_int_to_ptr<T: IntMathValue>(&self, int: T, ptr_type: <T::BaseType as IntMathType>::PtrConvType, name: &str) -> <<T::BaseType as IntMathType>::PtrConvType as PointerMathType>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildIntToPtr(self.builder, int.as_value_ref(), ptr_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as IntMathType>::PtrConvType as PointerMathType>::ValueType::new(value)
    }

    // SubType: <I, P>(&self, ptr: &PointerValue<P>, int_type: &IntType<I>, name) -> IntValue<I> {
    pub fn build_ptr_to_int<T: PointerMathValue>(&self, ptr: T, int_type: <T::BaseType as PointerMathType>::PtrConvType, name: &str) -> <<T::BaseType as PointerMathType>::PtrConvType as IntMathType>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPtrToInt(self.builder, ptr.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as PointerMathType>::PtrConvType as IntMathType>::ValueType::new(value)
    }

    pub fn clear_insertion_position(&self) {
        unsafe {
            LLVMClearInsertionPosition(self.builder)
        }
    }

    // REVIEW: Returning InstructionValue is the safe move here; but if the value means something
    // (IE the result of the switch) it should probably return BasicValueEnum?
    // SubTypes: I think value and case values must be the same subtype (maybe). Case value might need to be constants
    pub fn build_switch(&self, value: IntValue, else_block: &BasicBlock, cases: &[(IntValue, &BasicBlock)]) -> InstructionValue {
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

    // SubTypes: condition can only be IntValue<bool> or VectorValue<IntValue<Bool>>
    pub fn build_select<BV: BasicValue, IMV: IntMathValue>(&self, condition: IMV, then: BV, else_: BV, name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = unsafe {
            LLVMBuildSelect(self.builder, condition.as_value_ref(), then.as_value_ref(), else_.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    // The unsafety of this function should be fixable with subtypes. See GH #32
    pub unsafe fn build_global_string(&self, value: &str, name: &str) -> GlobalValue {
        let c_string_value = CString::new(value).expect("Conversion to CString failed unexpectedly");
        let c_string_name = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = LLVMBuildGlobalString(self.builder, c_string_value.as_ptr(), c_string_name.as_ptr());

        GlobalValue::new(value)
    }

    // REVIEW: Does this similar fn have the same issue build_global_string does? If so, mark as unsafe
    // and fix with subtypes.
    pub fn build_global_string_ptr(&self, value: &str, name: &str) -> GlobalValue {
        let c_string_value = CString::new(value).expect("Conversion to CString failed unexpectedly");
        let c_string_name = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = unsafe {
            LLVMBuildGlobalStringPtr(self.builder, c_string_value.as_ptr(), c_string_name.as_ptr())
        };

        GlobalValue::new(value)
    }

    // REVIEW: Do we need to constain types here? subtypes?
    pub fn build_shuffle_vector(&self, left: VectorValue, right: VectorValue, mask: VectorValue, name: &str) -> VectorValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = unsafe {
            LLVMBuildShuffleVector(self.builder, left.as_value_ref(), right.as_value_ref(), mask.as_value_ref(), c_string.as_ptr())
        };

        VectorValue::new(value)
    }

    // REVIEW: Is return type correct?
    // SubTypes: I think this should be type: BT -> BT::Value
    // https://llvm.org/docs/LangRef.html#i-va-arg
    pub fn build_va_arg<BT: BasicType>(&self, list: PointerValue, type_: BT , name: &str) -> BasicValueEnum {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildVAArg(self.builder, list.as_value_ref(), type_.as_type_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
        }
    }
}

type FunctionOrPointerValue = Either<FunctionValue, PointerValue>;

impl Into<FunctionOrPointerValue> for FunctionValue {
    fn into(self) -> FunctionOrPointerValue {
        Left(self)
    }
}

impl Into<FunctionOrPointerValue> for PointerValue {
    fn into(self) -> FunctionOrPointerValue {
        Right(self)
    }
}
