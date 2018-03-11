use libc::c_void;
use llvm_sys::execution_engine::{LLVMCreateGenericValueOfPointer, LLVMDisposeGenericValue, LLVMGenericValueIntWidth, LLVMGenericValueRef, LLVMGenericValueToInt, LLVMGenericValueToFloat, LLVMGenericValueToPointer};

use types::{AsTypeRef, FloatType};

// SubTypes: GenericValue<IntValue, FloatValue, or PointerValue>
#[derive(Debug)]
pub struct GenericValue {
    pub(crate) generic_value: LLVMGenericValueRef,
}

impl GenericValue {
    pub(crate) fn new(generic_value: LLVMGenericValueRef) -> Self {
        assert!(!generic_value.is_null());

        GenericValue {
            generic_value,
        }
    }

    // SubType: GenericValue<IntValue> only
    pub fn int_width(&self) -> u32 {
        unsafe {
            LLVMGenericValueIntWidth(self.generic_value)
        }
    }

    // SubType: create_generic_value() -> GenericValue<PointerValue, T>
    // REVIEW: How safe is this really?
    pub unsafe fn create_generic_value_of_pointer<T>(value: &mut T) -> Self {
        let value = LLVMCreateGenericValueOfPointer(value as *mut _ as *mut c_void);

        GenericValue::new(value)
    }

    // SubType: impl only for GenericValue<IntValue>
    pub fn as_int(&self, is_signed: bool) -> u64 {
        unsafe {
            LLVMGenericValueToInt(self.generic_value, is_signed as i32)
        }
    }

    // SubType: impl only for GenericValue<FloatValue>
    pub fn as_float(&self, float_type: &FloatType) -> f64 {
        unsafe {
            LLVMGenericValueToFloat(float_type.as_type_ref(), self.generic_value)
        }
    }

    // SubType: impl only for GenericValue<PointerValue, T>
    // REVIEW: How safe is this really?
    pub unsafe fn into_pointer<T>(self) -> *mut T {
        LLVMGenericValueToPointer(self.generic_value) as *mut T
    }
}

impl Drop for GenericValue {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeGenericValue(self.generic_value)
        }
    }
}
