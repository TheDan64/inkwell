use llvm_sys::LLVMThreadLocalMode;
use llvm_sys::core::{LLVMGetVisibility, LLVMSetVisibility, LLVMGetSection, LLVMSetSection, LLVMIsExternallyInitialized, LLVMSetExternallyInitialized, LLVMDeleteGlobal, LLVMIsGlobalConstant, LLVMSetGlobalConstant, LLVMGetPreviousGlobal, LLVMGetNextGlobal, LLVMHasUnnamedAddr, LLVMSetUnnamedAddr, LLVMIsThreadLocal, LLVMSetThreadLocal, LLVMGetThreadLocalMode, LLVMSetThreadLocalMode, LLVMGetInitializer, LLVMSetInitializer, LLVMIsDeclaration, LLVMGetDLLStorageClass, LLVMSetDLLStorageClass, LLVMGetAlignment, LLVMSetAlignment};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::{CString, CStr};

use {GlobalVisibility, ThreadLocalMode, DLLStorageClass};
use values::traits::AsValueRef;
use values::{BasicValueEnum, BasicValue, PointerValue, Value};

// REVIEW: GlobalValues are always PointerValues. With SubTypes, we should
// compress this into a PointerValue<Global> type
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

    pub fn get_dll_storage_class(&self) -> DLLStorageClass {
        let dll_storage_class = unsafe {
            LLVMGetDLLStorageClass(self.as_value_ref())
        };

        DLLStorageClass::new(dll_storage_class)
    }

    pub fn set_dll_storage_class(&self, dll_storage_class: DLLStorageClass) {
        unsafe {
            LLVMSetDLLStorageClass(self.as_value_ref(), dll_storage_class.as_llvm_class())
        }
    }

    pub fn get_initializer(&self) -> Option<BasicValueEnum> {
        let value = unsafe {
            LLVMGetInitializer(self.as_value_ref())
        };

        if value.is_null() {
            return None;
        }

        Some(BasicValueEnum::new(value))
    }

    // SubType: This input type should be tied to the BasicType
    pub fn set_initializer(&self, value: &BasicValue) {
        unsafe {
            LLVMSetInitializer(self.as_value_ref(), value.as_value_ref())
        }
    }

    pub fn is_thread_local(&self) -> bool {
        unsafe {
            LLVMIsThreadLocal(self.as_value_ref()) == 1
        }
    }

    // TODOC: Setting this to true is the same as setting GeneralDynamicTLSModel
    pub fn set_thread_local(&self, is_thread_local: bool) {
        unsafe {
            LLVMSetThreadLocal(self.as_value_ref(), is_thread_local as i32)
        }
    }

    pub fn get_thread_local_mode(&self) -> Option<ThreadLocalMode> {
        let thread_local_mode = unsafe {
            LLVMGetThreadLocalMode(self.as_value_ref())
        };

        ThreadLocalMode::new(thread_local_mode)
    }

    // REVIEW: Does this have any bad behavior if it isn't thread local or just a noop?
    // or should it call self.set_thread_local(true)?
    pub fn set_thread_local_mode(&self, thread_local_mode: Option<ThreadLocalMode>) {
        let thread_local_mode = match thread_local_mode {
            Some(mode) => mode.as_llvm_mode(),
            None => LLVMThreadLocalMode::LLVMNotThreadLocal,
        };

        unsafe {
            LLVMSetThreadLocalMode(self.as_value_ref(), thread_local_mode)
        }
    }

    // SubType: This should be moved into the type. GlobalValue<Initialized/Uninitialized>
    // TODO: FunctionValues as well?
    pub fn is_declaration(&self) -> bool {
        unsafe {
            LLVMIsDeclaration(self.as_value_ref()) == 1
        }
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

    pub fn set_visibility(&self, visibility: GlobalVisibility) {
        unsafe {
            LLVMSetVisibility(self.as_value_ref(), visibility.as_llvm_visibility())
        }
    }

    pub fn get_visibility(&self) -> GlobalVisibility {
        let visibility = unsafe {
            LLVMGetVisibility(self.as_value_ref())
        };

        GlobalVisibility::new(visibility)
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

    pub unsafe fn delete(self) {
        LLVMDeleteGlobal(self.as_value_ref())
    }

    pub fn as_pointer_value(&self) -> PointerValue {
        PointerValue::new(self.as_value_ref())
    }

    pub fn get_alignment(&self) -> u32 {
        unsafe {
            LLVMGetAlignment(self.as_value_ref())
        }
    }

    pub fn set_alignment(&self, alignment: u32) {
        unsafe {
            LLVMSetAlignment(self.as_value_ref(), alignment)
        }
    }
}

impl AsValueRef for GlobalValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.global_value.value
    }
}
