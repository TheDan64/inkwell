use llvm_sys::core::{LLVMConstFNeg, LLVMConstFAdd, LLVMConstFSub, LLVMConstFMul, LLVMConstFDiv, LLVMConstFRem, LLVMConstFPCast, LLVMConstFPToUI, LLVMConstFPToSI, LLVMConstFPTrunc, LLVMConstFPExt, LLVMConstFCmp, LLVMConstRealGetDouble};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use FloatPredicate;
use support::LLVMString;
use types::{AsTypeRef, FloatType, IntType};
use values::traits::AsValueRef;
use values::{InstructionValue, IntValue, Value, MetadataValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FloatValue {
    float_value: Value
}

impl FloatValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        FloatValue {
            float_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.float_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.float_value.set_name(name);
    }

    pub fn get_type(&self) -> FloatType {
        FloatType::new(self.float_value.get_type())
    }

    pub fn is_null(&self) -> bool {
        self.float_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.float_value.is_undef()
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.float_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.float_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.float_value.as_instruction()
    }

    pub fn const_neg(&self) -> Self {
        let value = unsafe {
            LLVMConstFNeg(self.as_value_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_add(&self, rhs: FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFAdd(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_sub(&self, rhs: FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFSub(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_mul(&self, rhs: FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFMul(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_div(&self, rhs: FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFDiv(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_remainder(&self, rhs: FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFRem(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_cast(&self, float_type: FloatType) -> Self {
        let value = unsafe {
            LLVMConstFPCast(self.as_value_ref(), float_type.as_type_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_to_unsigned_int(&self, int_type: IntType) -> IntValue {
        let value = unsafe {
            LLVMConstFPToUI(self.as_value_ref(), int_type.as_type_ref())
        };

        IntValue::new(value)
    }

    pub fn const_to_signed_int(&self, int_type: IntType) -> IntValue {
        let value = unsafe {
            LLVMConstFPToSI(self.as_value_ref(), int_type.as_type_ref())
        };

        IntValue::new(value)
    }

    pub fn const_truncate(&self, float_type: FloatType) -> FloatValue {
        let value = unsafe {
            LLVMConstFPTrunc(self.as_value_ref(), float_type.as_type_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_extend(&self, float_type: FloatType) -> FloatValue {
        let value = unsafe {
            LLVMConstFPExt(self.as_value_ref(), float_type.as_type_ref())
        };

        FloatValue::new(value)
    }

    pub fn has_metadata(&self) -> bool {
        self.float_value.has_metadata()
    }

    pub fn get_metadata(&self, kind_id: u32) -> Option<MetadataValue> {
        self.float_value.get_metadata(kind_id)
    }

    pub fn set_metadata(&self, metadata: MetadataValue, kind_id: u32) {
        self.float_value.set_metadata(metadata, kind_id)
    }

    // SubType: rhs same as lhs; return IntValue<bool>
    pub fn const_compare(&self, op: FloatPredicate, rhs: FloatValue) -> IntValue {
        let value = unsafe {
            LLVMConstFCmp(op.as_llvm_enum(), self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
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
    pub fn is_const(&self) -> bool {
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
    pub fn get_constant(&self) -> Option<(f64, bool)> {
        // Nothing bad happens as far as I can tell if we don't check if const
        // unlike the int versions, but just doing this just in case and for consistency
        if !self.is_const() {
            return None;
        }

        let mut lossy = 0;
        let constant = unsafe {
            LLVMConstRealGetDouble(self.as_value_ref(), &mut lossy)
        };

        Some((constant, lossy == 1))
    }

    pub fn replace_all_uses_with(&self, other: FloatValue) {
        self.float_value.replace_all_uses_with(other.as_value_ref())
    }
}

impl AsValueRef for FloatValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.float_value.value
    }
}
