use llvm_sys::core::{LLVMGetPointerAddressSpace, LLVMConstNull};
use llvm_sys::prelude::LLVMTypeRef;

use std::ffi::CStr;

use AddressSpace;
use context::ContextRef;
use types::traits::AsTypeRef;
use types::{Type, BasicType, ArrayType, FunctionType, VectorType};
use values::{PointerValue, IntValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PointerType {
    ptr_type: Type,
}

impl PointerType {
    pub(crate) fn new(ptr_type: LLVMTypeRef) -> Self {
        assert!(!ptr_type.is_null());

        PointerType {
            ptr_type: Type::new(ptr_type),
        }
    }

    pub fn is_sized(&self) -> bool {
        self.ptr_type.is_sized()
    }

    pub fn size_of(&self) -> IntValue {
        self.ptr_type.size_of()
    }

    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.ptr_type.ptr_type(address_space)
    }

    pub fn get_context(&self) -> ContextRef {
        self.ptr_type.get_context()
    }

    pub fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
        self.ptr_type.fn_type(param_types, is_var_args)
    }

    pub fn array_type(&self, size: u32) -> ArrayType {
        self.ptr_type.array_type(size)
    }

    pub fn get_address_space(&self) -> AddressSpace {
        unsafe {
            LLVMGetPointerAddressSpace(self.as_type_ref()).into()
        }
    }

    pub fn print_to_string(&self) -> &CStr {
        self.ptr_type.print_to_string()
    }

    #[cfg(not(feature = "llvm3-6"))]
    pub fn print_to_stderr(&self) {
        self.ptr_type.print_to_stderr()
    }

    pub fn const_null_ptr(&self) -> PointerValue {
        self.ptr_type.const_null_ptr()
    }

    pub fn const_null(&self) -> PointerValue {
        let null = unsafe {
            LLVMConstNull(self.as_type_ref())
        };

        PointerValue::new(null)
    }

    pub fn get_undef(&self) -> PointerValue {
        PointerValue::new(self.ptr_type.get_undef())
    }

    pub fn vec_type(&self, size: u32) -> VectorType {
        self.ptr_type.vec_type(size)
    }
}

impl AsTypeRef for PointerType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.ptr_type.type_
    }
}
