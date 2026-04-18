use libc::c_void;
use llvm_sys::execution_engine::{
    LLVMCreateGenericValueOfPointer, LLVMDisposeGenericValue, LLVMGenericValueIntWidth, LLVMGenericValueRef,
    LLVMGenericValueToFloat, LLVMGenericValueToInt, LLVMGenericValueToPointer, LLVMOpaqueGenericValue,
};

use crate::{
    support::assert_niche,
    types::{AsTypeRef, FloatType},
};

use std::{marker::PhantomData, ptr::NonNull};

// SubTypes: GenericValue<IntValue, FloatValue, or PointerValue>
#[repr(transparent)]
#[derive(Debug)]
pub struct GenericValue<'ctx> {
    pub(crate) generic_value: NonNull<LLVMOpaqueGenericValue>,
    _phantom: PhantomData<&'ctx ()>,
}
const _: () = assert_niche::<GenericValue>();

impl<'ctx> GenericValue<'ctx> {
    pub(crate) unsafe fn new(generic_value: LLVMGenericValueRef) -> Self {
        assert!(!generic_value.is_null());

        GenericValue {
            generic_value: unsafe { NonNull::new_unchecked(generic_value) },
            _phantom: PhantomData,
        }
    }

    // SubType: GenericValue<IntValue> only
    pub fn int_width(self) -> u32 {
        unsafe { LLVMGenericValueIntWidth(self.generic_value.as_ptr()) }
    }

    // SubType: create_generic_value() -> GenericValue<PointerValue, T>
    // REVIEW: How safe is this really?
    pub unsafe fn create_generic_value_of_pointer<T>(value: &mut T) -> Self {
        unsafe {
            let value = LLVMCreateGenericValueOfPointer(value as *mut _ as *mut c_void);

            GenericValue::new(value)
        }
    }

    // SubType: impl only for GenericValue<IntValue>
    pub fn as_int(self, is_signed: bool) -> u64 {
        unsafe { LLVMGenericValueToInt(self.generic_value.as_ptr(), is_signed as i32) }
    }

    // SubType: impl only for GenericValue<FloatValue>
    pub fn as_float(self, float_type: &FloatType<'ctx>) -> f64 {
        unsafe { LLVMGenericValueToFloat(float_type.as_type_ref(), self.generic_value.as_ptr()) }
    }

    // SubType: impl only for GenericValue<PointerValue, T>
    // REVIEW: How safe is this really?
    pub unsafe fn into_pointer<T>(self) -> *mut T {
        unsafe { LLVMGenericValueToPointer(self.generic_value.as_ptr()) as *mut T }
    }
}

impl Drop for GenericValue<'_> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeGenericValue(self.generic_value.as_ptr()) }
    }
}
