use llvm_sys::core::{LLVMGetParamTypes, LLVMIsFunctionVarArg, LLVMCountParamTypes};
use llvm_sys::prelude::LLVMTypeRef;

use std::ffi::CStr;
use std::fmt;
use std::mem::forget;

use context::ContextRef;
use types::traits::AsTypeRef;
use types::{Type, BasicTypeEnum};
// use values::FunctionValue;

#[derive(PartialEq, Eq)]
pub struct FunctionType {
    fn_type: Type,
}

impl FunctionType {
    pub(crate) fn new(fn_type: LLVMTypeRef) -> FunctionType {
        assert!(!fn_type.is_null());

        FunctionType {
            fn_type: Type::new(fn_type)
        }
    }

    pub fn is_var_arg(&self) -> bool {
        unsafe {
            LLVMIsFunctionVarArg(self.as_type_ref()) != 0
        }
    }

    pub fn get_param_types(&self) -> Vec<BasicTypeEnum> {
        let count = self.count_param_types();
        let mut raw_vec: Vec<LLVMTypeRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetParamTypes(self.as_type_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| BasicTypeEnum::new(*val)).collect()
    }

    pub fn count_param_types(&self) -> u32 {
        unsafe {
            LLVMCountParamTypes(self.as_type_ref())
        }
    }

    // REVIEW: Always false -> const fn?
    pub fn is_sized(&self) -> bool {
        self.fn_type.is_sized()
    }

    pub fn get_context(&self) -> ContextRef {
        self.fn_type.get_context()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.fn_type.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.fn_type.print_to_stderr()
    }

    // REVIEW: Can you do undef for functions?
    // Seems to "work" - no UB or SF so far but fails
    // LLVMIsAFunction() check. Commenting out for further research
    // pub fn get_undef(&self) -> FunctionValue {
    //     FunctionValue::new(self.fn_type.get_undef()).expect("Should always get an undef value")
    // }
}

impl fmt::Debug for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = self.print_to_string();

        write!(f, "FunctionType {{\n    address: {:?}\n    llvm_type: {:?}\n}}", self.as_type_ref(), llvm_type)
    }
}

impl AsTypeRef for FunctionType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.fn_type.type_
    }
}
