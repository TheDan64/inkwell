mod array_value;
mod enums;
mod float_value;
mod fn_value;
mod generic_value;
mod instruction_value;
mod int_value;
mod metadata_value;
mod phi_value;
mod ptr_value;
mod struct_value;
mod traits;
mod vec_value;

pub use values::array_value::ArrayValue;
pub use values::enums::{AnyValueEnum, AggregateValueEnum, BasicValueEnum, BasicMetadataValueEnum};
pub use values::float_value::FloatValue;
pub use values::fn_value::FunctionValue;
pub use values::generic_value::GenericValue;
pub use values::instruction_value::{InstructionValue, InstructionOpcode};
pub use values::int_value::IntValue;
pub use values::metadata_value::{MetadataValue, FIRST_CUSTOM_METADATA_KIND_ID};
pub use values::phi_value::PhiValue;
pub use values::ptr_value::PointerValue;
pub use values::struct_value::StructValue;
pub use values::traits::{AnyValue, AggregateValue, BasicValue};
pub use values::vec_value::VectorValue;
pub(crate) use values::traits::AsValueRef;

use llvm_sys::core::{LLVMGetValueName, LLVMIsConstant, LLVMIsNull, LLVMIsUndef, LLVMPrintTypeToString, LLVMPrintValueToString, LLVMSetGlobalConstant, LLVMSetValueName, LLVMTypeOf, LLVMDumpValue, LLVMIsAInstruction, LLVMGetMetadata, LLVMHasMetadata, LLVMSetMetadata, LLVMReplaceAllUsesWith};
use llvm_sys::prelude::{LLVMValueRef, LLVMTypeRef};

use std::ffi::{CString, CStr};
use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy)]
struct Value {
    value: LLVMValueRef,
}

impl Value {
    pub(crate) fn new(value: LLVMValueRef) -> Value {
        debug_assert!(!value.is_null(), "This should never happen since containing struct should check null ptrs");

        Value {
            value: value
        }
    }

    fn is_instruction(&self) -> bool {
        unsafe {
            !LLVMIsAInstruction(self.value).is_null()
        }
    }

    fn as_instruction(&self) -> Option<InstructionValue> {
        if !self.is_instruction() {
            return None;
        }

        Some(InstructionValue::new(self.value))
    }

    fn is_null(&self) -> bool {
        unsafe {
            LLVMIsNull(self.value) == 1
        }
    }

    fn set_global_constant(&self, num: i32) { // REVIEW: Need better name for this arg
        unsafe {
            LLVMSetGlobalConstant(self.value, num)
        }
    }

    // TODOC: According to https://stackoverflow.com/questions/21593752/llvm-how-to-pass-a-name-to-constantint
    // you can't use set_name name on a constant(by can't, I mean it wont do anything), unless it's also a global.
    // So, you can set names on variables (ie a function parameter)
    // REVIEW: It'd be great if we could encode this into the type system somehow
    fn set_name(&self, name: &str) {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetValueName(self.value, c_string.as_ptr());
        }
    }

    fn get_name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetValueName(self.value))
        }
    }

    fn is_undef(&self) -> bool {
        unsafe {
            LLVMIsUndef(self.value) == 1
        }
    }

    fn get_type(&self) -> LLVMTypeRef {
        unsafe {
            LLVMTypeOf(self.value)
        }
    }

    fn print_to_string(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.value))
        }
    }

    fn print_to_stderr(&self) {
        unsafe {
            LLVMDumpValue(self.value)
        }
    }

    fn has_metadata(&self) -> bool {
        unsafe {
            LLVMHasMetadata(self.value) == 1
        }
    }

    // SubTypes: -> Option<MetadataValue<Node>>
    // TODOC: This always returns a metadata node, which can be used to get its node values
    fn get_metadata(&self, kind_id: u32) -> Option<MetadataValue> {
        let metadata_value = unsafe {
            LLVMGetMetadata(self.value, kind_id)
        };

        if metadata_value.is_null() {
            return None;
        }

        Some(MetadataValue::new(metadata_value))
    }

    fn set_metadata(&self, metadata: &MetadataValue, kind_id: u32) {
        unsafe {
            LLVMSetMetadata(self.value, kind_id, metadata.as_value_ref())
        }
    }

    // REVIEW: I think this is memory safe, though it may result in an IR error
    // if used incorrectly, which is OK.
    fn replace_all_uses_with(&self, other: LLVMValueRef) {
        unsafe {
            LLVMReplaceAllUsesWith(self.value, other)
        }
    }

    // REVIEW: Remove?
    // fn get_type_kind(&self) -> LLVMTypeKind {
    //     (*self.get_type()).as_llvm_type_ref().get_kind()
    // }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = self.print_to_string();
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.value) == 1
        };
        let is_null = self.is_null();
        let is_undef = self.is_undef();

        write!(f, "Value {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_null: {:?}\n    is_undef: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.value, is_const, is_null, is_undef, llvm_value, llvm_type)
    }
}
