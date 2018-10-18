use llvm_sys::core::{LLVMCreateMemoryBufferWithContentsOfFile, LLVMCreateMemoryBufferWithSTDIN, LLVMCreateMemoryBufferWithMemoryRange, LLVMCreateMemoryBufferWithMemoryRangeCopy, LLVMGetBufferStart, LLVMGetBufferSize, LLVMDisposeMemoryBuffer};
use llvm_sys::prelude::LLVMMemoryBufferRef;
use llvm_sys::object::LLVMCreateObjectFile;

use object_file::ObjectFile;
use support::LLVMString;

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
        let path = path.to_str().expect("Did not find a valid Unicode path string");
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

    // REVIEW: Does this make more sense to take a byte array as input?
    /// This will create a new `MemoryBuffer` from the given input.
    ///
    /// This function is likely slightly cheaper than `create_from_memory_range_copy` since it intentionally
    /// leaks data to LLVM so that it doesn't have to reallocate. `create_from_memory_range_copy` may be removed
    /// in the future
    pub fn create_from_memory_range(input: &str, name: &str) -> Self {
        let input_c_string = CString::new(input).expect("Conversion to CString failed unexpectedly");
        let name_c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let memory_buffer = unsafe {
            LLVMCreateMemoryBufferWithMemoryRange(input_c_string.as_ptr(), input.len(), name_c_string.as_ptr(), false as i32)
        };

        // LLVM seems to want to take ownership of input_c_string, which is why we need to forget it
        // This originally was discovered when not forgetting it caused a subsequent as_slice call
        // to sometimes return partially garbage data
        // REVIEW: Does this apply to name_c_string as well?
        forget(input_c_string);

        MemoryBuffer::new(memory_buffer)
    }

    /// This will create a new `MemoryBuffer` from the given input.
    ///
    /// This function is likely slightly more expensive than `create_from_memory_range` since it does not leak
    /// data to LLVM, forcing LLVM to make a copy. This function may be removed in the future in favor of
    /// `create_from_memory_range`
    pub fn create_from_memory_range_copy(input: &str, name: &str) -> Self {
        let input_c_string = CString::new(input).expect("Conversion to CString failed unexpectedly");
        let name_c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let memory_buffer = unsafe {
            LLVMCreateMemoryBufferWithMemoryRangeCopy(input_c_string.as_ptr(), input.len(), name_c_string.as_ptr())
        };

        MemoryBuffer::new(memory_buffer)
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let start = LLVMGetBufferStart(self.memory_buffer);

            slice::from_raw_parts(start as *const _, self.get_size())
        }
    }

    pub fn get_size(&self) -> usize {
        unsafe {
            LLVMGetBufferSize(self.memory_buffer)
        }
    }

    // REVIEW: I haven't yet been able to find docs or other wrappers that confirm, but my suspicion
    // is that the method needs to take ownership of the MemoryBuffer... otherwise I see what looks like
    // a double free in valgrind when the MemoryBuffer drops so we are `forget`ting MemoryBuffer here
    // for now until we can confirm this is the correct thing to do
    pub fn create_object_file(self) -> Option<ObjectFile> {
        let object_file = unsafe {
            LLVMCreateObjectFile(self.memory_buffer)
        };

        forget(self);

        if object_file.is_null() {
            return None;
        }

        Some(ObjectFile::new(object_file))
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMemoryBuffer(self.memory_buffer);
        }
    }
}
