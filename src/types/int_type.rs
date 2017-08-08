use llvm_sys::core::{LLVMInt1Type, LLVMInt8Type, LLVMInt16Type, LLVMInt32Type, LLVMInt64Type, LLVMConstInt, LLVMConstNull, LLVMConstAllOnes, LLVMIntType, LLVMGetIntTypeWidth};
use llvm_sys::prelude::LLVMTypeRef;

use std::ffi::CStr;

use context::ContextRef;
use types::traits::AsTypeRef;
use types::{Type, ArrayType, BasicType, VectorType, PointerType, FunctionType};
use values::{IntValue, PointerValue};

#[derive(Debug, PartialEq, Eq)]
pub struct IntType {
    int_type: Type,
}

impl IntType {
    pub(crate) fn new(int_type: LLVMTypeRef) -> Self {
        assert!(!int_type.is_null());

        IntType {
            int_type: Type::new(int_type),
        }
    }

    pub fn bool_type() -> Self {
        let type_ = unsafe {
            LLVMInt1Type()
        };

        IntType::new(type_)
    }

    pub fn i8_type() -> Self {
        let type_ = unsafe {
            LLVMInt8Type()
        };

        IntType::new(type_)
    }

    pub fn i16_type() -> Self {
        let type_ = unsafe {
            LLVMInt16Type()
        };

        IntType::new(type_)
    }

    pub fn i32_type() -> Self {
        let type_ = unsafe {
            LLVMInt32Type()
        };

        IntType::new(type_)
    }

    pub fn i64_type() -> Self {
        let type_ = unsafe {
            LLVMInt64Type()
        };

        IntType::new(type_)
    }

    pub fn i128_type() -> Self {
        // REVIEW: The docs says there's a LLVMInt128Type, but
        // it might only be in a newer version

        Self::custom_width_int_type(128)
    }

    pub fn custom_width_int_type(bits: u32) -> Self {
        let type_ = unsafe {
            LLVMIntType(bits)
        };

        IntType::new(type_)
    }

    pub fn const_int(&self, value: u64, sign_extend: bool) -> IntValue {
        let value = unsafe {
            LLVMConstInt(self.as_type_ref(), value, sign_extend as i32)
        };

        IntValue::new(value)
    }

    pub fn const_all_ones(&self) -> IntValue {
        let value = unsafe {
            LLVMConstAllOnes(self.as_type_ref())
        };

        IntValue::new(value)
    }

    pub fn const_null_ptr(&self) -> PointerValue {
        self.int_type.const_null_ptr()
    }

    pub fn const_null(&self) -> IntValue {
        let null = unsafe {
            LLVMConstNull(self.as_type_ref())
        };

        IntValue::new(null)
    }

    pub fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
        self.int_type.fn_type(param_types, is_var_args)
    }

    pub fn array_type(&self, size: u32) -> ArrayType {
        self.int_type.array_type(size)
    }

    pub fn vec_type(&self, size: u32) -> VectorType {
        self.int_type.vec_type(size)
    }

    pub fn get_context(&self) -> ContextRef {
        self.int_type.get_context()
    }

    pub fn is_sized(&self) -> bool {
        self.int_type.is_sized()
    }

    pub fn ptr_type(&self, address_space: u32) -> PointerType {
        self.int_type.ptr_type(address_space)
    }

    pub fn get_bit_width(&self) -> u32 {
        unsafe {
            LLVMGetIntTypeWidth(self.as_type_ref())
        }
    }

    pub fn print_to_string(&self) -> &CStr {
        self.int_type.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.int_type.print_to_stderr()
    }

    pub fn get_undef(&self) -> IntValue {
        IntValue::new(self.int_type.get_undef())
    }
}

impl AsTypeRef for IntType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.int_type.type_
    }
}
