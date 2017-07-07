use llvm_sys::core::{LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildArrayAlloca, LLVMBuildArrayMalloc, LLVMBuildBr, LLVMBuildCall, LLVMBuildCast, LLVMBuildCondBr, LLVMBuildExtractValue, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFence, LLVMBuildFMul, LLVMBuildFNeg, LLVMBuildFree, LLVMBuildFSub, LLVMBuildGEP, LLVMBuildICmp, LLVMBuildInsertValue, LLVMBuildIsNotNull, LLVMBuildIsNull, LLVMBuildLoad, LLVMBuildMalloc, LLVMBuildMul, LLVMBuildNeg, LLVMBuildNot, LLVMBuildOr, LLVMBuildPhi, LLVMBuildPointerCast, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildSub, LLVMBuildUDiv, LLVMBuildUnreachable, LLVMBuildXor, LLVMDisposeBuilder, LLVMGetElementType, LLVMGetInsertBlock, LLVMGetReturnType, LLVMGetTypeKind, LLVMInsertIntoBuilder, LLVMPositionBuilderAtEnd, LLVMTypeOf};
use llvm_sys::prelude::{LLVMBuilderRef, LLVMValueRef};
use llvm_sys::{LLVMOpcode, LLVMIntPredicate, LLVMTypeKind, LLVMRealPredicate, LLVMAtomicOrdering};

use basic_block::BasicBlock;
use values::{AnyValue, BasicValue, PhiValue, FunctionValue, FloatValue, IntValue, PointerValue, Value, IntoIntValue};
use types::AnyType;

use std::ffi::CString;
use std::mem::transmute;

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

    // Known acceptable return Values: IntValue, FloatValue
    pub fn build_return(&self, value: Option<&BasicValue>) -> Value {
        // let value = unsafe {
        //     value.map_or(LLVMBuildRetVoid(self.builder), |value| LLVMBuildRet(self.builder, value.value))
        // };

        let value = unsafe {
            match value {
                Some(v) => LLVMBuildRet(self.builder, v.as_ref().value),
                None => LLVMBuildRetVoid(self.builder),
            }
        };

        Value::new(value)
    }

    pub fn build_call<V: Into<Value> + Copy>(&self, function: &FunctionValue, args: &[V], name: &str) -> Value {
        // LLVM gets upset when void calls are named because they don't return anything
        let name = unsafe {
            match LLVMGetTypeKind(LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(function.fn_value.value)))) {
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
            LLVMBuildCall(self.builder, function.fn_value.value, args.as_mut_ptr(), args.len() as u32, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_gep(&self, ptr: &PointerValue, ordered_indexes: &[&IntoIntValue], name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter()
                                                                 .map(|val| val.into_int_value().int_value.value)
                                                                 .collect();
        let value = unsafe {
            LLVMBuildGEP(self.builder, ptr.as_ref().value, index_values.as_mut_ptr(), index_values.len() as u32, c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    pub fn build_phi(&self, type_: &AnyType, name: &str) -> PhiValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPhi(self.builder, type_.as_ref().type_, c_string.as_ptr())
        };

        PhiValue::new(value)
    }

    pub fn build_store(&self, value: &AnyValue, ptr: &PointerValue) -> Value {
        let value = unsafe {
            LLVMBuildStore(self.builder, value.as_ref().value, ptr.as_ref().value)
        };

        Value::new(value)
    }

    pub fn build_load(&self, ptr: &PointerValue, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildLoad(self.builder, ptr.as_ref().value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_stack_allocation(&self, type_: &AnyType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAlloca(self.builder, type_.as_ref().type_, c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    pub fn build_heap_allocation(&self, type_: &AnyType, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMalloc(self.builder, type_.as_ref().type_, c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // TODO: Rename to "build_heap_allocated_array" + stack version?
    // REVIEW: Is this still a PointerValue (as opposed to an ArrayValue?)
    pub fn build_array_heap_allocation<V: Into<Value> + Copy>(&self, type_: &AnyType, size: &V, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayMalloc(self.builder, type_.as_ref().type_, (*size).into().value, c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    // REVIEW: Is this still a PointerValue (as opposed to an ArrayValue?)
    pub fn build_stack_allocated_array<V: Into<Value> + Copy>(&self, type_: &AnyType, size: &V, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayAlloca(self.builder, type_.as_ref().type_, (*size).into().value, c_string.as_ptr())
        };

        PointerValue::new(value)
    }

    /// REVIEW: Untested
    pub fn build_free(&self, ptr: &PointerValue) -> Value { // REVIEW: Why does free return? Seems like original pointer? Ever useful?
        let val = unsafe {
            LLVMBuildFree(self.builder, ptr.as_ref().value)
        };

        Value::new(val)
    }

    pub fn insert_instruction(&self, value: &Value) {
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

    pub fn build_int_div(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        // TODO: Support signed, possibly as metadata on IntValue?
        let value = unsafe {
            LLVMBuildUDiv(self.builder, lhs.int_value.value, rhs.int_value.value, c_string.as_ptr())
        };

        IntValue::new(value)
    }

    pub fn build_float_div(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFDiv(self.builder, lhs.float_value.value, rhs.float_value.value, c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    pub fn build_int_add(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAdd(self.builder, lhs.int_value.value, rhs.int_value.value, c_string.as_ptr())
        };

        IntValue::new(value)
    }

    /// REVIEW: Untested
    pub fn build_float_add(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFAdd(self.builder, lhs.float_value.value, rhs.float_value.value, c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    /// REVIEW: Untested
    pub fn build_xor(&self, lhs: &Value, rhs: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildXor(self.builder, lhs.value, rhs.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_and(&self, lhs: &Value, rhs: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAnd(self.builder, lhs.value, rhs.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_or(&self, lhs: &Value, rhs: &Value, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildOr(self.builder, lhs.value, rhs.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn build_int_sub(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildSub(self.builder, lhs.int_value.value, rhs.int_value.value, c_string.as_ptr())
        };

        IntValue::new(value)
    }

    /// REVIEW: Untested
    pub fn build_float_sub(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFSub(self.builder, lhs.float_value.value, rhs.float_value.value, c_string.as_ptr())
        };

        FloatValue::new(value)
    }

    /// REVIEW: Untested
    pub fn build_int_mul(&self, lhs: &IntValue, rhs: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMul(self.builder, lhs.int_value.value, rhs.int_value.value, c_string.as_ptr())
        };

        IntValue::new(value)
    }


    /// REVIEW: Untested
    pub fn build_float_mul(&self, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFMul(self.builder, lhs.float_value.value, rhs.float_value.value, c_string.as_ptr())
        };

        FloatValue::new(value)
    }


    /// REVIEW: Untested
    pub fn build_cast(&self, op: LLVMOpcode, from_value: &AnyValue, to_type: &AnyType, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildCast(self.builder, op, from_value.as_ref().value, to_type.as_ref().type_, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_pointer_cast(&self, from: &Value, to: &AnyType, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPointerCast(self.builder, from.value, to.as_ref().type_, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_int_compare(&self, op: LLVMIntPredicate, lhs: &IntValue, rhs: &IntValue, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildICmp(self.builder, op, lhs.int_value.value, rhs.int_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_float_compare(&self, op: LLVMRealPredicate, lhs: &FloatValue, rhs: &FloatValue, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFCmp(self.builder, op, lhs.float_value.value, rhs.float_value.value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_unconditional_branch(&self, destination_block: &BasicBlock) -> Value {
        let value = unsafe {
            LLVMBuildBr(self.builder, destination_block.basic_block)
        };

        Value::new(value)
    }

    pub fn build_conditional_branch(&self, comparison: &IntValue, then_block: &BasicBlock, else_block: &BasicBlock) -> Value {
        let value = unsafe {
            LLVMBuildCondBr(self.builder, comparison.int_value.value, then_block.basic_block, else_block.basic_block)
        };

        Value::new(value)
    }

    /// REVIEW: Combine with float neg?
    /// REVIEW: Untested
    pub fn build_neg(&self, value: &IntValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildNeg(self.builder, value.int_value.value, c_string.as_ptr())
        };

        IntValue::new(value)
    }

    /// REVIEW: Combine with int neg?
    /// REVIEW: Untested
    pub fn build_float_neg(&self, value: &FloatValue, name: &str) -> FloatValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildFNeg(self.builder, value.float_value.value, c_string.as_ptr())
        };

        FloatValue::new(value)
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

    pub fn build_extract_value(&self, value: &Value, index: u32, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildExtractValue(self.builder, value.value, index, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_insert_value(&self, value: &Value, ptr: &PointerValue, index: u32, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildInsertValue(self.builder, value.value, ptr.ptr_value.value, index, c_string.as_ptr())
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

    // REVIEW: Untested
    pub fn build_is_null(&self, value: &PointerValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNull(self.builder, value.ptr_value.value, c_string.as_ptr())
        };

        IntValue::new(val)
    }

    // REVIEW: Untested
    pub fn build_is_not_null(&self, value: &PointerValue, name: &str) -> IntValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let val = unsafe {
            LLVMBuildIsNotNull(self.builder, value.ptr_value.value, c_string.as_ptr())
        };

        IntValue::new(val)
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
        }
    }
}
