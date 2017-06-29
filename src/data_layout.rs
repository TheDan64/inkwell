use llvm_sys::core::LLVMDisposeMessage;

use std::os::raw::c_char;

pub struct DataLayout {
    pub(crate) data_layout: *mut c_char,
}

impl DataLayout {
    pub(crate) fn new(data_layout: *mut c_char) -> DataLayout {
        assert!(!data_layout.is_null());

        DataLayout {
            data_layout: data_layout
        }
    }
}

impl Drop for DataLayout {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMessage(self.data_layout)
        }
    }
}
