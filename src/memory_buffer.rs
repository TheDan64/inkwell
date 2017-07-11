use llvm_sys::prelude::{LLVMModuleRef, LLVMMemoryBufferRef};
use llvm_sys::core::{LLVMDisposeMessage, LLVMCreateMemoryBufferWithContentsOfFile, LLVMCreateMemoryBufferWithSTDIN, LLVMCreateMemoryBufferWithMemoryRange, LLVMCreateMemoryBufferWithMemoryRangeCopy, LLVMGetBufferStart, LLVMGetBufferSize, LLVMDisposeMemoryBuffer};

use std::ffi::{CString, CStr};
use std::mem::zeroed;

// REVIEW: This whole module is very untested
pub struct MemoryBuffer {
    memory_buffer: LLVMMemoryBufferRef
}

impl MemoryBuffer {
    pub(crate) fn new(memory_buffer: LLVMMemoryBufferRef) -> Self {
        assert!(!memory_buffer.is_null());

        MemoryBuffer {
            memory_buffer
        }
    }

    pub fn create_from_file(path: &str) -> Result<Self, String> {
        let c_string = CString::new(path).expect("Conversion to CString failed unexpectedly");
        let memory_buffer = 0 as *mut LLVMMemoryBufferRef;
        let err_str = unsafe { zeroed() };

        let return_code = unsafe {
            // REVIEW: Unclear why this expects *const i8 instead of *const u8
            LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr() as *const i8, memory_buffer, err_str)
        };

        // TODO: Verify 1 is error code (LLVM can be inconsistent)
        if return_code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(*err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(*err_str);

                rust_str
            };

            return Err(rust_str);
        }

        unsafe {
            Ok(MemoryBuffer::new(*memory_buffer))
        }
    }

    pub fn create_from_stdin() -> Result<Self, String> {
        let memory_buffer = 0 as *mut LLVMMemoryBufferRef;
        let err_str = unsafe { zeroed() };

        let return_code = unsafe {
            LLVMCreateMemoryBufferWithSTDIN(memory_buffer, err_str)
        };

        // TODO: Verify 1 is error code (LLVM can be inconsistent)
        if return_code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(*err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(*err_str);

                rust_str
            };

            return Err(rust_str);
        }

        unsafe {
            Ok(MemoryBuffer::new(*memory_buffer))
        }
    }

    pub fn create_from_memory_range(input: &str, name: &str) -> Self {
        let input_c_string = CString::new(input).expect("Conversion to CString failed unexpectedly");
        let name_c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let memory_buffer = unsafe {
            LLVMCreateMemoryBufferWithMemoryRange(input_c_string.as_ptr(), input.len(), name_c_string.as_ptr(), false as i32)
        };

        MemoryBuffer::new(memory_buffer)
    }

    pub fn create_from_memory_range_copy(input: &str, name: &str) -> Self {
        let input_c_string = CString::new(input).expect("Conversion to CString failed unexpectedly");
        let name_c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let memory_buffer = unsafe {
            LLVMCreateMemoryBufferWithMemoryRangeCopy(input_c_string.as_ptr(), input.len(), name_c_string.as_ptr())
        };

        MemoryBuffer::new(memory_buffer)
    }

    // REVIEW: I'm assuming this is borrowed data, but maybe it should be CString?
    pub fn as_slice(&self) -> &CStr {
        unsafe {
            let c_str = LLVMGetBufferStart(self.memory_buffer);

            CStr::from_ptr(c_str)
        }
    }

    pub fn get_size(&self) -> usize {
        unsafe {
            LLVMGetBufferSize(self.memory_buffer)
        }
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMemoryBuffer(self.memory_buffer)
        }
    }
}
