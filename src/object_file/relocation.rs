use std::marker::PhantomData;
use std::iter::FusedIterator;

use llvm_sys::object::{LLVMRelocationIteratorRef, LLVMSectionIteratorRef};

use crate::support::LLVMString;
use super::{Section, Symbol};

#[derive(Debug)]
pub struct Relocation<'a> {
    relocation: LLVMRelocationIteratorRef,
    _marker: PhantomData<&'a Section<'a>>,
}
impl<'a> Relocation<'a> {
    pub(super) fn new(relocation: LLVMRelocationIteratorRef) -> Self {
        Self {
            relocation,
            _marker: PhantomData,
        }
    }

    pub fn offset(&self) -> u64 {
        use llvm_sys::object::LLVMGetRelocationOffset;

        unsafe {
            LLVMGetRelocationOffset(self.relocation)
        }
    }

    pub fn get_type(&self) -> (u64, LLVMString) {
        use llvm_sys::object::LLVMGetRelocationType;
        use llvm_sys::object::LLVMGetRelocationTypeName;

        let type_id = unsafe {
            LLVMGetRelocationType(self.relocation)
        };
        let type_name_ptr = unsafe {
            LLVMGetRelocationTypeName(self.relocation)
        };

        (type_id, LLVMString::new(type_name_ptr))
    }
    
    pub fn value(&self) -> LLVMString {
        use llvm_sys::object::LLVMGetRelocationValueString;

        let value_ptr = unsafe {
            LLVMGetRelocationValueString(self.relocation)
        };

        LLVMString::new(value_ptr)
    }

    pub fn symbol(&self) -> Symbol<'a> {
        use llvm_sys::object::LLVMGetRelocationSymbol;

        let symbol = unsafe {
            LLVMGetRelocationSymbol(self.relocation)
        };
        Symbol::new(symbol)
    }
}

#[derive(Debug)]
pub struct RelocationIterator<'a> {
    iter: LLVMRelocationIteratorRef,
    section: LLVMSectionIteratorRef,
    _marker: PhantomData<&'a Section<'a>>,
}
impl<'a> RelocationIterator<'a> {
    pub(super) fn new(iter: LLVMRelocationIteratorRef, section: LLVMSectionIteratorRef) -> Self {
        Self {
            iter,
            section,
            _marker: PhantomData,
        }
    }
}
impl<'a> Iterator for RelocationIterator<'a> {
    type Item = Relocation<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use llvm_sys::object::{LLVMIsRelocationIteratorAtEnd, LLVMMoveToNextRelocation};

        let at_end = unsafe {
            LLVMIsRelocationIteratorAtEnd(self.section, self.iter) == 1
        };

        if at_end {
            return None;
        }

        let relocation = Relocation::new(self.iter);

        unsafe {
            LLVMMoveToNextRelocation(self.iter);
        }

        Some(relocation)
    }
}
impl<'a> FusedIterator for RelocationIterator<'a> {}
impl<'a> Drop for RelocationIterator<'a> {
    fn drop(&mut self) {
        use llvm_sys::object::LLVMDisposeRelocationIterator;

        unsafe {
            LLVMDisposeRelocationIterator(self.iter)
        }
    }
}
