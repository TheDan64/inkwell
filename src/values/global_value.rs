use llvm_sys::core::{LLVMGetVisibility, LLVMSetVisibility, LLVMGetSection, LLVMSetSection, LLVMIsExternallyInitialized, LLVMSetExternallyInitialized, LLVMDeleteGlobal, LLVMIsGlobalConstant, LLVMSetGlobalConstant, LLVMGetPreviousGlobal, LLVMGetNextGlobal, LLVMHasUnnamedAddr, LLVMSetUnnamedAddr};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::{CString, CStr};

use GlobalVisibility;
use values::traits::AsValueRef;
use values::{BasicValueEnum, Value};

// REVIEW: GlobalValues may always be PointerValues, in which case we can
// simplify from BasicValueEnum down to PointerValue
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GlobalValue {
    global_value: Value,
}

impl GlobalValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        GlobalValue {
            global_value: Value::new(value),
        }
    }

    pub fn get_previous_global(&self) -> Option<GlobalValue> {
        let value = unsafe {
            LLVMGetPreviousGlobal(self.as_value_ref())
        };

        if value.is_null() {
            return None;
        }

        Some(GlobalValue::new(value))
    }

    pub fn get_next_global(&self) -> Option<GlobalValue> {
        let value = unsafe {
            LLVMGetNextGlobal(self.as_value_ref())
        };

        if value.is_null() {
            return None;
        }

        Some(GlobalValue::new(value))
    }

    pub fn has_unnamed_addr(&self) -> bool {
        unsafe {
            LLVMHasUnnamedAddr(self.as_value_ref()) == 1
        }
    }

    pub fn set_unnamed_addr(&self, has_unnamed_addr: bool) {
        unsafe {
            LLVMSetUnnamedAddr(self.as_value_ref(), has_unnamed_addr as i32)
        }
    }

    pub fn is_constant(&self) -> bool {
        unsafe {
            LLVMIsGlobalConstant(self.as_value_ref()) == 1
        }
    }

    pub fn set_constant(&self, is_constant: bool) {
        unsafe {
            LLVMSetGlobalConstant(self.as_value_ref(), is_constant as i32)
        }
    }

    pub fn is_externally_initialized(&self) -> bool {
        unsafe {
            LLVMIsExternallyInitialized(self.as_value_ref()) == 1
        }
    }

    pub fn set_externally_initialized(&self, externally_initialized: bool) {
        unsafe {
            LLVMSetExternallyInitialized(self.as_value_ref(), externally_initialized as i32)
        }
    }

    pub fn set_visibility(&self, visibility: &GlobalVisibility) {
        unsafe {
            LLVMSetVisibility(self.as_value_ref(), visibility.as_llvm_visibility())
        }
    }

    pub fn get_visibility(&self) -> GlobalVisibility {
        let visibility = unsafe {
            LLVMGetVisibility(self.as_value_ref())
        };

        GlobalVisibility::from_llvm_visibility(visibility)
    }

    pub fn get_section(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetSection(self.as_value_ref()))
        }
    }

    pub fn set_section(&self, section: &str) {
        let c_string = CString::new(section).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetSection(self.as_value_ref(), c_string.as_ptr())
        }
    }

    pub fn as_basic_value(&self) -> BasicValueEnum {
        BasicValueEnum::new(self.as_value_ref())
    }

    pub unsafe fn delete(self) {
        LLVMDeleteGlobal(self.as_value_ref())
    }
}

impl AsValueRef for GlobalValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.global_value.value
    }
}
