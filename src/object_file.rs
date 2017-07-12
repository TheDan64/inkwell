use llvm_sys::object::{LLVMDisposeObjectFile, LLVMObjectFileRef};

// TODO: Iterator for LLVMSectionIteratorRef

pub struct ObjectFile {
    object_file: LLVMObjectFileRef
}

impl ObjectFile {
    pub(crate) fn new(object_file: LLVMObjectFileRef) -> Self {
        assert!(!object_file.is_null());

        ObjectFile {
            object_file
        }
    }
}

impl Drop for ObjectFile {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeObjectFile(self.object_file)
        }
    }
}
