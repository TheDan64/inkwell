use std::marker::PhantomData;

use llvm_sys::object::LLVMObjectFileRef;

use crate::context::Context;

use super::{SectionIterator, SymbolIterator};

/// Alias for compatibility with binary
pub type BinaryRef = LLVMObjectFileRef;

// Alias for other object_file submodules
pub type Binary<'a> = ObjectFile<'a>;

// REVIEW: Make sure SectionIterator's object_file ptr doesn't outlive ObjectFile
// REVIEW: This module is very untested
// TODO: More references to account for lifetimes
#[derive(Debug)]
pub struct ObjectFile<'ctx> {
    object_file: LLVMObjectFileRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> ObjectFile<'ctx> {
    pub(crate) fn new(object_file: LLVMObjectFileRef) -> Self {
        assert!(!object_file.is_null());

        Self {
            object_file,
            _marker: PhantomData,
        }
    }

    pub fn sections<'a>(&'a self) -> SectionIterator<'a> {
        SectionIterator::new(self.object_file)
    }

    pub fn symbols<'a>(&'a self) -> SymbolIterator<'a> {
        SymbolIterator::new(self.object_file)
    }
}

impl<'ctx> Drop for ObjectFile<'ctx> {
    fn drop(&mut self) {
        use llvm_sys::object::LLVMDisposeObjectFile;

        unsafe {
            LLVMDisposeObjectFile(self.object_file)
        }
    }
}
