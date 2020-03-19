//! A `Builder` enables you to build instructions.

use either::{Either, Left, Right};
use llvm_sys::core::{LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildArrayAlloca, LLVMBuildArrayMalloc, LLVMBuildAtomicRMW, LLVMBuildBr, LLVMBuildCall, LLVMBuildCast, LLVMBuildCondBr, LLVMBuildExtractValue, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFence, LLVMBuildFMul, LLVMBuildFNeg, LLVMBuildFree, LLVMBuildFSub, LLVMBuildGEP, LLVMBuildICmp, LLVMBuildInsertValue, LLVMBuildIsNotNull, LLVMBuildIsNull, LLVMBuildLoad, LLVMBuildMalloc, LLVMBuildMemCpy, LLVMBuildMemMove, LLVMBuildMul, LLVMBuildNeg, LLVMBuildNot, LLVMBuildOr, LLVMBuildPhi, LLVMBuildPointerCast, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildSub, LLVMBuildUDiv, LLVMBuildUnreachable, LLVMBuildXor, LLVMDisposeBuilder, LLVMGetElementType, LLVMGetInsertBlock, LLVMGetReturnType, LLVMGetTypeKind, LLVMInsertIntoBuilder, LLVMPositionBuilderAtEnd, LLVMTypeOf, LLVMBuildExtractElement, LLVMBuildInsertElement, LLVMBuildIntToPtr, LLVMBuildPtrToInt, LLVMInsertIntoBuilderWithName, LLVMClearInsertionPosition, LLVMPositionBuilder, LLVMPositionBuilderBefore, LLVMBuildAggregateRet, LLVMBuildStructGEP, LLVMBuildInBoundsGEP, LLVMBuildPtrDiff, LLVMBuildNSWAdd, LLVMBuildNUWAdd, LLVMBuildNSWSub, LLVMBuildNUWSub, LLVMBuildNSWMul, LLVMBuildNUWMul, LLVMBuildSDiv, LLVMBuildSRem, LLVMBuildURem, LLVMBuildFRem, LLVMBuildNSWNeg, LLVMBuildNUWNeg, LLVMBuildFPToUI, LLVMBuildFPToSI, LLVMBuildSIToFP, LLVMBuildUIToFP, LLVMBuildFPTrunc, LLVMBuildFPExt, LLVMBuildIntCast, LLVMBuildFPCast, LLVMBuildSExtOrBitCast, LLVMBuildZExtOrBitCast, LLVMBuildTruncOrBitCast, LLVMBuildSwitch, LLVMAddCase, LLVMBuildShl, LLVMBuildAShr, LLVMBuildLShr, LLVMBuildGlobalString, LLVMBuildGlobalStringPtr, LLVMBuildExactSDiv, LLVMBuildTrunc, LLVMBuildSExt, LLVMBuildZExt, LLVMBuildSelect, LLVMBuildAddrSpaceCast, LLVMBuildBitCast, LLVMBuildShuffleVector, LLVMBuildVAArg, LLVMBuildIndirectBr, LLVMAddDestination};
#[llvm_versions(3.9..=latest)]
use llvm_sys::core::LLVMBuildAtomicCmpXchg;
use llvm_sys::prelude::{LLVMBuilderRef, LLVMValueRef};
use llvm_sys::{LLVMTypeKind};

use crate::{AtomicOrdering, AtomicRMWBinOp, IntPredicate, FloatPredicate};
use crate::basic_block::BasicBlock;
use crate::values::{AggregateValue, AggregateValueEnum, AsValueRef, BasicValue, BasicValueEnum, PhiValue, FunctionValue, IntValue, PointerValue, VectorValue, InstructionValue, GlobalValue, IntMathValue, FloatMathValue, PointerMathValue, InstructionOpcode, CallSiteValue};
#[llvm_versions(3.9..=latest)]
use crate::values::StructValue;
use crate::types::{AsTypeRef, BasicType, IntMathType, FloatMathType, PointerType, PointerMathType};

use std::ffi::CString;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Builder<'ctx> {
    builder: LLVMBuilderRef,
    _marker: PhantomData<&'ctx ()>,
}

impl<'ctx> Builder<'ctx> {
    pub(crate) fn new(builder: LLVMBuilderRef) -> Self {
        debug_assert!(!builder.is_null());

        Builder {
            builder: builder,
            _marker: PhantomData,
        }
    }

    // REVIEW: Would probably make this API a bit simpler by taking Into<Option<&BasicValue>>
    // So that you could just do build_return(&value) or build_return(None). Is that frowned upon?
    /// Builds a function return instruction. It should be provided with `None` if the return type
    /// is void otherwise `Some(&value)` should be provided.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// // A simple function which returns its argument:
    /// let context = Context::create();
    /// let module = context.create_module("ret");
    /// let builder = context.create_builder();
    /// let i32_type = context.i32_type();
    /// let arg_types = [i32_type.into()];
    /// let fn_type = i32_type.fn_type(&arg_types, false);
    /// let fn_value = module.add_function("ret", fn_type, None);
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// let i32_arg = fn_value.get_first_param().unwrap();
    ///
    /// builder.position_at_end(entry);
    /// builder.build_return(Some(&i32_arg));
    /// ```
    pub fn build_return(&self, value: Option<&dyn BasicValue<'ctx>>) -> InstructionValue<'ctx> {
        let value = unsafe {
            value.map_or_else(|| LLVMBuildRetVoid(self.builder), |value| LLVMBuildRet(self.builder, value.as_value_ref()))
        };

        InstructionValue::new(value)
    }

    /// Builds a function return instruction for a return type which is an aggregate type (ie structs and arrays).
    /// It is not necessary to use this over `build_return` but may be more convenient to use.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// // This builds a simple function which returns a struct (tuple) of two ints.
    /// let context = Context::create();
    /// let module = context.create_module("ret");
    /// let builder = context.create_builder();
    /// let i32_type = context.i32_type();
    /// let i32_three = i32_type.const_int(3, false);
    /// let i32_seven = i32_type.const_int(7, false);
    /// let struct_type = context.struct_type(&[i32_type.into(), i32_type.into()], false);
    /// let fn_type = struct_type.fn_type(&[], false);
    /// let fn_value = module.add_function("ret", fn_type, None);
    /// let entry = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry);
    /// builder.build_aggregate_return(&[i32_three.into(), i32_seven.into()]);
    /// ```
    pub fn build_aggregate_return(&self, values: &[BasicValueEnum<'ctx>]) -> InstructionValue<'ctx> {
        let mut args: Vec<LLVMValueRef> = values.iter()
                                                .map(|val| val.as_value_ref())
                                                .collect();
        let value = unsafe {
            LLVMBuildAggregateRet(self.builder, args.as_mut_ptr(), args.len() as u32)
        };

        InstructionValue::new(value)
    }

    /// Builds a function call instruction. It can take either a `FunctionValue` or a `PointerValue`
    /// which is a function pointer. It will panic if the `PointerValue` is not a function pointer.
    /// This may be turned into a Result in the future, however.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// // A simple function which calls itself:
    /// let context = Context::create();
    /// let module = context.create_module("ret");
    /// let builder = context.create_builder();
    /// let i32_type = context.i32_type();
    /// let fn_type = i32_type.fn_type(&[i32_type.into()], false);
    /// let fn_value = module.add_function("ret", fn_type, None);
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// let i32_arg = fn_value.get_first_param().unwrap();
    ///
    /// builder.position_at_end(entry);
    ///
    /// let ret_val = builder.build_call(fn_value, &[i32_arg], "call")
    ///     .try_as_basic_value()
    ///     .left()
    ///     .unwrap();
    ///
    /// builder.build_return(Some(&ret_val));
    /// ```
    pub fn build_call<F>(&self, function: F, args: &[BasicValueEnum<'ctx>], name: &str) -> CallSiteValue<'ctx>
    where
        F: Into<FunctionOrPointerValue<'ctx>>,
    {
        let fn_val_ref = match function.into() {
            Left(val) => val.as_value_ref(),
            Right(val) => {
                // If using a pointer value, we must validate it's a valid function ptr
                let value_ref = val.as_value_ref();
                let ty_kind = unsafe { LLVMGetTypeKind(LLVMGetElementType(LLVMTypeOf(value_ref))) };
                let is_a_fn_ptr = match ty_kind {
                    LLVMTypeKind::LLVMFunctionTypeKind => true,
                    _ => false,
                };

                // REVIEW: We should probably turn this into a Result?
                assert!(is_a_fn_ptr, "build_call called with a pointer which is not a function pointer");

                value_ref
            },
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
    pub unsafe fn build_gep(&self, ptr: PointerValue<'ctx>, ordered_indexes: &[IntValue<'ctx>], name: &str) -> PointerValue<'ctx> {
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
    pub unsafe fn build_in_bounds_gep(&self, ptr: PointerValue<'ctx>, ordered_indexes: &[IntValue<'ctx>], name: &str) -> PointerValue<'ctx> {
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
    pub unsafe fn build_struct_gep(&self, ptr: PointerValue<'ctx>, index: u32, name: &str) -> PointerValue<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = LLVMBuildStructGEP(self.builder, ptr.as_value_ref(), index, c_string.as_ptr());

        PointerValue::new(value)
    }

    /// Builds an instruction which calculates the difference of two pointers.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// // Builds a function which diffs two pointers
    /// let context = Context::create();
    /// let module = context.create_module("ret");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let i32_type = context.i32_type();
    /// let i32_ptr_type = i32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = void_type.fn_type(&[i32_ptr_type.into(), i32_ptr_type.into()], false);
    /// let fn_value = module.add_function("ret", fn_type, None);
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// let i32_ptr_param1 = fn_value.get_first_param().unwrap().into_pointer_value();
    /// let i32_ptr_param2 = fn_value.get_nth_param(1).unwrap().into_pointer_value();
    ///
    /// builder.position_at_end(entry);
    /// builder.build_ptr_diff(i32_ptr_param1, i32_ptr_param2, "diff");
    /// builder.build_return(None);
    /// ```
    pub fn build_ptr_diff(&self, lhs_ptr: PointerValue<'ctx>, rhs_ptr: PointerValue<'ctx>, name: &str) -> IntValue<'ctx> {
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
    pub fn build_phi<T: BasicType<'ctx>>(&self, type_: T, name: &str) -> PhiValue<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPhi(self.builder, type_.as_type_ref(), c_string.as_ptr())
        };

        PhiValue::new(value)
    }

    /// Builds a store instruction. It allows you to store a value of type `T` in a pointer to a type `T`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// // Builds a function which takes an i32 pointer and stores a 7 in it.
    /// let context = Context::create();
    /// let module = context.create_module("ret");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let i32_type = context.i32_type();
    /// let i32_ptr_type = i32_type.ptr_type(AddressSpace::Generic);
    /// let i32_seven = i32_type.const_int(7, false);
    /// let fn_type = void_type.fn_type(&[i32_ptr_type.into()], false);
    /// let fn_value = module.add_function("ret", fn_type, None);
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// let i32_ptr_param = fn_value.get_first_param().unwrap().into_pointer_value();
    ///
    /// builder.position_at_end(entry);
    /// builder.build_store(i32_ptr_param, i32_seven);
    /// builder.build_return(None);
    /// ```
    pub fn build_store<V: BasicValue<'ctx>>(&self, ptr: PointerValue<'ctx>, value: V) -> InstructionValue<'ctx> {
        let value = unsafe {
            LLVMBuildStore(self.builder, value.as_value_ref(), ptr.as_value_ref())
        };

        InstructionValue::new(value)
    }

    /// Builds a load instruction. It allows you to retrieve a value of type `T` from a pointer to a type `T`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// // Builds a function which takes an i32 pointer and returns the pointed at i32.
    /// let context = Context::create();
    /// let module = context.create_module("ret");
    /// let builder = context.create_builder();
    /// let i32_type = context.i32_type();
    /// let i32_ptr_type = i32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = i32_type.fn_type(&[i32_ptr_type.into()], false);
    /// let fn_value = module.add_function("ret", fn_type, None);
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// let i32_ptr_param = fn_value.get_first_param().unwrap().into_pointer_value();
    ///
    /// builder.position_at_end(entry);
    ///
    /// let pointee = builder.build_load(i32_ptr_param, "load");
    ///
    /// builder.build_return(Some(&pointee));
    /// ```
    pub fn build_load(&self, ptr: PointerValue<'ctx>, name: &str) -> BasicValueEnum<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildLoad(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    // TODOC: Stack allocation
    pub fn build_alloca<T: BasicType<'ctx>>(&self, ty: T, name: &str) -> PointerValue<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAlloca(self.builder, ty.as_type_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // TODOC: Stack allocation
    pub fn build_array_alloca<T: BasicType<'ctx>>(&self, ty: T, size: IntValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayAlloca(self.builder, ty.as_type_ref(), size.as_value_ref(), c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    /// Alignment arguments are specified in bytes, and should always be a power of 2.
    ///
    /// The final argument should be a pointer-sized integer.
    ///
    /// [`TargetData::ptr_sized_int_type_in_context`](https://thedan64.github.io/inkwell/inkwell/targets/struct.TargetData.html#method.ptr_sized_int_type_in_context) will get you one of those.
    pub fn build_memcpy<T: IntMathValue<'ctx>>(
        &self,
        dest: PointerValue<'ctx>,
        dest_align_bytes: u32,
        src: PointerValue<'ctx>,
        src_align_bytes: u32,
        size: T,
    ) -> Result<PointerValue<'ctx>, &'static str> {
        let value = unsafe {
            LLVMBuildMemCpy(
                self.builder,
                dest.as_value_ref(),
                dest_align_bytes,
                src.as_value_ref(),
                src_align_bytes,
                size.as_value_ref(),
            )
        };

        Ok(PointerValue::new(value))
    }

    /// Alignment arguments are specified in bytes, and should always be a power of 2.
    ///
    /// The final argument should be a pointer-sized integer.
    ///
    /// [`TargetData::ptr_sized_int_type_in_context`](https://thedan64.github.io/inkwell/inkwell/targets/struct.TargetData.html#method.ptr_sized_int_type_in_context) will get you one of those.
    pub fn build_memmove<T: IntMathValue<'ctx>>(
        &self,
        dest: PointerValue<'ctx>,
        dest_align_bytes: u32,
        src: PointerValue<'ctx>,
        src_align_bytes: u32,
        size: T,
    ) -> Result<PointerValue<'ctx>, &'static str> {
        let value = unsafe {
            LLVMBuildMemMove(
                self.builder,
                dest.as_value_ref(),
                dest_align_bytes,
                src.as_value_ref(),
                src_align_bytes,
                size.as_value_ref(),
            )
        };

        Ok(PointerValue::new(value))
    }

    // TODOC: Heap allocation
    pub fn build_malloc<T: BasicType<'ctx>>(&self, ty: T, name: &str) -> Result<PointerValue<'ctx>, &'static str> {
        // LLVMBulidMalloc segfaults if ty is unsized
        if !ty.is_sized() {
            return Err("Cannot build malloc call for an unsized type");
        }

        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMalloc(self.builder, ty.as_type_ref(), c_string.as_ptr())
        };

        Ok(PointerValue::new(value))
    }

    // TODOC: Heap allocation
    pub fn build_array_malloc<T: BasicType<'ctx>>(
        &self,
        ty: T,
        size: IntValue<'ctx>,
        name: &str
    ) -> Result<PointerValue<'ctx>, &'static str> {
        // LLVMBulidArrayMalloc segfaults if ty is unsized
        if !ty.is_sized() {
            return Err("Cannot build array malloc call for an unsized type");
        }

        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayMalloc(self.builder, ty.as_type_ref(), size.as_value_ref(), c_string.as_ptr())
        };

        Ok(PointerValue::new(value))
    }

    // SubType: <P>(&self, ptr: PointerValue<P>) -> InstructionValue {
    pub fn build_free(&self, ptr: PointerValue<'ctx>) -> InstructionValue<'ctx> {
        let val = unsafe {
            LLVMBuildFree(self.builder, ptr.as_value_ref())
        };

        InstructionValue::new(val)
    }

    pub fn insert_instruction(&self, instruction: &InstructionValue<'ctx>, name: Option<&str>) {
        match name {
            Some(name) => {
                let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

                unsafe {
                    LLVMInsertIntoBuilderWithName(self.builder, instruction.as_value_ref(), c_string.as_ptr())
                }
            },
            None => unsafe {
                LLVMInsertIntoBuilder(self.builder, instruction.as_value_ref());
            },
        }
    }

    pub fn get_insert_block(&self) -> Option<BasicBlock<'ctx>> {
        let bb = unsafe {
            LLVMGetInsertBlock(self.builder)
        };

        BasicBlock::new(bb)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I: IntSubType>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    //     if I::sign() == Unsigned { LLVMBuildUDiv() } else { LLVMBuildSDiv() }
    pub fn build_int_unsigned_div<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildUDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_signed_div<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_exact_signed_div<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExactSDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_unsigned_rem<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildURem(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }


    // TODO: Possibly make this generic over sign via struct metadata or subtypes
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_signed_rem<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSRem(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_int_s_extend<T: IntMathValue<'ctx>>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSExt(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Does this need vector support?
    pub fn build_address_space_cast(
        &self,
        ptr_val: PointerValue<'ctx>,
        ptr_type: PointerType<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
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
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// let i32_arg = fn_value.get_first_param().unwrap();
    ///
    /// builder.position_at_end(entry);
    ///
    /// builder.build_bitcast(i32_arg, f32_type, "i32tof32");
    /// builder.build_return(None);
    ///
    /// assert!(module.verify().is_ok());
    /// ```
    pub fn build_bitcast<T, V>(&self, val: V, ty: T, name: &str) -> BasicValueEnum<'ctx>
    where
        T: BasicType<'ctx>,
        V: BasicValue<'ctx>,
    {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildBitCast(self.builder, val.as_value_ref(), ty.as_type_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    pub fn build_int_s_extend_or_bit_cast<T: IntMathValue<'ctx>>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSExtOrBitCast(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_int_z_extend<T: IntMathValue<'ctx>>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildZExt(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_int_z_extend_or_bit_cast<T: IntMathValue<'ctx>>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildZExtOrBitCast(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_int_truncate<T: IntMathValue<'ctx>>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildTrunc(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_int_truncate_or_bit_cast<T: IntMathValue<'ctx>>(&self, int_value: T, int_type: T::BaseType, name: &str) -> T {
       let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

       let value = unsafe {
            LLVMBuildTruncOrBitCast(self.builder, int_value.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_float_rem<T: FloatMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFRem(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Consolidate these two casts into one via subtypes
    pub fn build_float_to_unsigned_int<T: FloatMathValue<'ctx>>(
        &self,
        float: T,
        int_type: <T::BaseType as FloatMathType<'ctx>>::MathConvType,
        name: &str,
    ) -> <<T::BaseType as FloatMathType<'ctx>>::MathConvType as IntMathType<'ctx>>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPToUI(self.builder, float.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as FloatMathType>::MathConvType as IntMathType>::ValueType::new(value)
    }

    pub fn build_float_to_signed_int<T: FloatMathValue<'ctx>>(
        &self,
        float: T,
        int_type: <T::BaseType as FloatMathType<'ctx>>::MathConvType,
        name: &str,
    ) -> <<T::BaseType as FloatMathType<'ctx>>::MathConvType as IntMathType<'ctx>>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPToSI(self.builder, float.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as FloatMathType>::MathConvType as IntMathType>::ValueType::new(value)
    }

    // REVIEW: Consolidate these two casts into one via subtypes
    pub fn build_unsigned_int_to_float<T: IntMathValue<'ctx>>(
        &self,
        int: T,
        float_type: <T::BaseType as IntMathType<'ctx>>::MathConvType,
        name: &str,
    ) -> <<T::BaseType as IntMathType<'ctx>>::MathConvType as FloatMathType<'ctx>>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildUIToFP(self.builder, int.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as IntMathType>::MathConvType as FloatMathType>::ValueType::new(value)
    }

    pub fn build_signed_int_to_float<T: IntMathValue<'ctx>>(
        &self,
        int: T,
        float_type: <T::BaseType as IntMathType<'ctx>>::MathConvType,
        name: &str,
    ) -> <<T::BaseType as IntMathType<'ctx>>::MathConvType as FloatMathType<'ctx>>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSIToFP(self.builder, int.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as IntMathType>::MathConvType as FloatMathType>::ValueType::new(value)
    }

    pub fn build_float_trunc<T: FloatMathValue<'ctx>>(&self, float: T, float_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPTrunc(self.builder, float.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_float_ext<T: FloatMathValue<'ctx>>(&self, float: T, float_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPExt(self.builder, float.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_float_cast<T: FloatMathValue<'ctx>>(&self, float: T, float_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFPCast(self.builder, float.as_value_ref(), float_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <L, R>(&self, lhs: &IntValue<L>, rhs: &IntType<R>, name: &str) -> IntValue<R> {
    pub fn build_int_cast<T: IntMathValue<'ctx>>(&self, int: T, int_type: T::BaseType, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildIntCast(self.builder, int.as_value_ref(), int_type.as_type_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_float_div<T: FloatMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFDiv(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_add<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_add via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nsw_add<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_add via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nuw_add<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <F>(&self, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name: &str) -> FloatValue<F> {
    pub fn build_float_add<T: FloatMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFAdd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: (&self, lhs: &IntValue<bool>, rhs: &IntValue<bool>, name: &str) -> IntValue<bool> {
    pub fn build_xor<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildXor(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: (&self, lhs: &IntValue<bool>, rhs: &IntValue<bool>, name: &str) -> IntValue<bool> {
    pub fn build_and<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAnd(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: (&self, lhs: &IntValue<bool>, rhs: &IntValue<bool>, name: &str) -> IntValue<bool> {
    pub fn build_or<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
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
    /// let entry_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(entry_block);
    ///
    /// let shift = builder.build_left_shift(value, n, "left_shift"); // value << n
    ///
    /// builder.build_return(Some(&shift));
    /// ```
    pub fn build_left_shift<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
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
    /// //fix doc error about overflowing_literals
    /// //rendered rfc: https://github.com/rust-lang/rfcs/blob/master/text/2438-deny-integer-literal-overflow-lint.md
    /// //tracking issue: https://github.com/rust-lang/rust/issues/54502
    /// #![allow(overflowing_literals)]
    ///
    /// // Logical Right Shift
    /// assert_eq!(0b1100_0000u8 >> 2, 0b0011_0000);
    /// assert_eq!(0b0000_0010u8 >> 1, 0b0000_0001);
    /// assert_eq!(0b0000_1100u8 >> 2, 0b0000_0011);
    ///
    /// // Sign Extended Right Shift
    /// assert_eq!(0b0100_0000i8 >> 2, 0b0001_0000);
    /// assert_eq!(0b1110_0000u8 as i8 >> 1, 0b1111_0000u8 as i8);
    /// assert_eq!(0b1100_0000u8 as i8 >> 2, 0b1111_0000u8 as i8);
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
    /// let entry_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(entry_block);
    ///
    /// // Whether or not your right shift is sign extended (true) or logical (false) depends
    /// // on the boolean input parameter:
    /// let shift = builder.build_right_shift(value, n, false, "right_shift"); // value >> n
    ///
    /// builder.build_return(Some(&shift));
    /// ```
    pub fn build_right_shift<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, sign_extend: bool, name: &str) -> T {
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
    pub fn build_int_sub<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_sub via flag param
    pub fn build_int_nsw_sub<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_sub via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nuw_sub<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <F>(&self, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name: &str) -> FloatValue<F> {
    pub fn build_float_sub<T: FloatMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFSub(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_mul<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_mul via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nsw_mul<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_mul via flag param
    // SubType: <I>(&self, lhs: &IntValue<I>, rhs: &IntValue<I>, name: &str) -> IntValue<I> {
    pub fn build_int_nuw_mul<T: IntMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <F>(&self, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name: &str) -> FloatValue<F> {
    pub fn build_float_mul<T: FloatMathValue<'ctx>>(&self, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFMul(self.builder, lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    pub fn build_cast<T: BasicType<'ctx>, V: BasicValue<'ctx>>(
        &self,
        op: InstructionOpcode,
        from_value: V,
        to_type: T,
        name: &str,
    ) -> BasicValueEnum<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildCast(self.builder, op.into(), from_value.as_value_ref(), to_type.as_type_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    // SubType: <F, T>(&self, from: &PointerValue<F>, to: &PointerType<T>, name: &str) -> PointerValue<T> {
    pub fn build_pointer_cast<T: PointerMathValue<'ctx>>(&self, from: T, to: T::BaseType, name: &str) -> T {
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
    pub fn build_int_compare<T: IntMathValue<'ctx>>(&self, op: IntPredicate, lhs: T, rhs: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildICmp(self.builder, op.into(), lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <F>(&self, op, lhs: &FloatValue<F>, rhs: &FloatValue<F>, name) -> IntValue<bool> { ?
    // Note: see comment on build_int_compare regarding return value type
    pub fn build_float_compare<T: FloatMathValue<'ctx>>(
        &self,
        op: FloatPredicate,
        lhs: T,
        rhs: T,
        name: &str,
    ) -> <<T::BaseType as FloatMathType<'ctx>>::MathConvType as IntMathType<'ctx>>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFCmp(self.builder, op.into(), lhs.as_value_ref(), rhs.as_value_ref(), c_string.as_ptr())
        };

        <<T::BaseType as FloatMathType>::MathConvType as IntMathType>::ValueType::new(value)
    }

    pub fn build_unconditional_branch(&self, destination_block: BasicBlock<'ctx>) -> InstructionValue<'ctx> {
        let value = unsafe {
            LLVMBuildBr(self.builder, destination_block.basic_block)
        };

        InstructionValue::new(value)
    }

    pub fn build_conditional_branch(
        &self,
        comparison: IntValue<'ctx>,
        then_block: BasicBlock<'ctx>,
        else_block: BasicBlock<'ctx>,
    ) -> InstructionValue<'ctx> {
        let value = unsafe {
            LLVMBuildCondBr(self.builder, comparison.as_value_ref(), then_block.basic_block, else_block.basic_block)
        };

        InstructionValue::new(value)
    }

    pub fn build_indirect_branch<BV: BasicValue<'ctx>>(
        &self,
        address: BV,
        destinations: &[BasicBlock<'ctx>],
    ) -> InstructionValue<'ctx> {
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
    pub fn build_int_neg<T: IntMathValue<'ctx>>(&self, value: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: Possibly incorperate into build_int_neg via flag and subtypes
    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<I> {
    pub fn build_int_nsw_neg<T: IntMathValue<'ctx>>(&self, value: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNSWNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<I> {
    pub fn build_int_nuw_neg<T: IntMathValue<'ctx>>(&self, value: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNUWNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <F>(&self, value: &FloatValue<F>, name) -> FloatValue<F> {
    pub fn build_float_neg<T: FloatMathValue<'ctx>>(&self, value: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFNeg(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // SubType: <I>(&self, value: &IntValue<I>, name) -> IntValue<bool> { ?
    pub fn build_not<T: IntMathValue<'ctx>>(&self, value: T, name: &str) -> T {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNot(self.builder, value.as_value_ref(), c_string.as_ptr())
        };

        T::new(value)
    }

    // REVIEW: What if instruction and basic_block are completely unrelated?
    // It'd be great if we could get the BB from the instruction behind the scenes
    pub fn position_at(&self, basic_block: BasicBlock<'ctx>, instruction: &InstructionValue<'ctx>) {
        unsafe {
            LLVMPositionBuilder(self.builder, basic_block.basic_block, instruction.as_value_ref())
        }
    }

    pub fn position_before(&self, instruction: &InstructionValue<'ctx>) {
        unsafe {
            LLVMPositionBuilderBefore(self.builder, instruction.as_value_ref())
        }
    }

    pub fn position_at_end(&self, basic_block: BasicBlock<'ctx>) {
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
    /// let entry = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry);
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
    pub fn build_extract_value<AV: AggregateValue<'ctx>>(
        &self,
        agg: AV,
        index: u32,
        name: &str,
    ) -> Option<BasicValueEnum<'ctx>> {
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
    /// or array and returns the resulting aggregate value.
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
    /// let entry = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry);
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
    pub fn build_insert_value<AV, BV>(&self, agg: AV, value: BV, index: u32, name: &str) -> Option<AggregateValueEnum<'ctx>>
    where
        AV: AggregateValue<'ctx>,
        BV: BasicValue<'ctx>,
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

        Some(AggregateValueEnum::new(value))
    }

    /// Builds an extract element instruction which extracts a `BasicValueEnum`
    /// from a vector.
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("av");
    /// let i32_type = context.i32_type();
    /// let i32_zero = i32_type.const_int(0, false);
    /// let vec_type = i32_type.vec_type(2);
    /// let fn_type = i32_type.fn_type(&[vec_type.into()], false);
    /// let fn_value = module.add_function("vec_fn", fn_type, None);
    /// let builder = context.create_builder();
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// let vector_param = fn_value.get_first_param().unwrap().into_vector_value();
    ///
    /// builder.position_at_end(entry);
    ///
    /// let extracted = builder.build_extract_element(vector_param, i32_zero, "insert");
    ///
    /// builder.build_return(Some(&extracted));
    /// ```
    pub fn build_extract_element(&self, vector: VectorValue<'ctx>, index: IntValue<'ctx>, name: &str) -> BasicValueEnum<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExtractElement(self.builder, vector.as_value_ref(), index.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    /// Builds an insert element instruction which inserts a `BasicValue` into a vector
    /// and returns the resulting vector.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("av");
    /// let void_type = context.void_type();
    /// let i32_type = context.i32_type();
    /// let i32_zero = i32_type.const_int(0, false);
    /// let i32_seven = i32_type.const_int(7, false);
    /// let vec_type = i32_type.vec_type(2);
    /// let fn_type = void_type.fn_type(&[vec_type.into()], false);
    /// let fn_value = module.add_function("vec_fn", fn_type, None);
    /// let builder = context.create_builder();
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// let vector_param = fn_value.get_first_param().unwrap().into_vector_value();
    ///
    /// builder.position_at_end(entry);
    /// builder.build_insert_element(vector_param, i32_seven, i32_zero, "insert");
    /// builder.build_return(None);
    /// ```
    pub fn build_insert_element<V: BasicValue<'ctx>>(
        &self,
        vector: VectorValue<'ctx>,
        element: V,
        index: IntValue<'ctx>,
        name: &str,
    ) -> VectorValue<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildInsertElement(self.builder, vector.as_value_ref(), element.as_value_ref(), index.as_value_ref(), c_string.as_ptr())
        };

        VectorValue::new(value)
    }

    pub fn build_unreachable(&self) -> InstructionValue<'ctx> {
        let val = unsafe {
            LLVMBuildUnreachable(self.builder)
        };

        InstructionValue::new(val)
    }

    // REVIEW: Not sure if this should return InstructionValue or an actual value
    // TODO: Better name for num?
    pub fn build_fence(&self, atomic_ordering: AtomicOrdering, num: i32, name: &str) -> InstructionValue<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildFence(self.builder, atomic_ordering.into(), num, c_string.as_ptr())
        };

        InstructionValue::new(val)
    }

    // SubType: <P>(&self, ptr: &PointerValue<P>, name) -> IntValue<bool> {
    pub fn build_is_null<T: PointerMathValue<'ctx>>(&self, ptr: T, name: &str) -> <<T::BaseType as PointerMathType<'ctx>>::PtrConvType as IntMathType<'ctx>>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNull(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        <<T::BaseType as PointerMathType>::PtrConvType as IntMathType>::ValueType::new(val)
    }

    // SubType: <P>(&self, ptr: &PointerValue<P>, name) -> IntValue<bool> {
    pub fn build_is_not_null<T: PointerMathValue<'ctx>>(&self, ptr: T, name: &str) -> <<T::BaseType as PointerMathType<'ctx>>::PtrConvType as IntMathType<'ctx>>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNotNull(self.builder, ptr.as_value_ref(), c_string.as_ptr())
        };

        <<T::BaseType as PointerMathType>::PtrConvType as IntMathType>::ValueType::new(val)
    }

    // SubType: <I, P>(&self, int: &IntValue<I>, ptr_type: &PointerType<P>, name) -> PointerValue<P> {
    pub fn build_int_to_ptr<T: IntMathValue<'ctx>>(&self, int: T, ptr_type: <T::BaseType as IntMathType<'ctx>>::PtrConvType, name: &str) -> <<T::BaseType as IntMathType<'ctx>>::PtrConvType as PointerMathType<'ctx>>::ValueType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildIntToPtr(self.builder, int.as_value_ref(), ptr_type.as_type_ref(), c_string.as_ptr())
        };

        <<T::BaseType as IntMathType>::PtrConvType as PointerMathType>::ValueType::new(value)
    }

    // SubType: <I, P>(&self, ptr: &PointerValue<P>, int_type: &IntType<I>, name) -> IntValue<I> {
    pub fn build_ptr_to_int<T: PointerMathValue<'ctx>>(
        &self,
        ptr: T,
        int_type: <T::BaseType as PointerMathType<'ctx>>::PtrConvType,
        name: &str,
    ) -> <<T::BaseType as PointerMathType<'ctx>>::PtrConvType as IntMathType<'ctx>>::ValueType {
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
    pub fn build_switch(&self, value: IntValue<'ctx>, else_block: BasicBlock<'ctx>, cases: &[(IntValue<'ctx>, BasicBlock<'ctx>)]) -> InstructionValue<'ctx> {
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
    pub fn build_select<BV: BasicValue<'ctx>, IMV: IntMathValue<'ctx>>(&self, condition: IMV, then: BV, else_: BV, name: &str) -> BasicValueEnum<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = unsafe {
            LLVMBuildSelect(self.builder, condition.as_value_ref(), then.as_value_ref(), else_.as_value_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    // The unsafety of this function should be fixable with subtypes. See GH #32
    pub unsafe fn build_global_string(&self, value: &str, name: &str) -> GlobalValue<'ctx> {
        let c_string_value = CString::new(value).expect("Conversion to CString failed unexpectedly");
        let c_string_name = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = LLVMBuildGlobalString(self.builder, c_string_value.as_ptr(), c_string_name.as_ptr());

        GlobalValue::new(value)
    }

    // REVIEW: Does this similar fn have the same issue build_global_string does? If so, mark as unsafe
    // and fix with subtypes.
    pub fn build_global_string_ptr(&self, value: &str, name: &str) -> GlobalValue<'ctx> {
        let c_string_value = CString::new(value).expect("Conversion to CString failed unexpectedly");
        let c_string_name = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = unsafe {
            LLVMBuildGlobalStringPtr(self.builder, c_string_value.as_ptr(), c_string_name.as_ptr())
        };

        GlobalValue::new(value)
    }

    // REVIEW: Do we need to constrain types here? subtypes?
    pub fn build_shuffle_vector(&self, left: VectorValue<'ctx>, right: VectorValue<'ctx>, mask: VectorValue<'ctx>, name: &str) -> VectorValue<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = unsafe {
            LLVMBuildShuffleVector(self.builder, left.as_value_ref(), right.as_value_ref(), mask.as_value_ref(), c_string.as_ptr())
        };

        VectorValue::new(value)
    }

    // REVIEW: Is return type correct?
    // SubTypes: I think this should be type: BT -> BT::Value
    // https://llvm.org/docs/LangRef.html#i-va-arg
    pub fn build_va_arg<BT: BasicType<'ctx>>(&self, list: PointerValue<'ctx>, type_: BT, name: &str) -> BasicValueEnum<'ctx> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildVAArg(self.builder, list.as_value_ref(), type_.as_type_ref(), c_string.as_ptr())
        };

        BasicValueEnum::new(value)
    }

    /// Builds an atomicrmw instruction. It allows you to atomically modify memory.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::{AddressSpace, AtomicOrdering, AtomicRMWBinOp};
    /// let context = Context::create();
    /// let module = context.create_module("rmw");
    /// let void_type = context.void_type();
    /// let i32_type = context.i32_type();
    /// let i32_seven = i32_type.const_int(7, false);
    /// let i32_ptr_type = i32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = void_type.fn_type(&[i32_ptr_type.into()], false);
    /// let fn_value = module.add_function("rmw", fn_type, None);
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// let i32_ptr_param = fn_value.get_first_param().unwrap().into_pointer_value();
    /// let builder = context.create_builder();
    /// builder.position_at_end(entry);
    /// builder.build_atomicrmw(AtomicRMWBinOp::Add, i32_ptr_param, i32_seven, AtomicOrdering::Unordered);
    /// builder.build_return(None);
    /// ```
    // https://llvm.org/docs/LangRef.html#atomicrmw-instruction
    pub fn build_atomicrmw(
        &self,
        op: AtomicRMWBinOp,
        ptr: PointerValue<'ctx>,
        value: IntValue<'ctx>,
        ordering: AtomicOrdering,
    ) -> Result<IntValue<'ctx>, &'static str> {
        // TODO: add support for fadd, fsub and xchg on floating point types in LLVM 9+.

        // "The type of ‘<value>’ must be an integer type whose bit width is a power of two greater than or equal to eight and less than or equal to a target-specific size limit. The type of the ‘<pointer>’ operand must be a pointer to that type." -- https://releases.llvm.org/3.6.2/docs/LangRef.html#atomicrmw-instruction
        if value.get_type().get_bit_width() < 8 ||
           !value.get_type().get_bit_width().is_power_of_two() {
            return Err("The bitwidth of value must be a power of 2 and greater than 8.");
        }
        if ptr.get_type().get_element_type() != value.get_type().into() {
            return Err("Pointer's pointee type must match the value's type.");
        }

        let val = unsafe {
            LLVMBuildAtomicRMW(self.builder, op.into(), ptr.as_value_ref(), value.as_value_ref(), ordering.into(), false as i32)
        };

        Ok(IntValue::new(val))
    }

    /// Builds a cmpxchg instruction. It allows you to atomically compare and replace memory.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::{AddressSpace, AtomicOrdering};
    /// let context = Context::create();
    /// let module = context.create_module("cmpxchg");
    /// let void_type = context.void_type();
    /// let i32_type = context.i32_type();
    /// let i32_ptr_type = i32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = void_type.fn_type(&[i32_ptr_type.into()], false);
    /// let fn_value = module.add_function("", fn_type, None);
    /// let i32_ptr_param = fn_value.get_first_param().unwrap().into_pointer_value();
    /// let i32_seven = i32_type.const_int(7, false);
    /// let i32_eight = i32_type.const_int(8, false);
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// let builder = context.create_builder();
    /// builder.position_at_end(entry);
    /// builder.build_cmpxchg(i32_ptr_param, i32_seven, i32_eight, AtomicOrdering::AcquireRelease, AtomicOrdering::Monotonic);
    /// builder.build_return(None);
    /// ```
    // https://llvm.org/docs/LangRef.html#cmpxchg-instruction
    #[llvm_versions(3.9..=latest)]
    pub fn build_cmpxchg<V: BasicValue<'ctx>>(
        &self,
        ptr: PointerValue<'ctx>,
        cmp: V,
        new: V,
        success: AtomicOrdering,
        failure: AtomicOrdering,
    ) -> Result<StructValue<'ctx>, &'static str> {
        let cmp = cmp.as_basic_value_enum();
        let new = new.as_basic_value_enum();
        if cmp.get_type() != new.get_type() {
            return Err("The value to compare against and the value to replace with must have the same type.");
        }
        if !cmp.is_int_value() && !cmp.is_pointer_value() {
            return Err("The values must have pointer or integer type.");
        }
        if ptr.get_type().get_element_type().to_basic_type_enum() != cmp.get_type() {
            return Err("The pointer does not point to an element of the value type.");
        }

        // "Both ordering parameters must be at least monotonic, the ordering constraint on failure must be no stronger than that on success, and the failure ordering cannot be either release or acq_rel." -- https://llvm.org/docs/LangRef.html#cmpxchg-instruction
        if success < AtomicOrdering::Monotonic || failure < AtomicOrdering::Monotonic {
            return Err("Both success and failure orderings must be Monotonic or stronger.");
        }
        if failure > success {
            return Err("The failure ordering may not be stronger than the success ordering.");
        }
        if failure == AtomicOrdering::Release || failure == AtomicOrdering::AcquireRelease {
            return Err("The failure ordering may not be release or acquire release.");
        }

        let val = unsafe {
            LLVMBuildAtomicCmpXchg(self.builder, ptr.as_value_ref(), cmp.as_value_ref(), new.as_value_ref(), success.into(), failure.into(), false as i32)
        };

        Ok(StructValue::new(val).into())
    }
}

impl Drop for Builder<'_> {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
        }
    }
}

type FunctionOrPointerValue<'ctx> = Either<FunctionValue<'ctx>, PointerValue<'ctx>>;

impl<'ctx> Into<FunctionOrPointerValue<'ctx>> for FunctionValue<'ctx> {
    fn into(self) -> FunctionOrPointerValue<'ctx> {
        Left(self)
    }
}

impl<'ctx> Into<FunctionOrPointerValue<'ctx>> for PointerValue<'ctx> {
    fn into(self) -> FunctionOrPointerValue<'ctx> {
        Right(self)
    }
}
