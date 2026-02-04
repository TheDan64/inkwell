use llvm_sys::core::{
    LLVMCreateMemoryBufferWithContentsOfFile, LLVMCreateMemoryBufferWithMemoryRange,
    LLVMCreateMemoryBufferWithMemoryRangeCopy, LLVMCreateMemoryBufferWithSTDIN, LLVMDisposeMemoryBuffer,
    LLVMGetBufferSize, LLVMGetBufferStart,
};
use llvm_sys::object::LLVMCreateBinary;
use llvm_sys::prelude::LLVMMemoryBufferRef;

use crate::context::Context;
use crate::object_file::BinaryFile;
use crate::support::{to_c_str, LLVMString};

use std::marker::PhantomData;
use std::path::Path;
use std::ptr;
use std::slice;

#[derive(Debug)]
pub struct MemoryBuffer<'a> {
    pub(crate) memory_buffer: LLVMMemoryBufferRef,
    _phantom: PhantomData<&'a [u8]>,
}

// The backing memory is owned and managed by LLVM, such that it is valid throughout the buffer's
// lifetime, hence 'static.
impl MemoryBuffer<'static> {
    /// Create a memory buffer from file content.
    ///
    /// Return `Err` with error message if failed.
    pub fn create_from_file(path: &Path) -> Result<Self, LLVMString> {
        let path = to_c_str(path.to_str().expect("Did not find a valid Unicode path string"));
        let mut memory_buffer = ptr::null_mut();
        let mut err_string = ptr::null_mut();

        let is_err = unsafe {
            LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), &mut memory_buffer, &mut err_string) == 1
        };

        if is_err {
            unsafe {
                return Err(LLVMString::new(err_string));
            }
        }

        unsafe { Ok(Self::new(memory_buffer)) }
    }

    /// Create a memory buffer from stdin.
    ///
    /// Return `Err` with error message if failed.
    pub fn create_from_stdin() -> Result<Self, LLVMString> {
        let mut memory_buffer = ptr::null_mut();
        let mut err_string = ptr::null_mut();

        let is_err = unsafe { LLVMCreateMemoryBufferWithSTDIN(&mut memory_buffer, &mut err_string) == 1 };

        if is_err {
            unsafe {
                return Err(LLVMString::new(err_string));
            }
        }

        unsafe { Ok(Self::new(memory_buffer)) }
    }

    /// Create a memory buffer copied from a byte slice with a trailing nul byte.
    ///
    /// # Panics
    ///
    /// Panics if the input byte slice does not terminate with a nul byte.
    pub fn create_from_memory_range_copy(input: &[u8], name: &str) -> Self {
        assert_eq!(
            input[input.len() - 1],
            b'\0',
            "input byte slice must terminate with a nul byte"
        );

        let name_c_string = to_c_str(name);

        let memory_buffer = unsafe {
            LLVMCreateMemoryBufferWithMemoryRangeCopy(
                input.as_ptr() as *const libc::c_char,
                // decremented since technically the nul byte is one past the end of the input.
                input.len() - 1,
                name_c_string.as_ptr(),
            )
        };

        unsafe { Self::new(memory_buffer) }
    }
}

impl<'a> MemoryBuffer<'a> {
    pub unsafe fn new(memory_buffer: LLVMMemoryBufferRef) -> Self {
        assert!(!memory_buffer.is_null());

        Self {
            memory_buffer,
            _phantom: PhantomData,
        }
    }

    pub fn as_mut_ptr(&self) -> LLVMMemoryBufferRef {
        self.memory_buffer
    }

    /// Create a memory buffer from a byte slice with a trailing nul byte.
    ///
    /// # Panics
    ///
    /// Panics if the input byte slice does not terminate with a nul byte.
    pub fn create_from_memory_range(input: &'a [u8], name: &str) -> Self {
        assert_eq!(
            input[input.len() - 1],
            b'\0',
            "input byte slice must terminate with a nul byte"
        );

        let name_c_string = to_c_str(name);

        let memory_buffer = unsafe {
            LLVMCreateMemoryBufferWithMemoryRange(
                input.as_ptr() as *const libc::c_char,
                // decremented since technically the nul byte is one past the end of the input.
                input.len() - 1,
                name_c_string.as_ptr(),
                false as i32, // guaranteed to have nul-terminator by CStr
            )
        };

        unsafe { MemoryBuffer::new(memory_buffer) }
    }

    /// Gets a byte slice of this [`MemoryBuffer`], containing the trailing nul byte.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let start = LLVMGetBufferStart(self.memory_buffer);

            // SAFETY: from LLVM `MemoryBuffer.h`:
            // "this interface guarantees you can read one character past the end of the file,
            // and that this character will read as '\0'."
            //
            // we include it here, so we can create a MemoryBuffer from the returned slice again.
            slice::from_raw_parts(start as *const _, self.get_size())
        }
    }

    /// Gets the byte size of this `MemoryBuffer`, counting the trailing nul byte.
    pub fn get_size(&self) -> usize {
        // buffer size does not include the trailing nul byte, hence incremented.
        unsafe { LLVMGetBufferSize(self.memory_buffer) + 1 }
    }

    /// Convert this [`MemoryBuffer`] and optional [`Context`] into a [`BinaryFile`].
    ///
    /// The context is required if the resulting file is LLVM IR.
    ///
    /// Return `Err` with error message if failed.
    pub fn create_binary_file(&self, context: Option<&Context>) -> Result<BinaryFile<'_>, LLVMString> {
        let context = context.map_or(ptr::null_mut(), |c| c.raw());
        let mut err_string = ptr::null_mut();

        let binary_file = unsafe { LLVMCreateBinary(self.memory_buffer, context, &mut err_string) };

        if binary_file.is_null() {
            unsafe {
                return Err(LLVMString::new(err_string));
            }
        }

        unsafe { Ok(BinaryFile::new(binary_file)) }
    }
}

impl<'a> Drop for MemoryBuffer<'a> {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMemoryBuffer(self.memory_buffer);
        }
    }
}
