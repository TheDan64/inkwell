use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use support::LLVMString;
use types::StructType;
use values::traits::AsValueRef;
use values::{InstructionValue, Value, MetadataValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct StructValue {
    struct_value: Value
}

impl StructValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        StructValue {
            struct_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.struct_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.struct_value.set_name(name);
    }

    pub fn get_type(&self) -> StructType {
        StructType::new(self.struct_value.get_type())
    }

    pub fn is_null(&self) -> bool {
        self.struct_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.struct_value.is_undef()
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.struct_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.struct_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.struct_value.as_instruction()
    }

    pub fn has_metadata(&self) -> bool {
        self.struct_value.has_metadata()
    }

    pub fn get_metadata(&self, kind_id: u32) -> Option<MetadataValue> {
        self.struct_value.get_metadata(kind_id)
    }

    pub fn set_metadata(&self, metadata: MetadataValue, kind_id: u32) {
        self.struct_value.set_metadata(metadata, kind_id)
    }

    pub fn replace_all_uses_with(&self, other: StructValue) {
        self.struct_value.replace_all_uses_with(other.as_value_ref())
    }
}

impl AsValueRef for StructValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.struct_value.value
    }
}
