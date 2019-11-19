use std::marker::PhantomData;
use std::ffi::CStr;
use std::iter::FusedIterator;

use llvm_sys::object::LLVMSectionIteratorRef;

use super::{Binary, BinaryRef, RelocationIterator};

#[derive(Debug)]
pub struct Section<'a> {
    section: LLVMSectionIteratorRef,
    _marker: PhantomData<&'a Binary<'a>>,
}
impl<'a> Section<'a> {
    pub(super) fn new(section: LLVMSectionIteratorRef) -> Self {
        assert!(!section.is_null());

        Self {
            section,
            _marker: PhantomData,
        }
    }

    pub fn name(&self) -> &CStr {
        use llvm_sys::object::LLVMGetSectionName;

        unsafe {
            CStr::from_ptr(LLVMGetSectionName(self.section))
        }
    }

    pub fn address(&self) -> u64 {
        use llvm_sys::object::LLVMGetSectionAddress;

        unsafe {
            LLVMGetSectionAddress(self.section)
        }
    }

    pub fn size(&self) -> u64 {
        use llvm_sys::object::LLVMGetSectionSize;

        unsafe {
            LLVMGetSectionSize(self.section)
        }
    }

    pub fn contents(&self) -> &CStr {
        use llvm_sys::object::LLVMGetSectionContents;

        unsafe {
            CStr::from_ptr(LLVMGetSectionContents(self.section))
        }
    }

    pub fn relocations(&self) -> RelocationIterator<'a> {
        use llvm_sys::object::LLVMGetRelocations;

        let iter = unsafe {
            LLVMGetRelocations(self.section)
        };

        RelocationIterator::new(iter, self.section)
    }
}


#[derive(Debug)]
pub struct SectionIterator<'a> {
    binary: BinaryRef,
    iter: LLVMSectionIteratorRef,
    done: bool,
    _marker: PhantomData<&'a Binary<'a>>,
}
impl<'a> SectionIterator<'a> {
    #[llvm_versions(9.0..=latest)]
    pub(super) fn new(binary: BinaryRef) -> Self {
        use llvm_sys::object::LLVMObjectFileCopySectionIterator;

        assert!(!binary.is_null());

        let iter = unsafe { LLVMObjectFileCopySectionIterator(binary) };
        let done = iter.is_null();

        Self {
            binary,
            iter,
            done,
            _marker: PhantomData,
        }
    }

    #[llvm_versions(3.6..9.0)]
    pub(super) fn new(binary: BinaryRef) -> Self {
        use llvm_sys::object::LLVMGetSections;

        assert!(!binary.is_null());

        let iter = unsafe {
            LLVMGetSections(binary)
        };

        let done = iter.is_null();

        Self {
            binary,
            iter,
            done,
            _marker: PhantomData,
        }
    }
}
impl<'a> Iterator for SectionIterator<'a> {
    type Item = Section<'a>;

    #[llvm_versions(9.0..=latest)]
    fn next(&mut self) -> Option<Self::Item> {
        use llvm_sys::object::{LLVMObjectFileIsSectionIteratorAtEnd, LLVMMoveToNextSection};

        if self.done {
            return None;
        }

        let at_end = unsafe {
            LLVMObjectFileIsSectionIteratorAtEnd(self.binary, self.iter) == 1
        };

        if at_end {
            self.done = true;
            return None;
        }

        let section = Section::new(self.iter);

        unsafe {
            LLVMMoveToNextSection(self.iter);
        }

        Some(section)
    }

    #[llvm_versions(3.6..9.0)]
    fn next(&mut self) -> Option<Self::Item> {
        use llvm_sys::object::{LLVMIsSectionIteratorAtEnd, LLVMMoveToNextSection};

        if self.done {
            return None;
        }

        // REVIEW: Should it compare against 1? End checking order might also be off
        let at_end = unsafe {
            LLVMIsSectionIteratorAtEnd(self.binary, self.iter) == 1
        };

        if at_end {
            self.done = true;
            return None;
        }

        let section = Section::new(self.iter);

        unsafe {
            LLVMMoveToNextSection(self.iter)
        }

        Some(section)
    }
}
impl<'a> FusedIterator for SectionIterator<'a> {}

impl<'a> Drop for SectionIterator<'a> {
    fn drop(&mut self) {
        use llvm_sys::object::LLVMDisposeSectionIterator;

        unsafe {
            LLVMDisposeSectionIterator(self.iter);
        }
    }
}
