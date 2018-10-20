//! A `Comdat` helps resolve linker errors for duplicate sections.
// https://llvm.org/doxygen/IR_2Comdat_8h_source.html
// https://stackoverflow.com/questions/1834597/what-is-the-comdat-section-used-for

use llvm_sys::comdat::{LLVMComdatSelectionKind, LLVMSetComdatSelectionKind, LLVMGetComdatSelectionKind};
use llvm_sys::prelude::LLVMComdatRef;

enum_rename!{
    /// Determines how linker conflicts are to be resolved.
    ComdatSelectionKind <=> LLVMComdatSelectionKind {
        /// The linker may choose any COMDAT.
        Any <=> LLVMAnyComdatSelectionKind,
        /// The data referenced by the COMDAT must be the same.
        ExactMatch <=> LLVMExactMatchComdatSelectionKind,
        /// The linker will choose the largest COMDAT.
        Largest <=> LLVMLargestComdatSelectionKind,
        /// No other Module may specify this COMDAT.
        NoDuplicates <=> LLVMNoDuplicatesComdatSelectionKind,
        /// The data referenced by the COMDAT must be the same size.
        SameSize <=> LLVMSameSizeComdatSelectionKind,
    }
}

/// A `Comdat` determines how to resolve duplicate sections when linking.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Comdat(pub(crate) LLVMComdatRef);

impl Comdat {
    pub(crate) fn new(comdat: LLVMComdatRef) -> Self {
        debug_assert!(!comdat.is_null());

        Comdat(comdat)
    }

    /// Gets what kind of `Comdat` this is.
    pub fn get_selection_kind(&self) -> ComdatSelectionKind {
        let kind_ptr = unsafe {
            LLVMGetComdatSelectionKind(self.0)
        };

        ComdatSelectionKind::new(kind_ptr)
    }

    /// Sets what kind of `Comdat` this should be.
    pub fn set_selection_kind(&self, kind: ComdatSelectionKind) {
        unsafe {
            LLVMSetComdatSelectionKind(self.0, kind.as_llvm_enum())
        }
    }
}
