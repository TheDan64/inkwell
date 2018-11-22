use llvm_sys::core::{LLVMMDNode, LLVMMDString, LLVMIsAMDNode, LLVMIsAMDString, LLVMGetMDString, LLVMGetMDNodeNumOperands, LLVMGetMDNodeOperands, LLVMGetMDKindID};
use llvm_sys::prelude::LLVMValueRef;

#[llvm_versions(7.0 => latest)]
use llvm_sys::prelude::LLVMMetadataRef;
#[llvm_versions(7.0 => latest)]
use llvm_sys::core::LLVMValueAsMetadata;

use support::LLVMString;
use values::traits::AsValueRef;
use values::{BasicValue, BasicMetadataValueEnum, Value};

use std::ffi::{CString, CStr};
use std::fmt;
use std::mem::forget;
use std::slice::from_raw_parts;

// TODOC: Varies by version
#[cfg(feature = "llvm3-6")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 12;
#[cfg(feature = "llvm3-7")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 14;
#[cfg(feature = "llvm3-8")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 18;
#[cfg(feature = "llvm3-9")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 20;
#[cfg(feature = "llvm4-0")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 22;
#[cfg(feature = "llvm5-0")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 23;
#[cfg(feature = "llvm6-0")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 25;
#[cfg(feature = "llvm7-0")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 25;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct MetadataValue {
    metadata_value: Value,
}

impl MetadataValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        unsafe {
            assert!(!LLVMIsAMDNode(value).is_null() || !LLVMIsAMDString(value).is_null());
        }

        MetadataValue {
            metadata_value: Value::new(value),
        }
    }

    #[llvm_versions(7.0 => latest)]
    pub(crate) fn as_metadata_ref(&self) -> LLVMMetadataRef {
        unsafe {
            LLVMValueAsMetadata(self.as_value_ref())
        }
    }

    // SubTypes: This can probably go away with subtypes
    pub fn is_node(&self) -> bool {
        unsafe {
            LLVMIsAMDNode(self.as_value_ref()) == self.as_value_ref()
        }
    }

    // SubTypes: This can probably go away with subtypes
    pub fn is_string(&self) -> bool {
        unsafe {
            LLVMIsAMDString(self.as_value_ref()) == self.as_value_ref()
        }
    }

    pub fn create_node(values: &[&BasicValue]) -> Self {
        let mut tuple_values: Vec<LLVMValueRef> = values.iter()
                                                        .map(|val| val.as_value_ref())
                                                        .collect();
        let metadata_value = unsafe {
            LLVMMDNode(tuple_values.as_mut_ptr(), tuple_values.len() as u32)
        };

        MetadataValue::new(metadata_value)
    }

    pub fn create_string(string: &str) -> Self {
        let c_string = CString::new(string).expect("Conversion to CString failed unexpectedly");

        let metadata_value = unsafe {
            LLVMMDString(c_string.as_ptr(), string.len() as u32)
        };

        MetadataValue::new(metadata_value)
    }

    pub fn get_string_value(&self) -> Option<&CStr> {
        if self.is_node() {
            return None;
        }

        let mut len = 0;
        let c_str = unsafe {
            CStr::from_ptr(LLVMGetMDString(self.as_value_ref(), &mut len))
        };

        Some(c_str)
    }

    // SubTypes: Node only one day
    pub fn get_node_size(&self) -> u32 {
        if self.is_string() {
            return 0;
        }

        unsafe {
            LLVMGetMDNodeNumOperands(self.as_value_ref())
        }
    }

    // SubTypes: Node only one day
    // REVIEW: BasicMetadataValueEnum only if you can put metadata in metadata...
    pub fn get_node_values(&self) -> Vec<BasicMetadataValueEnum> {
        if self.is_string() {
            return Vec::new();
        }

        let count = self.get_node_size();
        let mut raw_vec: Vec<LLVMValueRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let slice = unsafe {
            LLVMGetMDNodeOperands(self.as_value_ref(), ptr);

            from_raw_parts(ptr, count as usize)
        };

        slice.iter().map(|val| BasicMetadataValueEnum::new(*val)).collect()
    }

    // What is this even useful for
    pub fn get_kind_id(key: &str) -> u32 {
        unsafe {
            // REVIEW: Why does str give us *u8 but LLVM wants *i8?
            LLVMGetMDKindID(key.as_ptr() as *const i8, key.len() as u32)
        }
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.metadata_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.metadata_value.print_to_stderr()
    }

    pub fn replace_all_uses_with(&self, other: &MetadataValue) {
        self.metadata_value.replace_all_uses_with(other.as_value_ref())
    }
}

impl AsValueRef for MetadataValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.metadata_value.value
    }
}

impl fmt::Debug for MetadataValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut d = f.debug_struct("MetadataValue");
        d.field("address", &self.as_value_ref());

        if self.is_string() {
            d.field("value", &self.get_string_value().unwrap());
        } else {
            d.field("values", &self.get_node_values());
        }

        d.field("repr", &self.print_to_string());

        d.finish()
    }
}
