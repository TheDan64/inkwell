use llvm_sys::core::{LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildArrayAlloca, LLVMBuildArrayMalloc, LLVMBuildBr, LLVMBuildCall, LLVMBuildCast, LLVMBuildCondBr, LLVMBuildExtractValue, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFence, LLVMBuildFMul, LLVMBuildFNeg, LLVMBuildFree, LLVMBuildFSub, LLVMBuildGEP, LLVMBuildICmp, LLVMBuildInsertValue, LLVMBuildIsNotNull, LLVMBuildIsNull, LLVMBuildLoad, LLVMBuildMalloc, LLVMBuildMul, LLVMBuildNeg, LLVMBuildNot, LLVMBuildOr, LLVMBuildPhi, LLVMBuildPointerCast, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildSub, LLVMBuildUDiv, LLVMBuildUnreachable, LLVMBuildXor, LLVMDisposeBuilder, LLVMGetElementType, LLVMGetInsertBlock, LLVMGetReturnType, LLVMGetTypeKind, LLVMInsertIntoBuilder, LLVMPositionBuilderAtEnd, LLVMTypeOf};
use llvm_sys::prelude::{LLVMBuilderRef, LLVMValueRef};
use llvm_sys::{LLVMOpcode, LLVMIntPredicate, LLVMTypeKind, LLVMRealPredicate, LLVMAtomicOrdering};

use basic_block::BasicBlock;
use values::{FunctionValue, IntValue, Value};
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

    pub fn build_call<V: Into<Value> + Copy>(&self, function: &FunctionValue, args: &[V], name: &str) -> Value {
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

    pub fn build_gep<V: Into<Value> + Copy>(&self, ptr: &Value, ordered_indexes: &[V], name: &str) -> Value {
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

    pub fn build_phi(&self, type_: &AnyType, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildPhi(self.builder, type_.as_ref().type_, c_string.as_ptr())
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

    pub fn build_stack_allocation(&self, type_: &AnyType, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildAlloca(self.builder, type_.as_ref().type_, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_heap_allocation(&self, type_: &AnyType, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildMalloc(self.builder, type_.as_ref().type_, c_string.as_ptr())
        };

        Value::new(value)
    }

    // TODO: Rename to "build_heap_allocated_aRray" + stack version?
    pub fn build_array_heap_allocation<V: Into<Value> + Copy>(&self, type_: &AnyType, size: &V, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayMalloc(self.builder, type_.as_ref().type_, (*size).into().value, c_string.as_ptr())
        };

        Value::new(value)
    }

    pub fn build_stack_allocated_array<V: Into<Value> + Copy>(&self, type_: &AnyType, size: &V, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildArrayAlloca(self.builder, type_.as_ref().type_, (*size).into().value, c_string.as_ptr())
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
    pub fn build_cast(&self, op: LLVMOpcode, from_value: &Value, to_type: &AnyType, name: &str) -> Value {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMBuildCast(self.builder, op, from_value.value, to_type.as_ref().type_, c_string.as_ptr())
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
