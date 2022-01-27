use std::ffi::CStr;
use std::fmt;

use crate::support::{LLVMString, LLVMStringOrRaw};

#[derive(PartialEq, Eq)]
pub struct DataLayout<'a> {
    pub(crate) data_layout: LLVMStringOrRaw<'a>,
}

impl<'a> DataLayout<'a> {
    pub(crate) unsafe fn new_owned(data_layout: *const ::libc::c_char) -> DataLayout<'static> {
        debug_assert!(!data_layout.is_null());

        DataLayout {
            data_layout: LLVMStringOrRaw::owned(LLVMString::new(data_layout)),
        }
    }

    pub(crate) unsafe fn new_borrowed(data_layout: *const ::libc::c_char) -> Self {
        debug_assert!(!data_layout.is_null());

        DataLayout {
            data_layout: LLVMStringOrRaw::borrowed(data_layout),
        }
    }

    pub fn as_str(&self) -> &CStr {
        self.data_layout.as_str()
    }

    pub fn as_ptr(&self) -> *const ::libc::c_char {
        self.data_layout.as_ptr()
    }
}

impl<'a> fmt::Debug for DataLayout<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DataLayout")
            .field("address", &self.as_ptr())
            .field("repr", &self.as_str())
            .finish()
    }
}
