use llvm_sys::core::{LLVMIsConstant, LLVMIsAConstantArray, LLVMIsAConstantDataArray};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;
use std::fmt;

use types::ArrayType;
use values::traits::AsValueRef;
use values::{Value, InstructionValue};

#[derive(PartialEq, Eq)]
pub struct ArrayValue {
    array_value: Value
}

impl ArrayValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        ArrayValue {
            array_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.array_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.array_value.set_name(name);
    }

    pub fn get_type(&self) -> ArrayType {
        ArrayType::new(self.array_value.get_type())
    }

    pub fn is_null(&self) -> bool {
        self.array_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.array_value.is_undef()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.array_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.array_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.array_value.as_instruction()
    }
}

impl AsValueRef for ArrayValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.array_value.value
    }
}

impl fmt::Debug for ArrayValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = self.print_to_string();
        let llvm_type = self.get_type();
        let name = self.get_name();
        let is_const = unsafe {
            LLVMIsConstant(self.as_value_ref()) == 1
        };
        let is_null = self.is_null();
        let is_const_array = unsafe {
            !LLVMIsAConstantArray(self.as_value_ref()).is_null()
        };
        let is_const_data_array = unsafe {
            !LLVMIsAConstantDataArray(self.as_value_ref()).is_null()
        };

        write!(f, "Value {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_const_array: {:?}\n    is_const_data_array: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.as_value_ref(), is_const, is_const_array, is_const_data_array, is_null, llvm_value, llvm_type.print_to_string())
    }
}
