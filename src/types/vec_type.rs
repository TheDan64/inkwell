use llvm_sys::core::{LLVMConstVector, LLVMConstNull, LLVMGetVectorSize};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use std::ffi::CStr;

use types::traits::AsTypeRef;
use types::Type;
use values::{BasicValue, PointerValue, VectorValue};

// REVIEW: vec_type() is impl for IntType & FloatType. Need to
// find out if it is valid for other types too. Maybe PointerType?
#[derive(Debug, PartialEq, Eq)]
pub struct VectorType {
    vec_type: Type,
}

impl VectorType {
    pub(crate) fn new(vector_type: LLVMTypeRef) -> Self {
        assert!(!vector_type.is_null());

        VectorType {
            vec_type: Type::new(vector_type),
        }
    }

    pub fn size(&self) -> u32 {
        unsafe {
            LLVMGetVectorSize(self.as_type_ref())
        }
    }

    // REVIEW:
    // TypeSafety v2 (GH Issue #8) could help here by constraining
    // sub-types to be the same across the board. For now, we could
    // have V just be the set of Int & Float and any others that
    // are valid for Vectors
    // REVIEW: Maybe we could make this use &self if the vector size
    // is stored as a const and the input values took a const size?
    // Something like: values: &[&V; self.size]. Doesn't sound possible though
    pub fn const_vector<V: BasicValue>(values: &[&V]) -> VectorValue {
        let mut values: Vec<LLVMValueRef> = values.iter()
                                                  .map(|val| val.as_value_ref())
                                                  .collect();
        let vec_value = unsafe {
            LLVMConstVector(values.as_mut_ptr(), values.len() as u32)
        };

        VectorValue::new(vec_value)
    }

    pub fn const_null_ptr(&self) -> PointerValue {
        self.vec_type.const_null_ptr()
    }

    pub fn const_null(&self) -> VectorValue {
        let null = unsafe {
            LLVMConstNull(self.as_type_ref())
        };

        VectorValue::new(null)
    }

    pub fn print_to_string(&self) -> &CStr {
        self.vec_type.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.vec_type.print_to_stderr()
    }

    pub fn get_undef(&self) -> VectorValue {
        VectorValue::new(self.vec_type.get_undef())
    }

    pub fn is_sized(&self) -> bool {
        self.vec_type.is_sized()
    }
}

impl AsTypeRef for VectorType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.vec_type.type_
    }
}
