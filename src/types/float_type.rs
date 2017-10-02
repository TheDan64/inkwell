use llvm_sys::core::{LLVMConstReal, LLVMConstNull, LLVMHalfType, LLVMFloatType, LLVMDoubleType, LLVMFP128Type, LLVMPPCFP128Type, LLVMConstRealOfStringAndSize};
use llvm_sys::execution_engine::LLVMCreateGenericValueOfFloat;
use llvm_sys::prelude::LLVMTypeRef;

use std::ffi::CStr;

use context::ContextRef;
use types::traits::AsTypeRef;
use types::{Type, PointerType, FunctionType, BasicType, ArrayType, VectorType};
use values::{FloatValue, GenericValue, PointerValue, IntValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FloatType {
    float_type: Type,
}

impl FloatType {
    pub(crate) fn new(float_type: LLVMTypeRef) -> Self {
        assert!(!float_type.is_null());

        FloatType {
            float_type: Type::new(float_type),
        }
    }

    pub fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
        self.float_type.fn_type(param_types, is_var_args)
    }

    pub fn array_type(&self, size: u32) -> ArrayType {
        self.float_type.array_type(size)
    }

    pub fn vec_type(&self, size: u32) -> VectorType {
        self.float_type.vec_type(size)
    }

    pub fn const_float(&self, value: f64) -> FloatValue {
        let value = unsafe {
            LLVMConstReal(self.float_type.type_, value)
        };

        FloatValue::new(value)
    }

    // REVIEW: What happens when string is invalid? Nullptr?
    pub fn const_float_from_string(&self, slice: &str) -> FloatValue {
        let value = unsafe {
            LLVMConstRealOfStringAndSize(self.as_type_ref(), slice.as_ptr() as *const i8, slice.len() as u32)
        };

        FloatValue::new(value)
    }

    pub fn const_null_ptr(&self) -> PointerValue {
        self.float_type.const_null_ptr()
    }

    pub fn const_null(&self) -> FloatValue {
        let null = unsafe {
            LLVMConstNull(self.as_type_ref())
        };

        FloatValue::new(null)
    }

    // REVIEW: Always true -> const fn?
    pub fn is_sized(&self) -> bool {
        self.float_type.is_sized()
    }

    pub fn size_of(&self) -> IntValue {
        self.float_type.size_of()
    }

    pub fn get_context(&self) -> ContextRef {
        self.float_type.get_context()
    }

    pub fn ptr_type(&self, address_space: u32) -> PointerType {
        self.float_type.ptr_type(address_space)
    }

    pub fn f16_type() -> Self {
        let float_type = unsafe {
            LLVMHalfType()
        };

        FloatType::new(float_type)
    }

    pub fn f32_type() -> Self {
        let float_type = unsafe {
            LLVMFloatType()
        };

        FloatType::new(float_type)
    }

    pub fn f64_type() -> Self {
        let float_type = unsafe {
            LLVMDoubleType()
        };

        FloatType::new(float_type)
    }

    pub fn f128_type() -> Self {
        let float_type = unsafe {
            LLVMFP128Type()
        };

        FloatType::new(float_type)
    }

    pub fn ppc_f128_type() -> Self {
        let float_type = unsafe {
            LLVMPPCFP128Type()
        };

        FloatType::new(float_type)
    }

    pub fn print_to_string(&self) -> &CStr {
        self.float_type.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.float_type.print_to_stderr()
    }

    pub fn get_undef(&self) -> FloatValue {
        FloatValue::new(self.float_type.get_undef())
    }

    pub fn create_generic_value(&self, value: f64) -> GenericValue {
        let value = unsafe {
            LLVMCreateGenericValueOfFloat(self.as_type_ref(), value)
        };

        GenericValue::new(value)
    }
}

impl AsTypeRef for FloatType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.float_type.type_
    }
}
