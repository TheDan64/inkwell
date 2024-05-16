#[llvm_versions(..=17)]
use crate::types::IntType;
#[llvm_versions(..=15)]
use llvm_sys::core::LLVMConstFNeg;
use llvm_sys::core::{LLVMConstFCmp, LLVMConstRealGetDouble};
#[llvm_versions(..=17)]
use llvm_sys::core::{LLVMConstFPCast, LLVMConstFPExt, LLVMConstFPToSI, LLVMConstFPToUI, LLVMConstFPTrunc};
use llvm_sys::prelude::LLVMValueRef;

use std::convert::TryFrom;
use std::ffi::CStr;
use std::fmt::{self, Display};

#[llvm_versions(..=17)]
use crate::types::AsTypeRef;
use crate::types::FloatType;
use crate::values::traits::AsValueRef;
use crate::values::{InstructionValue, IntValue, Value};
use crate::FloatPredicate;

use super::AnyValue;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FloatValue<'ctx> {
    float_value: Value<'ctx>,
}

impl<'ctx> FloatValue<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of type float.
    pub unsafe fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        FloatValue {
            float_value: Value::new(value),
        }
    }

    /// Gets name of the `FloatValue`. If the value is a constant, this will
    /// return an empty string.
    pub fn get_name(&self) -> &CStr {
        self.float_value.get_name()
    }

    /// Set name of the `FloatValue`.
    pub fn set_name(&self, name: &str) {
        self.float_value.set_name(name)
    }

    pub fn get_type(self) -> FloatType<'ctx> {
        unsafe { FloatType::new(self.float_value.get_type()) }
    }

    pub fn is_null(self) -> bool {
        self.float_value.is_null()
    }

    pub fn is_undef(self) -> bool {
        self.float_value.is_undef()
    }

    pub fn print_to_stderr(self) {
        self.float_value.print_to_stderr()
    }

    pub fn as_instruction(self) -> Option<InstructionValue<'ctx>> {
        self.float_value.as_instruction()
    }

    #[llvm_versions(..=15)]
    pub fn const_neg(self) -> Self {
        unsafe { FloatValue::new(LLVMConstFNeg(self.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_add(self, rhs: FloatValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstFAdd;

        unsafe { FloatValue::new(LLVMConstFAdd(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_sub(self, rhs: FloatValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstFSub;

        unsafe { FloatValue::new(LLVMConstFSub(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_mul(self, rhs: FloatValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstFMul;

        unsafe { FloatValue::new(LLVMConstFMul(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_div(self, rhs: FloatValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstFDiv;

        unsafe { FloatValue::new(LLVMConstFDiv(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=14)]
    pub fn const_remainder(self, rhs: FloatValue<'ctx>) -> Self {
        use llvm_sys::core::LLVMConstFRem;

        unsafe { FloatValue::new(LLVMConstFRem(self.as_value_ref(), rhs.as_value_ref())) }
    }

    #[llvm_versions(..=17)]
    pub fn const_cast(self, float_type: FloatType<'ctx>) -> Self {
        unsafe { FloatValue::new(LLVMConstFPCast(self.as_value_ref(), float_type.as_type_ref())) }
    }

    #[llvm_versions(..=17)]
    pub fn const_to_unsigned_int(self, int_type: IntType<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstFPToUI(self.as_value_ref(), int_type.as_type_ref())) }
    }

    #[llvm_versions(..=17)]
    pub fn const_to_signed_int(self, int_type: IntType<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstFPToSI(self.as_value_ref(), int_type.as_type_ref())) }
    }

    #[llvm_versions(..=17)]
    pub fn const_truncate(self, float_type: FloatType<'ctx>) -> FloatValue<'ctx> {
        unsafe { FloatValue::new(LLVMConstFPTrunc(self.as_value_ref(), float_type.as_type_ref())) }
    }

    #[llvm_versions(..=17)]
    pub fn const_extend(self, float_type: FloatType<'ctx>) -> FloatValue<'ctx> {
        unsafe { FloatValue::new(LLVMConstFPExt(self.as_value_ref(), float_type.as_type_ref())) }
    }

    // SubType: rhs same as lhs; return IntValue<bool>
    pub fn const_compare(self, op: FloatPredicate, rhs: FloatValue<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstFCmp(op.into(), self.as_value_ref(), rhs.as_value_ref())) }
    }

    /// Determines whether or not a `FloatValue` is a constant.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f64_type = context.f64_type();
    /// let f64_val = f64_type.const_float(1.2);
    ///
    /// assert!(f64_val.is_const());
    /// ```
    pub fn is_const(self) -> bool {
        self.float_value.is_const()
    }

    /// Obtains a constant `FloatValue`'s value and whether or not it lost info.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f64_type = context.f64_type();
    /// let f64_1_2 = f64_type.const_float(1.2);
    ///
    /// assert_eq!(f64_1_2.get_constant(), Some((1.2, false)));
    /// ```
    pub fn get_constant(self) -> Option<(f64, bool)> {
        // Nothing bad happens as far as I can tell if we don't check if const
        // unlike the int versions, but just doing this just in case and for consistency
        if !self.is_const() {
            return None;
        }

        let mut lossy = 0;
        let constant = unsafe { LLVMConstRealGetDouble(self.as_value_ref(), &mut lossy) };

        Some((constant, lossy == 1))
    }

    pub fn replace_all_uses_with(self, other: FloatValue<'ctx>) {
        self.float_value.replace_all_uses_with(other.as_value_ref())
    }
}

unsafe impl AsValueRef for FloatValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.float_value.value
    }
}

impl Display for FloatValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl<'ctx> TryFrom<InstructionValue<'ctx>> for FloatValue<'ctx> {
    type Error = ();

    fn try_from(value: InstructionValue) -> Result<Self, Self::Error> {
        if value.get_type().is_float_type() {
            unsafe { Ok(FloatValue::new(value.as_value_ref())) }
        } else {
            Err(())
        }
    }
}
