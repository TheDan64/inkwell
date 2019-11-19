use std::mem::MaybeUninit;
use std::ffi::CString;
use std::marker::PhantomData;

use llvm_sys::object::{LLVMBinaryRef, LLVMBinaryType};

use crate::support::LLVMString;
use crate::memory_buffer::MemoryBuffer;
use crate::context::{Context, ContextRef};

use super::{SectionIterator, SymbolIterator};

/// Alias for compatibility with object_file
pub type BinaryRef = LLVMBinaryRef;

#[llvm_enum(LLVMBinaryType)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryType {
    #[llvm_variant(LLVMBinaryTypeArchive)]
    Archive,
    #[llvm_variant(LLVMBinaryTypeMachOUniversalBinary)]
    MachOUniversalBinary,
    #[llvm_variant(LLVMBinaryTypeCOFFImportFile)]
    COFFImportFile,
    #[llvm_variant(LLVMBinaryTypeIR)]
    IR,
    #[llvm_variant(LLVMBinaryTypeWinRes)]
    WinRes,
    #[llvm_variant(LLVMBinaryTypeCOFF)]
    COFF,
    #[llvm_variant(LLVMBinaryTypeELF32L)]
    ELF32L,
    #[llvm_variant(LLVMBinaryTypeELF32B)]
    ELF32B,
    #[llvm_variant(LLVMBinaryTypeELF64L)]
    ELF64L,
    #[llvm_variant(LLVMBinaryTypeELF64B)]
    ELF64B,
    #[llvm_variant(LLVMBinaryTypeMachO32L)]
    MachO32L,
    #[llvm_variant(LLVMBinaryTypeMachO32B)]
    MachO32B,
    #[llvm_variant(LLVMBinaryTypeMachO64L)]
    MachO64L,
    #[llvm_variant(LLVMBinaryTypeMachO64B)]
    MachO64B,
    #[llvm_variant(LLVMBinaryTypeWasm)]
    Wasm,
}

#[derive(Debug)]
pub struct Binary<'ctx> {
    binary: LLVMBinaryRef,
    buffer: Option<MemoryBuffer>,
    _marker: PhantomData<&'ctx Context>,
}
impl<'ctx> Binary<'ctx> {
    pub fn create(buffer: MemoryBuffer, context: ContextRef<'ctx>) -> Result<Self, LLVMString> {
        use llvm_sys::object::LLVMCreateBinary;

        let mut err_string = MaybeUninit::uninit();
        let binary = unsafe {
            LLVMCreateBinary(buffer.memory_buffer, context.context, err_string.as_mut_ptr())
        };
        if binary.is_null() {
            let err_str = unsafe { err_string.assume_init() };
            return Err(LLVMString::new(err_str));
        }
        Ok(Binary {
            binary,
            buffer: Some(buffer),
            _marker: PhantomData,
        })
    }

    /// Returns the type of binary file this is
    pub fn get_type(&self) -> BinaryType {
        use llvm_sys::object::LLVMBinaryGetType;

        let ty = unsafe {
            LLVMBinaryGetType(self.binary)
        };

        ty.into()
    }

    /// For a Mach-O universal binary file, this function retrieves the object
    /// file corresponding to the given architecture if it is present, and returns
    /// it as a new binary
    pub fn macho_universal_binary_copy_object_for_arch(&self, arch: &str) -> Result<Self, LLVMString> {
        use llvm_sys::object::LLVMMachOUniversalBinaryCopyObjectForArch;

        let mut err_string = MaybeUninit::uninit();
        let binary = unsafe {
            let arch_c_string = CString::new(arch).expect("Conversion to CString failed unexpectedly");
            let bytes = arch_c_string.as_bytes();
            let arch_ptr = bytes.as_ptr() as *const ::libc::c_char;
            let arch_len = bytes.len();
            LLVMMachOUniversalBinaryCopyObjectForArch(self.binary, arch_ptr, arch_len, err_string.as_mut_ptr())
        };
        if binary.is_null() {
            let err_str = unsafe { err_string.assume_init() };
            return Err(LLVMString::new(err_str));
        }
        Ok(Binary {
            binary,
            buffer: None,
            _marker: PhantomData,
        })

    }

    /// Retrieve a copy of the section iterator for this object file
    pub fn sections<'a>(&'a self) -> SectionIterator<'a> {
        SectionIterator::new(self.binary)
    }

    /// Retrieve a copy of the symbol iterator for this object file
    pub fn symbols<'a>(&'a self) -> SymbolIterator<'a> {
        SymbolIterator::new(self.binary)
    }
}

impl<'ctx> Drop for Binary<'ctx> {
    fn drop(&mut self) {
        use llvm_sys::object::LLVMDisposeBinary;

        unsafe {
            LLVMDisposeBinary(self.binary)
        }
    }
}
