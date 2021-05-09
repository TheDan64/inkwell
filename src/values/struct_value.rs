use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use crate::types::StructType;
use crate::values::traits::AsValueRef;
use crate::values::{InstructionValue, Value};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct StructValue<'ctx> {
    struct_value: Value<'ctx>,
}

impl<'ctx> StructValue<'ctx> {
    pub(crate) unsafe fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        StructValue {
            struct_value: Value::new(value),
        }
    }

    /// Gets the name of a `StructValue`. If the value is a constant, this will
    /// return an empty string.
    pub fn get_name(&self) -> &CStr {
        self.struct_value.get_name()
    }

    pub fn get_type(self) -> StructType<'ctx> {
        unsafe {
            StructType::new(self.struct_value.get_type())
        }
    }

    pub fn is_null(self) -> bool {
        self.struct_value.is_null()
    }

    pub fn is_undef(self) -> bool {
        self.struct_value.is_undef()
    }

    pub fn print_to_stderr(self) {
        self.struct_value.print_to_stderr()
    }

    pub fn as_instruction(self) -> Option<InstructionValue<'ctx>> {
        self.struct_value.as_instruction()
    }

    pub fn replace_all_uses_with(self, other: StructValue<'ctx>) {
        self.struct_value.replace_all_uses_with(other.as_value_ref())
    }
}

impl AsValueRef for StructValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.struct_value.value
    }
}
