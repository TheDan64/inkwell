use llvm_sys::LLVMMemoryBuffer;
use llvm_sys::core::{
    LLVMCreateMemoryBufferWithContentsOfFile, LLVMCreateMemoryBufferWithMemoryRange,
    LLVMCreateMemoryBufferWithMemoryRangeCopy, LLVMCreateMemoryBufferWithSTDIN, LLVMDisposeMemoryBuffer,
    LLVMGetBufferSize, LLVMGetBufferStart,
};
use llvm_sys::object::LLVMCreateBinary;
use llvm_sys::prelude::LLVMMemoryBufferRef;

use crate::context::Context;
use crate::object_file::BinaryFile;
use crate::support::{LLVMString, assert_niche, to_c_str};

use std::marker::PhantomData;
use std::path::Path;
use std::ptr::{self, NonNull};
use std::slice;

#[repr(transparent)]
#[derive(Debug)]
pub struct MemoryBuffer<'a> {
    pub(crate) memory_buffer: NonNull<LLVMMemoryBuffer>,
    _phantom: PhantomData<&'a [u8]>,
}
const _: () = assert_niche::<MemoryBuffer>();

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

    /// Create a memory buffer copied from a byte slice.
    pub fn create_from_memory_range_copy(input: &[u8], name: &str) -> Self {
        let name_c_string = to_c_str(name);

        let memory_buffer = unsafe {
            LLVMCreateMemoryBufferWithMemoryRangeCopy(
                input.as_ptr() as *const libc::c_char,
                input.len(),
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
            memory_buffer: unsafe { NonNull::new_unchecked(memory_buffer) },
            _phantom: PhantomData,
        }
    }

    pub fn as_mut_ptr(&self) -> LLVMMemoryBufferRef {
        self.memory_buffer.as_ptr()
    }

    /// Create a memory buffer from a byte slice.
    pub fn create_from_memory_range(input: &'a [u8], name: &str) -> Self {
        let name_c_string = to_c_str(name);

        let memory_buffer = unsafe {
            /* LLVMCreateMemoryBufferWithMemoryRange does not expect a null-terminated string.
             * StringRef documentation: https://llvm.org/doxygen/StringRef_8h_source.html#l00132
             * ```
             * Get a pointer to the start of the string (which may not be null
             * terminated).
             * ```
             */
            LLVMCreateMemoryBufferWithMemoryRange(
                input.as_ptr() as *const libc::c_char,
                input.len(),
                name_c_string.as_ptr(),
                /* If this is `true`, LLVM will expect a null-terminator. If it is false, the null-terminator is not
                 * necessary.
                 * https://llvm.org/doxygen/IR_2Core_8cpp_source.html#l04679
                 * ```
                 * LLVMMemoryBufferRef LLVMCreateMemoryBufferWithMemoryRange(
                 *    const char *InputData,
                 *    size_t InputDataLength,
                 *    const char *BufferName,
                 *    LLVMBool RequiresNullTerminator) {
                 *
                 * return wrap(MemoryBuffer::getMemBuffer(StringRef(InputData, InputDataLength),
                 *                                        StringRef(BufferName),
                 *                                        RequiresNullTerminator).release());
                 * }
                 * ```
                 * `MemoryBuffer::getMemBuffer` documentation.
                 * https://llvm.org/doxygen/classllvm_1_1MemoryBuffer.html#a0f68098734d6d3b451aacf5b38a67131
                 * ```
                 * Note that InputData must be null terminated if RequiresNullTerminator is true.
                 * ```
                 */
                false as libc::c_int,
            )
        };

        unsafe { MemoryBuffer::new(memory_buffer) }
    }

    /// Gets a byte slice of this [`MemoryBuffer`], containing the trailing nul byte.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let start = LLVMGetBufferStart(self.as_mut_ptr());

            // SAFETY: from LLVM `MemoryBuffer.h`:
            // "this interface guarantees you can read one character past the end of the file,
            // and that this character will read as '\0'."
            //
            // we include it here, so we can create a MemoryBuffer from the returned slice again.
            slice::from_raw_parts(start as *const _, self.get_size())
        }
    }

    /// Gets the byte size of this `MemoryBuffer`.
    pub fn get_size(&self) -> usize {
        unsafe { LLVMGetBufferSize(self.as_mut_ptr()) }
    }

    /// Convert this [`MemoryBuffer`] and optional [`Context`] into a [`BinaryFile`].
    ///
    /// The context is required if the resulting file is LLVM IR.
    ///
    /// Return `Err` with error message if failed.
    pub fn create_binary_file(&self, context: Option<&Context>) -> Result<BinaryFile<'_>, LLVMString> {
        let context = context.map_or(ptr::null_mut(), |c| c.raw());
        let mut err_string = ptr::null_mut();

        let binary_file = unsafe { LLVMCreateBinary(self.as_mut_ptr(), context, &mut err_string) };

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
            LLVMDisposeMemoryBuffer(self.as_mut_ptr());
        }
    }
}
