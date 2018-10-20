use llvm_sys::core::{LLVMConstNot, LLVMConstNeg, LLVMConstNSWNeg, LLVMConstNUWNeg, LLVMConstAdd, LLVMConstNSWAdd, LLVMConstNUWAdd, LLVMConstSub, LLVMConstNSWSub, LLVMConstNUWSub, LLVMConstMul, LLVMConstNSWMul, LLVMConstNUWMul, LLVMConstUDiv, LLVMConstSDiv, LLVMConstSRem, LLVMConstURem, LLVMConstIntCast, LLVMConstXor, LLVMConstOr, LLVMConstAnd, LLVMConstExactSDiv, LLVMConstShl, LLVMConstLShr, LLVMConstAShr, LLVMConstUIToFP, LLVMConstSIToFP, LLVMConstIntToPtr, LLVMConstTrunc, LLVMConstSExt, LLVMConstZExt, LLVMConstTruncOrBitCast, LLVMConstSExtOrBitCast, LLVMConstZExtOrBitCast, LLVMConstBitCast, LLVMConstICmp, LLVMConstIntGetZExtValue, LLVMConstIntGetSExtValue, LLVMConstSelect};
#[llvm_versions(4.0 => latest)]
use llvm_sys::core::LLVMConstExactUDiv;
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use IntPredicate;
use support::LLVMString;
use types::{AsTypeRef, FloatType, PointerType, IntType};
use values::traits::AsValueRef;
use values::{BasicValue, BasicValueEnum, FloatValue, InstructionValue, PointerValue, Value, MetadataValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IntValue {
    int_value: Value,
}

impl IntValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        IntValue {
            int_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.int_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.int_value.set_name(name);
    }

    pub fn get_type(&self) -> IntType {
        IntType::new(self.int_value.get_type())
    }

    pub fn is_null(&self) -> bool {
        self.int_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.int_value.is_undef()
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.int_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.int_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.int_value.as_instruction()
    }

    pub fn const_not(&self) -> Self {
        let value = unsafe {
            LLVMConstNot(self.as_value_ref())
        };

        IntValue::new(value)
    }

    // REVIEW: What happens when not using a const value? This and other fns
    pub fn const_neg(&self) -> Self {
        let value = unsafe {
            LLVMConstNeg(self.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nsw_neg(&self) -> Self {
        let value = unsafe {
            LLVMConstNSWNeg(self.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nuw_neg(&self) -> Self {
        let value = unsafe {
            LLVMConstNUWNeg(self.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_add(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstAdd(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nsw_add(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstNSWAdd(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nuw_add(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstNUWAdd(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_sub(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstSub(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nsw_sub(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstNSWSub(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nuw_sub(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstNUWSub(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_mul(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstMul(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nsw_mul(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstNSWMul(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nuw_mul(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstNUWMul(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_unsigned_div(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstUDiv(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_signed_div(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstSDiv(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_exact_signed_div(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstExactSDiv(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    #[llvm_versions(4.0 => latest)]
    pub fn const_exact_unsigned_div(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstExactUDiv(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_unsigned_remainder(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstURem(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_signed_remainder(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstSRem(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_and(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstAnd(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_or(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstOr(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_xor(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstXor(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    // TODO: Could infer is_signed from type (one day)?
    pub fn const_cast(&self, int_type: IntType, is_signed: bool) -> Self {
        let value = unsafe {
            LLVMConstIntCast(self.as_value_ref(), int_type.as_type_ref(), is_signed as i32)
        };

        IntValue::new(value)
    }

    // TODO: Give shift methods more descriptive names
    pub fn const_shl(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstShl(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_rshr(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstLShr(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_ashr(&self, rhs: IntValue) -> Self {
        let value = unsafe {
            LLVMConstAShr(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    // SubType: const_to_float impl only for unsigned types
    pub fn const_unsigned_to_float(&self, float_type: FloatType) -> FloatValue {
        let value = unsafe {
            LLVMConstUIToFP(self.as_value_ref(), float_type.as_type_ref())
        };

        FloatValue::new(value)
    }

    // SubType: const_to_float impl only for signed types
    pub fn const_signed_to_float(&self, float_type: FloatType) -> FloatValue {
        let value = unsafe {
            LLVMConstSIToFP(self.as_value_ref(), float_type.as_type_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_to_pointer(&self, ptr_type: PointerType) -> PointerValue {
        let value = unsafe {
            LLVMConstIntToPtr(self.as_value_ref(), ptr_type.as_type_ref())
        };

        PointerValue::new(value)
    }

    pub fn const_truncate(&self, int_type: IntType) -> IntValue {
        let value = unsafe {
            LLVMConstTrunc(self.as_value_ref(), int_type.as_type_ref())
        };

        IntValue::new(value)
    }

    // TODO: More descriptive name
    pub fn const_s_extend(&self, int_type: IntType) -> IntValue {
        let value = unsafe {
            LLVMConstSExt(self.as_value_ref(), int_type.as_type_ref())
        };

        IntValue::new(value)
    }

    // TODO: More descriptive name
    pub fn const_z_ext(&self, int_type: IntType) -> IntValue {
        let value = unsafe {
            LLVMConstZExt(self.as_value_ref(), int_type.as_type_ref())
        };

        IntValue::new(value)
    }

    pub fn const_truncate_or_bit_cast(&self, int_type: IntType) -> IntValue {
        let value = unsafe {
            LLVMConstTruncOrBitCast(self.as_value_ref(), int_type.as_type_ref())
        };

        IntValue::new(value)
    }

    // TODO: More descriptive name
    pub fn const_s_extend_or_bit_cast(&self, int_type: IntType) -> IntValue {
        let value = unsafe {
            LLVMConstSExtOrBitCast(self.as_value_ref(), int_type.as_type_ref())
        };

        IntValue::new(value)
    }

    // TODO: More descriptive name
    pub fn const_z_ext_or_bit_cast(&self, int_type: IntType) -> IntValue {
        let value = unsafe {
            LLVMConstZExtOrBitCast(self.as_value_ref(), int_type.as_type_ref())
        };

        IntValue::new(value)
    }

    pub fn const_bit_cast(&self, int_type: IntType) -> IntValue {
        let value = unsafe {
            LLVMConstBitCast(self.as_value_ref(), int_type.as_type_ref())
        };

        IntValue::new(value)
    }

    pub fn has_metadata(&self) -> bool {
        self.int_value.has_metadata()
    }

    pub fn get_metadata(&self, kind_id: u32) -> Option<MetadataValue> {
        self.int_value.get_metadata(kind_id)
    }

    pub fn set_metadata(&self, metadata: MetadataValue, kind_id: u32) {
        self.int_value.set_metadata(metadata, kind_id)
    }

    // SubType: rhs same as lhs; return IntValue<bool>
    pub fn const_int_compare(&self, op: IntPredicate, rhs: IntValue) -> IntValue {
        let value = unsafe {
            LLVMConstICmp(op.as_llvm_enum(), self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    // SubTypes: self can only be IntValue<bool>
    pub fn const_select<BV: BasicValue>(&self, then: BV, else_: BV) -> BasicValueEnum {
        let value = unsafe {
            LLVMConstSelect(self.as_value_ref(), then.as_value_ref(), else_.as_value_ref())
        };

        BasicValueEnum::new(value)
    }

    /// Determines whether or not an `IntValue` is a constant.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i64_type = context.i64_type();
    /// let i64_val = i64_type.const_int(12, false);
    ///
    /// assert!(i64_val.is_const());
    /// ```
    pub fn is_const(&self) -> bool {
        self.int_value.is_const()
    }

    /// Obtains a constant `IntValue`'s zero extended value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_all_ones = i8_type.const_all_ones();
    ///
    /// assert_eq!(i8_all_ones.get_zero_extended_constant(), Some(255));
    /// ```
    pub fn get_zero_extended_constant(&self) -> Option<u64> {
        // Garbage values are produced on non constant values
        if !self.is_const() {
            return None;
        }

        unsafe {
            Some(LLVMConstIntGetZExtValue(self.as_value_ref()))
        }
    }

    /// Obtains a constant `IntValue`'s sign extended value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_all_ones = i8_type.const_all_ones();
    ///
    /// assert_eq!(i8_all_ones.get_sign_extended_constant(), Some(-1));
    /// ```
    pub fn get_sign_extended_constant(&self) -> Option<i64> {
        // Garbage values are produced on non constant values
        if !self.is_const() {
            return None;
        }

        unsafe {
            Some(LLVMConstIntGetSExtValue(self.as_value_ref()))
        }
    }

    pub fn replace_all_uses_with(&self, other: IntValue) {
        self.int_value.replace_all_uses_with(other.as_value_ref())
    }
}

impl AsValueRef for IntValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.int_value.value
    }
}
