//! A `Comdat` helps resolve linker errors for duplicate sections.
// https://llvm.org/doxygen/IR_2Comdat_8h_source.html
// https://stackoverflow.com/questions/1834597/what-is-the-comdat-section-used-for

use llvm_sys::comdat::{LLVMComdatSelectionKind, LLVMGetComdatSelectionKind, LLVMSetComdatSelectionKind};
use llvm_sys::prelude::LLVMComdatRef;

#[llvm_enum(LLVMComdatSelectionKind)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Determines how linker conflicts are to be resolved.
pub enum ComdatSelectionKind {
    /// The linker may choose any COMDAT.
    #[llvm_variant(LLVMAnyComdatSelectionKind)]
    Any,
    /// The data referenced by the COMDAT must be the same.
    #[llvm_variant(LLVMExactMatchComdatSelectionKind)]
    ExactMatch,
    /// The linker will choose the largest COMDAT.
    #[llvm_variant(LLVMLargestComdatSelectionKind)]
    Largest,
    /// No other Module may specify this COMDAT.
    #[llvm_variant(LLVMNoDuplicatesComdatSelectionKind)]
    NoDuplicates,
    /// The data referenced by the COMDAT must be the same size.
    #[llvm_variant(LLVMSameSizeComdatSelectionKind)]
    SameSize,
}

/// A `Comdat` determines how to resolve duplicate sections when linking.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Comdat(pub(crate) LLVMComdatRef);

impl Comdat {
    /// Creates a new `Comdat` type from a raw pointer.
    pub unsafe fn new(comdat: LLVMComdatRef) -> Self {
        debug_assert!(!comdat.is_null());

        Comdat(comdat)
    }

    /// Acquires the underlying raw pointer belonging to this `Comdat` type.
    pub fn as_mut_ptr(&self) -> LLVMComdatRef {
        self.0
    }

    /// Gets what kind of `Comdat` this is.
    pub fn get_selection_kind(self) -> ComdatSelectionKind {
        let kind_ptr = unsafe { LLVMGetComdatSelectionKind(self.0) };

        ComdatSelectionKind::new(kind_ptr)
    }

    /// Sets what kind of `Comdat` this should be.
    pub fn set_selection_kind(self, kind: ComdatSelectionKind) {
        unsafe { LLVMSetComdatSelectionKind(self.0, kind.into()) }
    }
}
