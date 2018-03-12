use llvm_sys::core::LLVMDisposeMessage;

use std::cell::Cell;
use std::ffi::CStr;
use std::fmt;

// REVIEW: We should consider making this more generic. DataLayout could just be
// a newtype on a LLVMMessage struct

#[derive(Eq)]
pub struct DataLayout {
    pub(crate) data_layout: Cell<*mut i8>,
}

impl DataLayout {
    pub(crate) fn new(data_layout: *mut i8) -> DataLayout {
        assert!(!data_layout.is_null());

        DataLayout {
            data_layout: Cell::new(data_layout),
        }
    }

    pub fn as_str(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(self.data_layout.get())
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
            .field("address", &self.data_layout.get())
            .field("repr", &self.as_str())
            .finish()
    }
}

impl Drop for DataLayout {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMessage(self.data_layout.get())
        }
    }
}
