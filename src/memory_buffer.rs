use llvm_sys::core::{LLVMCreateMemoryBufferWithContentsOfFile, LLVMCreateMemoryBufferWithSTDIN, LLVMCreateMemoryBufferWithMemoryRange, LLVMCreateMemoryBufferWithMemoryRangeCopy, LLVMGetBufferStart, LLVMGetBufferSize, LLVMDisposeMemoryBuffer};
use llvm_sys::prelude::LLVMMemoryBufferRef;
use llvm_sys::object::LLVMCreateObjectFile;

use crate::object_file::ObjectFile;
use crate::support::LLVMString;

use std::ffi::CString;
use std::mem::{forget, zeroed};
use std::path::Path;
use std::ptr;
use std::slice;

#[derive(Debug)]
pub struct MemoryBuffer {
    pub(crate) memory_buffer: LLVMMemoryBufferRef
}

impl MemoryBuffer {
    pub(crate) fn new(memory_buffer: LLVMMemoryBufferRef) -> Self {
        assert!(!memory_buffer.is_null());

        MemoryBuffer {
            memory_buffer
        }
    }

    pub fn create_from_file(path: &Path) -> Result<Self, LLVMString> {
        let path = CString::new(path.to_str().expect("Did not find a valid Unicode path string")).expect("Failed to convert to CString");
        let mut memory_buffer = ptr::null_mut();
        let mut err_string = unsafe { zeroed() };

        let return_code = unsafe {
            // REVIEW: Unclear why this expects *const i8 instead of *const u8
            LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr() as *const i8, &mut memory_buffer, &mut err_string)
        };

        // TODO: Verify 1 is error code (LLVM can be inconsistent)
        if return_code == 1 {
            return Err(LLVMString::new(err_string));
        }

        Ok(MemoryBuffer::new(memory_buffer))
    }

    pub fn create_from_stdin() -> Result<Self, LLVMString> {
        let mut memory_buffer = ptr::null_mut();
        let mut err_string = unsafe { zeroed() };

        let return_code = unsafe {
            LLVMCreateMemoryBufferWithSTDIN(&mut memory_buffer, &mut err_string)
        };

        // TODO: Verify 1 is error code (LLVM can be inconsistent)
        if return_code == 1 {
            return Err(LLVMString::new(err_string));
        }

        Ok(MemoryBuffer::new(memory_buffer))
    }

    /// This function is likely slightly cheaper than `create_from_memory_range_copy` since it intentionally
    /// leaks data to LLVM so that it doesn't have to reallocate. `create_from_memory_range_copy` may be removed
    /// in the future
    pub fn create_from_memory_range(input: &[u8], name: &str) -> Self {
        let name_c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let memory_buffer = unsafe {
            LLVMCreateMemoryBufferWithMemoryRange(input.as_ptr() as *const i8, input.len(), name_c_string.as_ptr(), false as i32)
        };

        MemoryBuffer::new(memory_buffer)
    }

    /// This will create a new `MemoryBuffer` from the given input.
    ///
    /// This function is likely slightly more expensive than `create_from_memory_range` since it does not leak
    /// data to LLVM, forcing LLVM to make a copy. This function may be removed in the future in favor of
    /// `create_from_memory_range`
    pub fn create_from_memory_range_copy(input: &[u8], name: &str) -> Self {
        let name_c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let memory_buffer = unsafe {
            LLVMCreateMemoryBufferWithMemoryRangeCopy(input.as_ptr() as *const i8, input.len(), name_c_string.as_ptr())
        };

        MemoryBuffer::new(memory_buffer)
    }

    /// Gets a byte slice of this `MemoryBuffer`.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let start = LLVMGetBufferStart(self.memory_buffer);

            slice::from_raw_parts(start as *const _, self.get_size())
        }
    }

    /// Gets the byte size of this `MemoryBuffer`.
    pub fn get_size(&self) -> usize {
        unsafe {
            LLVMGetBufferSize(self.memory_buffer)
        }
    }

    /// Convert this `MemoryBuffer` into an `ObjectFile`. LLVM does not currently
    /// provide any way to determine the cause of error if conversion fails.
    pub fn create_object_file(self) -> Result<ObjectFile, ()> {
        let object_file = unsafe {
            LLVMCreateObjectFile(self.memory_buffer)
        };

        forget(self);

        if object_file.is_null() {
            return Err(());
        }

        Ok(ObjectFile::new(object_file))
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMemoryBuffer(self.memory_buffer);
        }
    }
}
