use llvm_sys::core::{LLVMConstArray, LLVMConstNull, LLVMGetArrayLength};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use {AddressSpace, LLVMString};
use context::ContextRef;
use types::traits::AsTypeRef;
use types::{Type, BasicType, PointerType, FunctionType};
use values::{BasicValue, ArrayValue, PointerValue, IntValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ArrayType {
    array_type: Type,
}

impl ArrayType {
    pub(crate) fn new(array_type: LLVMTypeRef) -> Self {
        assert!(!array_type.is_null());

        ArrayType {
            array_type: Type::new(array_type),
        }
    }

    // REVIEW: Can be unsized if inner type is opaque struct
    pub fn is_sized(&self) -> bool {
        self.array_type.is_sized()
    }

    // TODO: impl only for ArrayType<!StructType<Opaque>>
    pub fn size_of(&self) -> Option<IntValue> {
        if self.is_sized() {
            return Some(self.array_type.size_of())
        }

        None
    }

    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.array_type.ptr_type(address_space)
    }

    pub fn get_context(&self) -> ContextRef {
        self.array_type.get_context()
    }

    pub fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
        self.array_type.fn_type(param_types, is_var_args)
    }

    pub fn array_type(&self, size: u32) -> ArrayType {
        self.array_type.array_type(size)
    }

    pub fn const_array<V: BasicValue>(&self, values: &[V]) -> ArrayValue {
        let mut values: Vec<LLVMValueRef> = values.iter()
                                                  .map(|val| val.as_value_ref())
                                                  .collect();
        let value = unsafe {
            LLVMConstArray(self.as_type_ref(), values.as_mut_ptr(), values.len() as u32)
        };

        ArrayValue::new(value)
    }

    pub fn const_null_ptr(&self) -> PointerValue {
        self.array_type.const_null_ptr()
    }

    pub fn const_null(&self) -> ArrayValue {
        let null = unsafe {
            LLVMConstNull(self.as_type_ref())
        };

        ArrayValue::new(null)
    }

    pub fn len(&self) -> u32 {
        unsafe {
            LLVMGetArrayLength(self.as_type_ref())
        }
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.array_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm5-0")))]
    pub fn print_to_stderr(&self) {
        self.array_type.print_to_stderr()
    }

    pub fn get_undef(&self) -> ArrayValue {
        ArrayValue::new(self.array_type.get_undef())
    }
}

impl AsTypeRef for ArrayType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.array_type.type_
    }
}
