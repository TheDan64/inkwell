use std::ffi::CStr;
use std::fmt;

use support::{LLVMString, LLVMStringOrRaw};

#[derive(Eq)]
pub struct DataLayout {
    pub(crate) data_layout: LLVMStringOrRaw,
}

impl DataLayout {
    pub(crate) fn new_owned(data_layout: *const i8) -> DataLayout {
        debug_assert!(!data_layout.is_null());

        DataLayout {
            data_layout: LLVMStringOrRaw::Owned(LLVMString::new(data_layout)),
        }
    }

    pub(crate) fn new_borrowed(data_layout: *const i8) -> DataLayout {
        debug_assert!(!data_layout.is_null());

        DataLayout {
            data_layout: LLVMStringOrRaw::Borrowed(data_layout),
        }
    }

    pub fn as_str(&self) -> &CStr {
        self.data_layout.as_str()
    }

    pub fn as_ptr(&self) -> *const i8 {
        match self.data_layout {
            LLVMStringOrRaw::Owned(ref llvm_string) => llvm_string.ptr,
            LLVMStringOrRaw::Borrowed(ptr) => ptr,
        }
    }
}

impl PartialEq for DataLayout {
    fn eq(&self, other: &DataLayout) -> bool {
        self.as_str() == other.as_str()
    }
}

impl fmt::Debug for DataLayout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DataLayout")
            .field("address", &self.as_ptr())
            .field("repr", &self.as_str())
            .finish()
    }
}
