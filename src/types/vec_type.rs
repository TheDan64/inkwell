use llvm_sys::core::{LLVMConstVector, LLVMConstNull, LLVMGetVectorSize};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use AddressSpace;
use support::LLVMString;
use types::{ArrayType, BasicTypeEnum, Type, traits::AsTypeRef, FunctionType, PointerType};
use values::{BasicValue, PointerValue, VectorValue, IntValue};

// REVIEW: vec_type() is impl for IntType & FloatType. Need to
// find out if it is valid for other types too. Maybe PointerType?
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

    // REVIEW: Can be unsized if inner type is opaque struct
    pub fn is_sized(&self) -> bool {
        self.vec_type.is_sized()
    }

    // TODO: impl only for VectorType<!StructType<Opaque>>
    // REVIEW: What about Opaque struct hiding in deeper levels?
    // like VectorType<ArrayType<StructType<Opaque>>>?
    pub fn size_of(&self) -> Option<IntValue> {
        if self.is_sized() {
            return Some(self.vec_type.size_of())
        }

        None
    }

    fn get_alignment(&self) -> IntValue {
        self.vec_type.get_alignment()
    }

    pub fn get_size(&self) -> u32 {
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
    pub fn const_vector<V: BasicValue>(values: &[V]) -> VectorValue {
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

    pub fn print_to_string(&self) -> LLVMString {
        self.vec_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm5-0")))]
    pub fn print_to_stderr(&self) {
        self.vec_type.print_to_stderr()
    }

    pub fn get_undef(&self) -> VectorValue {
        VectorValue::new(self.vec_type.get_undef())
    }

    // SubType: VectorType<BT> -> BT?
    pub fn get_element_type(&self) -> BasicTypeEnum {
        self.vec_type.get_element_type()
    }

    pub fn vec_type(&self, size: u32) -> VectorType {
        self.vec_type.vec_type(size)
    }

    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.vec_type.ptr_type(address_space)
    }

    pub fn fn_type(&self, param_types: &[BasicTypeEnum], is_var_args: bool) -> FunctionType {
        self.vec_type.fn_type(param_types, is_var_args)
    }

    pub fn array_type(&self, size: u32) -> ArrayType {
        self.vec_type.array_type(size)
    }
}

impl AsTypeRef for VectorType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.vec_type.type_
    }
}
