#[llvm_versions(..=16)]
use llvm_sys::core::LLVMConstSelect;
#[llvm_versions(..=17)]
use llvm_sys::core::{
    LLVMConstAShr, LLVMConstAnd, LLVMConstIntCast, LLVMConstLShr, LLVMConstOr, LLVMConstSExt, LLVMConstSExtOrBitCast,
    LLVMConstSIToFP, LLVMConstUIToFP, LLVMConstZExt, LLVMConstZExtOrBitCast,
};
use llvm_sys::core::{
    LLVMConstAdd, LLVMConstBitCast, LLVMConstICmp, LLVMConstIntGetSExtValue, LLVMConstIntGetZExtValue,
    LLVMConstIntToPtr, LLVMConstMul, LLVMConstNSWAdd, LLVMConstNSWMul, LLVMConstNSWNeg, LLVMConstNSWSub,
    LLVMConstNUWAdd, LLVMConstNUWMul, LLVMConstNUWNeg, LLVMConstNUWSub, LLVMConstNeg, LLVMConstNot, LLVMConstShl,
    LLVMConstSub, LLVMConstTrunc, LLVMConstTruncOrBitCast, LLVMConstXor, LLVMIsAConstantInt,
};
use llvm_sys::prelude::LLVMValueRef;

use std::convert::TryFrom;
use std::ffi::CStr;
use std::fmt::{self, Display};

#[llvm_versions(..=17)]
use crate::types::FloatType;
use crate::types::{AsTypeRef, IntType, PointerType};
use crate::values::traits::AsValueRef;
#[llvm_versions(..=17)]
use crate::values::FloatValue;
#[llvm_versions(..=16)]
use crate::values::{BasicValue, BasicValueEnum};
use crate::values::{InstructionValue, PointerValue, Value};
use crate::IntPredicate;

use super::AnyValue;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct IntValue<'ctx> {
    int_value: Value<'ctx>,
}

impl<'ctx> IntValue<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of type int.
    pub unsafe fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        IntValue {
            int_value: Value::new(value),
        }
    }

    /// Gets the name of an `IntValue`. If the value is a constant, this will
    /// return an empty string.
    pub fn get_name(&self) -> &CStr {
        self.int_value.get_name()
    }

    /// Set name of the `IntValue`.
    pub fn set_name(&self, name: &str) {
        self.int_value.set_name(name)
    }

    pub fn get_type(self) -> IntType<'ctx> {
        unsafe { IntType::new(self.int_value.get_type()) }
    }

    pub fn is_null(self) -> bool {
        self.int_value.is_null()
    }

    pub fn is_undef(self) -> bool {
        self.int_value.is_undef()
    }

    pub fn print_to_stderr(self) {
        self.int_value.print_to_stderr()
    }

    pub fn as_instruction(self) -> Option<InstructionValue<'ctx>> {
        self.int_value.as_instruction()
    }

    pub fn const_not(self) -> Self {
        unsafe { IntValue::new(LLVMConstNot(self.as_value_ref())) }
    }

    // REVIEW: What happens when not using a const value? This and other fns
    pub fn const_neg(self) -> Self {
        unsafe { IntValue::new(LLVMConstNeg(self.as_value_ref())) }
    }

    pub fn const_nsw_neg(self) -> Self {
        unsafe { IntValue::new(LLVMConstNSWNeg(self.as_value_ref())) }
    }

    pub fn const_nuw_neg(self) -> Self {
        unsafe { IntValue::new(LLVMConstNUWNeg(self.as_value_ref())) }
    }

    pub fn const_add(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstAdd(self.as_value_ref(), rhs.as_value_ref())) }
    }

    pub fn const_nsw_add(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstNSWAdd(self.as_value_ref(), rhs.as_value_ref())) }
    }

    pub fn const_nuw_add(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstNUWAdd(self.as_value_ref(), rhs.as_value_ref())) }
    }

    pub fn const_sub(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstSub(self.as_value_ref(), rhs.as_value_ref())) }
    }

    pub fn const_nsw_sub(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstNSWSub(self.as_value_ref(), rhs.as_value_ref())) }
    }

    pub fn const_nuw_sub(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstNUWSub(self.as_value_ref(), rhs.as_value_ref())) }
    }

    pub fn const_mul(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstMul(self.as_value_ref(), rhs.as_value_ref())) }
    }

    pub fn const_nsw_mul(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstNSWMul(self.as_value_ref(), rhs.as_value_ref())) }
    }

    pub fn const_nuw_mul(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstNUWMul(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_unsigned_div(self, rhs: IntValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstUDiv;

        unsafe { IntValue::new(LLVMConstUDiv(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_signed_div(self, rhs: IntValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstSDiv;

        unsafe { IntValue::new(LLVMConstSDiv(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_exact_signed_div(self, rhs: IntValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstExactSDiv;

        unsafe { IntValue::new(LLVMConstExactSDiv(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_exact_unsigned_div(self, rhs: IntValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstExactUDiv;

        unsafe { IntValue::new(LLVMConstExactUDiv(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_unsigned_remainder(self, rhs: IntValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstURem;

        unsafe { IntValue::new(LLVMConstURem(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_signed_remainder(self, rhs: IntValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstSRem;

        unsafe { IntValue::new(LLVMConstSRem(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=17)]
    pub fn const_and(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstAnd(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=17)]
    pub fn const_or(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstOr(self.as_value_ref(), rhs.as_value_ref())) }
    }

    pub fn const_xor(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstXor(self.as_value_ref(), rhs.as_value_ref())) }
    }

    // TODO: Could infer is_signed from type (one day)?
    #[llvm_versions(..=17)]
    pub fn const_cast(self, int_type: IntType<'ctx>, is_signed: bool) -> Self {
        unsafe {
            IntValue::new(LLVMConstIntCast(
                self.as_value_ref(),
                int_type.as_type_ref(),
                is_signed as i32,
            ))
        }
    }

    // TODO: Give shift methods more descriptive names
    pub fn const_shl(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstShl(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=17)]
    pub fn const_rshr(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstLShr(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=17)]
    pub fn const_ashr(self, rhs: IntValue<'ctx>) -> Self {
        unsafe { IntValue::new(LLVMConstAShr(self.as_value_ref(), rhs.as_value_ref())) }
    }

    // SubType: const_to_float impl only for unsigned types
    #[llvm_versions(..=17)]
    pub fn const_unsigned_to_float(self, float_type: FloatType<'ctx>) -> FloatValue<'ctx> {
        unsafe { FloatValue::new(LLVMConstUIToFP(self.as_value_ref(), float_type.as_type_ref())) }
    }

    // SubType: const_to_float impl only for signed types
    #[llvm_versions(..=17)]
    pub fn const_signed_to_float(self, float_type: FloatType<'ctx>) -> FloatValue<'ctx> {
        unsafe { FloatValue::new(LLVMConstSIToFP(self.as_value_ref(), float_type.as_type_ref())) }
    }

    pub fn const_to_pointer(self, ptr_type: PointerType<'ctx>) -> PointerValue<'ctx> {
        unsafe { PointerValue::new(LLVMConstIntToPtr(self.as_value_ref(), ptr_type.as_type_ref())) }
    }

    pub fn const_truncate(self, int_type: IntType<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstTrunc(self.as_value_ref(), int_type.as_type_ref())) }
    }

    // TODO: More descriptive name
    #[llvm_versions(..=17)]
    pub fn const_s_extend(self, int_type: IntType<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstSExt(self.as_value_ref(), int_type.as_type_ref())) }
    }

    // TODO: More descriptive name
    #[llvm_versions(..=17)]
    pub fn const_z_ext(self, int_type: IntType<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstZExt(self.as_value_ref(), int_type.as_type_ref())) }
    }

    pub fn const_truncate_or_bit_cast(self, int_type: IntType<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstTruncOrBitCast(self.as_value_ref(), int_type.as_type_ref())) }
    }

    // TODO: More descriptive name
    #[llvm_versions(..=17)]
    pub fn const_s_extend_or_bit_cast(self, int_type: IntType<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstSExtOrBitCast(self.as_value_ref(), int_type.as_type_ref())) }
    }

    // TODO: More descriptive name
    #[llvm_versions(..=17)]
    pub fn const_z_ext_or_bit_cast(self, int_type: IntType<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstZExtOrBitCast(self.as_value_ref(), int_type.as_type_ref())) }
    }

    pub fn const_bit_cast(self, int_type: IntType) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstBitCast(self.as_value_ref(), int_type.as_type_ref())) }
    }

    // SubType: rhs same as lhs; return IntValue<bool>
    pub fn const_int_compare(self, op: IntPredicate, rhs: IntValue<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstICmp(op.into(), self.as_value_ref(), rhs.as_value_ref())) }
    }

    // SubTypes: self can only be IntValue<bool>
    #[llvm_versions(..=16)]
    pub fn const_select<BV: BasicValue<'ctx>>(self, then: BV, else_: BV) -> BasicValueEnum<'ctx> {
        unsafe {
            BasicValueEnum::new(LLVMConstSelect(
                self.as_value_ref(),
                then.as_value_ref(),
                else_.as_value_ref(),
            ))
        }
    }

    /// Determines whether or not an `IntValue` is an `llvm::Constant`.
    ///
    /// Constants includes values that are not known at compile time, for
    /// example the address of a function casted to an integer.
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
    pub fn is_const(self) -> bool {
        self.int_value.is_const()
    }

    /// Determines whether or not an `IntValue` is an `llvm::ConstantInt`.
    ///
    /// ConstantInt only includes values that are known at compile time.
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
    /// assert!(i64_val.is_constant_int());
    /// ```
    pub fn is_constant_int(self) -> bool {
        !unsafe { LLVMIsAConstantInt(self.as_value_ref()) }.is_null()
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
    pub fn get_zero_extended_constant(self) -> Option<u64> {
        // Garbage values are produced on non constant values
        if !self.is_constant_int() {
            return None;
        }
        if self.get_type().get_bit_width() > 64 {
            return None;
        }

        unsafe { Some(LLVMConstIntGetZExtValue(self.as_value_ref())) }
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
    pub fn get_sign_extended_constant(self) -> Option<i64> {
        // Garbage values are produced on non constant values
        if !self.is_constant_int() {
            return None;
        }
        if self.get_type().get_bit_width() > 64 {
            return None;
        }

        unsafe { Some(LLVMConstIntGetSExtValue(self.as_value_ref())) }
    }

    pub fn replace_all_uses_with(self, other: IntValue<'ctx>) {
        self.int_value.replace_all_uses_with(other.as_value_ref())
    }
}

unsafe impl AsValueRef for IntValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.int_value.value
    }
}

impl Display for IntValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl<'ctx> TryFrom<InstructionValue<'ctx>> for IntValue<'ctx> {
    type Error = ();

    fn try_from(value: InstructionValue) -> Result<Self, Self::Error> {
        if value.get_type().is_int_type() {
            unsafe { Ok(IntValue::new(value.as_value_ref())) }
        } else {
            Err(())
        }
    }
}
