use llvm_sys::core::LLVMVoidType;
use llvm_sys::prelude::LLVMTypeRef;

use context::ContextRef;
use types::traits::AsTypeRef;
use types::{Type, BasicType, FunctionType, PointerType};
use values::PointerValue;

use std::ffi::CStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VoidType {
    void_type: Type,
}

impl VoidType {
    pub(crate) fn new(void_type: LLVMTypeRef) -> Self {
        assert!(!void_type.is_null());

        VoidType {
            void_type: Type::new(void_type),
        }
    }

    // REVIEW: Always false -> const fn?
    pub fn is_sized(&self) -> bool {
        self.void_type.is_sized()
    }

    pub fn get_context(&self) -> ContextRef {
        self.void_type.get_context()
    }

    pub fn ptr_type(&self, address_space: u32) -> PointerType {
        self.void_type.ptr_type(address_space)
    }

    pub fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
        self.void_type.fn_type(param_types, is_var_args)
    }

    pub fn void_type() -> Self {
        let void_type = unsafe {
            LLVMVoidType()
        };

        VoidType::new(void_type)
    }

    pub fn print_to_string(&self) -> &CStr {
        self.void_type.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.void_type.print_to_stderr()
    }

    pub fn const_null_ptr(&self) -> PointerValue {
        self.void_type.const_null_ptr()
    }
}

impl AsTypeRef for VoidType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.void_type.type_
    }
}
