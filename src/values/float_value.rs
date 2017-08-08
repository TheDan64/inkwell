use llvm_sys::core::{LLVMConstFNeg, LLVMConstFAdd, LLVMConstFSub, LLVMConstFMul, LLVMConstFDiv, LLVMConstFRem, LLVMConstFPCast};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use types::FloatType;
use types::AsTypeRef;
use values::traits::AsValueRef;
use values::{InstructionValue, Value};

#[derive(Debug, PartialEq, Eq)]
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

    pub fn print_to_string(&self) -> &CStr {
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

    // TODO: operator overloading to call this
    pub fn const_add(&self, rhs: &FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFAdd(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    // TODO: operator overloading to call this
    pub fn const_sub(&self, rhs: &FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFSub(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    // TODO: operator overloading to call this
    pub fn const_mul(&self, rhs: &FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFMul(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    // TODO: operator overloading to call this
    pub fn const_div(&self, rhs: &FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFDiv(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_remainder(&self, rhs: &FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFRem(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_cast(&self, float_type: &FloatType) -> Self {
        let value = unsafe {
            LLVMConstFPCast(self.as_value_ref(), float_type.as_type_ref())
        };

        FloatValue::new(value)
    }
}

impl AsValueRef for FloatValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.float_value.value
    }
}
