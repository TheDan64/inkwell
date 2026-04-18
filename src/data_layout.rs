use std::fmt;
use std::{ffi::CStr, ptr::NonNull};

use crate::support::{LLVMString, LLVMStringOrRaw};

#[derive(Eq)]
pub struct DataLayout {
    pub(crate) data_layout: LLVMStringOrRaw,
}

impl DataLayout {
    pub(crate) unsafe fn new_owned(data_layout: *const ::libc::c_char) -> DataLayout {
        unsafe {
            debug_assert!(!data_layout.is_null());

            DataLayout {
                data_layout: LLVMStringOrRaw::Owned(LLVMString::new(data_layout)),
            }
        }
    }

    pub(crate) unsafe fn new_borrowed(data_layout: *const ::libc::c_char) -> DataLayout {
        debug_assert!(!data_layout.is_null());

        DataLayout {
            data_layout: LLVMStringOrRaw::Borrowed(unsafe { NonNull::new_unchecked(data_layout.cast_mut()) }),
        }
    }

    pub fn as_str(&self) -> &CStr {
        self.data_layout.as_str()
    }

    pub fn as_ptr(&self) -> *const ::libc::c_char {
        match self.data_layout {
            LLVMStringOrRaw::Owned(ref llvm_string) => llvm_string.ptr.as_ptr(),
            LLVMStringOrRaw::Borrowed(ptr) => ptr.as_ptr().cast_const(),
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
