use llvm_sys::core::{LLVMIsAMDNode, LLVMIsAMDString, LLVMGetMDString, LLVMGetMDNodeNumOperands, LLVMGetMDNodeOperands};
use llvm_sys::prelude::LLVMValueRef;

#[llvm_versions(7.0..=latest)]
use llvm_sys::prelude::LLVMMetadataRef;
#[llvm_versions(7.0..=latest)]
use llvm_sys::core::LLVMValueAsMetadata;

use crate::support::LLVMString;
use crate::values::traits::AsValueRef;
use crate::values::{BasicMetadataValueEnum, Value};

use std::ffi::CStr;
use std::fmt;

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
#[cfg(feature = "llvm8-0")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 26;
#[cfg(feature = "llvm9-0")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 28;
#[cfg(any(feature = "llvm10-0", feature = "llvm11-0"))]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 30;
#[cfg(feature = "llvm12-0")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 31;
#[cfg(feature = "llvm13-0")]
pub const FIRST_CUSTOM_METADATA_KIND_ID: u32 = 31;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct MetadataValue<'ctx> {
    metadata_value: Value<'ctx>,
}

impl<'ctx> MetadataValue<'ctx> {
    pub(crate) unsafe fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());
        assert!(!LLVMIsAMDNode(value).is_null() || !LLVMIsAMDString(value).is_null());

        MetadataValue {
            metadata_value: Value::new(value),
        }
    }

    #[llvm_versions(7.0..=latest)]
    pub(crate) fn as_metadata_ref(self) -> LLVMMetadataRef {
        unsafe {
            LLVMValueAsMetadata(self.as_value_ref())
        }
    }

    // SubTypes: This can probably go away with subtypes
    pub fn is_node(self) -> bool {
        unsafe {
            LLVMIsAMDNode(self.as_value_ref()) == self.as_value_ref()
        }
    }

    // SubTypes: This can probably go away with subtypes
    pub fn is_string(self) -> bool {
        unsafe {
            LLVMIsAMDString(self.as_value_ref()) == self.as_value_ref()
        }
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
    pub fn get_node_size(self) -> u32 {
        if self.is_string() {
            return 0;
        }

        unsafe {
            LLVMGetMDNodeNumOperands(self.as_value_ref())
        }
    }

    // SubTypes: Node only one day
    // REVIEW: BasicMetadataValueEnum only if you can put metadata in metadata...
    pub fn get_node_values(self) -> Vec<BasicMetadataValueEnum<'ctx>> {
        if self.is_string() {
            return Vec::new();
        }

        let count = self.get_node_size() as usize;
        let mut vec: Vec<LLVMValueRef> = Vec::with_capacity(count);
        let ptr = vec.as_mut_ptr();

        unsafe {
            LLVMGetMDNodeOperands(self.as_value_ref(), ptr);

            vec.set_len(count)
        };

        vec.iter()
            .map(|val| unsafe { BasicMetadataValueEnum::new(*val) })
            .collect()
    }

    pub fn print_to_string(self) -> LLVMString {
        self.metadata_value.print_to_string()
    }

    pub fn print_to_stderr(self) {
        self.metadata_value.print_to_stderr()
    }

    pub fn replace_all_uses_with(self, other: &MetadataValue<'ctx>) {
        self.metadata_value.replace_all_uses_with(other.as_value_ref())
    }
}

impl AsValueRef for MetadataValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.metadata_value.value
    }
}

impl fmt::Debug for MetadataValue<'_> {
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
