use std::ffi::CStr;
use std::fmt;

use crate::support::LLVMString;

/// A data layout string, as attached to a module or described by a `TargetData`.
#[derive(Eq)]
pub struct DataLayout {
    pub(crate) data_layout: LLVMString,
}

impl DataLayout {
    pub(crate) unsafe fn new_owned(data_layout: *const ::libc::c_char) -> DataLayout {
        unsafe {
            debug_assert!(!data_layout.is_null());

            DataLayout {
                data_layout: LLVMString::new(data_layout),
            }
        }
    }

    pub fn as_str(&self) -> &CStr {
        &self.data_layout
    }

    pub fn as_ptr(&self) -> *const ::libc::c_char {
        self.data_layout.ptr.as_ptr()
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
