use llvm_sys::object::{LLVMDisposeObjectFile, LLVMObjectFileRef, LLVMSectionIteratorRef, LLVMGetSections, LLVMDisposeSectionIterator, LLVMSymbolIteratorRef, LLVMIsSectionIteratorAtEnd, LLVMGetSectionName, LLVMDisposeRelocationIterator, LLVMRelocationIteratorRef, LLVMDisposeSymbolIterator, LLVMGetSectionContents, LLVMGetSectionSize, LLVMMoveToNextSection, LLVMGetSectionAddress, LLVMGetSymbolName, LLVMGetSymbolSize, LLVMGetRelocations, LLVMGetSymbolAddress, LLVMGetRelocationOffset, LLVMGetRelocationSymbol, LLVMGetRelocationType, LLVMGetRelocationTypeName, LLVMGetRelocationValueString, LLVMMoveToNextSymbol, LLVMMoveToNextRelocation, LLVMIsSymbolIteratorAtEnd, LLVMIsRelocationIteratorAtEnd, LLVMGetSymbols};

use std::ffi::CStr;

// REVIEW: Make sure SectionIterator's object_file ptr doesn't outlive ObjectFile
// REVIEW: This module is very untested
// TODO: More references to account for lifetimes
#[derive(Debug)]
pub struct ObjectFile {
    object_file: LLVMObjectFileRef
}

impl ObjectFile {
    pub(crate) fn new(object_file: LLVMObjectFileRef) -> Self {
        assert!(!object_file.is_null());

        ObjectFile {
            object_file
        }
    }

    pub fn get_sections(&self) -> SectionIterator {
        let section_iterator = unsafe {
            LLVMGetSections(self.object_file)
        };

        SectionIterator::new(section_iterator, self.object_file)
    }

    pub fn get_symbols(&self) -> SymbolIterator {
        let symbol_iterator = unsafe {
            LLVMGetSymbols(self.object_file)
        };

        SymbolIterator::new(symbol_iterator, self.object_file)
    }
}

impl Drop for ObjectFile {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeObjectFile(self.object_file)
        }
    }
}

#[derive(Debug)]
pub struct SectionIterator {
    section_iterator: LLVMSectionIteratorRef,
    object_file: LLVMObjectFileRef,
    before_first: bool,
}

impl SectionIterator {
    fn new(section_iterator: LLVMSectionIteratorRef, object_file: LLVMObjectFileRef) -> Self {
        assert!(!section_iterator.is_null());

        SectionIterator {
            section_iterator,
            object_file,
            before_first: true
        }
    }
}

impl Iterator for SectionIterator {
    type Item = Section;

    fn next(&mut self) -> Option<Self::Item> {
        if self.before_first {
            self.before_first = false;
        } else {
            unsafe {
                LLVMMoveToNextSection(self.section_iterator);
            }
        }

        let at_end = unsafe {
            LLVMIsSectionIteratorAtEnd(self.object_file, self.section_iterator) == 1
        };

        if at_end {
            return None;
        }

        let section = Section::new(self.section_iterator, self.object_file);

        Some(section)
    }
}

impl Drop for SectionIterator {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeSectionIterator(self.section_iterator)
        }
    }
}

#[derive(Debug)]
pub struct Section {
    section: LLVMSectionIteratorRef,
    object_file: LLVMObjectFileRef,
}

impl Section {
    fn new(section: LLVMSectionIteratorRef, object_file: LLVMObjectFileRef) -> Self {
        assert!(!section.is_null());

        Section {
            section,
            object_file,
        }
    }

    pub fn get_name(&self) -> Option<&CStr> {
        let name = unsafe {
            LLVMGetSectionName(self.section)
        };
        if !name.is_null() {
            Some(unsafe { CStr::from_ptr(name) })
        } else {
            None
        }
    }

    pub fn size(&self) -> u64 {
        unsafe {
            LLVMGetSectionSize(self.section)
        }
    }

    pub fn get_contents(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                LLVMGetSectionContents(self.section) as *const u8,
                self.size() as usize)
        }
    }

    pub fn get_address(&self) -> u64 {
        unsafe {
            LLVMGetSectionAddress(self.section)
        }
    }

    pub fn get_relocations(&self) -> RelocationIterator {
        let relocation_iterator = unsafe {
            LLVMGetRelocations(self.section)
        };

        RelocationIterator::new(relocation_iterator, self.section, self.object_file)
    }
}

#[derive(Debug)]
pub struct RelocationIterator {
    relocation_iterator: LLVMRelocationIteratorRef,
    section_iterator: LLVMSectionIteratorRef,
    object_file: LLVMObjectFileRef,
    before_first: bool,
}

impl RelocationIterator {
    fn new(relocation_iterator: LLVMRelocationIteratorRef, section_iterator: LLVMSectionIteratorRef, object_file: LLVMObjectFileRef) -> Self {
        assert!(!relocation_iterator.is_null());

        RelocationIterator {
            relocation_iterator,
            section_iterator,
            object_file,
            before_first: true
        }
    }
}

impl Iterator for RelocationIterator {
    type Item = Relocation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.before_first {
            self.before_first = false;
        } else {
            unsafe {
                LLVMMoveToNextRelocation(self.relocation_iterator)
            }
        }

        let at_end = unsafe {
            LLVMIsRelocationIteratorAtEnd(self.section_iterator, self.relocation_iterator) == 1
        };

        if at_end {
            return None;
        }

        let relocation = Relocation::new(self.relocation_iterator, self.object_file);

        Some(relocation)
    }
}

impl Drop for RelocationIterator {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeRelocationIterator(self.relocation_iterator)
        }
    }
}

#[derive(Debug)]
pub struct Relocation {
    relocation: LLVMRelocationIteratorRef,
    object_file: LLVMObjectFileRef,
}

impl Relocation {
    fn new(relocation: LLVMRelocationIteratorRef, object_file: LLVMObjectFileRef) -> Self {
        assert!(!relocation.is_null());

        Relocation {
            relocation,
            object_file,
        }
    }

    pub fn get_offset(&self) -> u64 {
        unsafe {
            LLVMGetRelocationOffset(self.relocation)
        }
    }

    pub fn get_symbols(&self) -> SymbolIterator {
        let symbol_iterator = unsafe {
            // REVIEW: Is this just returning a single Symbol (given the name) and not a full iterator?
            LLVMGetRelocationSymbol(self.relocation)
        };

        SymbolIterator::new(symbol_iterator, self.object_file)
    }

    pub fn get_type(&self) -> (u64, &CStr) {
        let type_int = unsafe {
            LLVMGetRelocationType(self.relocation)
        };
        let type_name = unsafe {
            CStr::from_ptr(LLVMGetRelocationTypeName(self.relocation))
        };

        (type_int, type_name)
    }

    pub fn get_value(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetRelocationValueString(self.relocation))
        }
    }
}

#[derive(Debug)]
pub struct SymbolIterator {
    symbol_iterator: LLVMSymbolIteratorRef,
    object_file: LLVMObjectFileRef,
    before_first: bool,
}

impl SymbolIterator {
    fn new(symbol_iterator: LLVMSymbolIteratorRef, object_file: LLVMObjectFileRef) -> Self {
        assert!(!symbol_iterator.is_null());

        SymbolIterator {
            symbol_iterator,
            object_file,
            before_first: true
        }
    }
}

impl Iterator for SymbolIterator {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        if self.before_first {
            self.before_first = false;
        } else {
            unsafe {
                LLVMMoveToNextSymbol(self.symbol_iterator)
            }
        }

        let at_end = unsafe {
            LLVMIsSymbolIteratorAtEnd(self.object_file, self.symbol_iterator) == 1
        };

        if at_end {
            return None;
        }

        let symbol = Symbol::new(self.symbol_iterator);

        Some(symbol)
    }
}

impl Drop for SymbolIterator {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeSymbolIterator(self.symbol_iterator)
        }
    }
}

#[derive(Debug)]
pub struct Symbol {
    symbol: LLVMSymbolIteratorRef,
}

impl Symbol {
    fn new(symbol: LLVMSymbolIteratorRef) -> Self {
        assert!(!symbol.is_null());

        Symbol {
            symbol,
        }
    }

    pub fn get_name(&self) -> Option<&CStr> {
        let name = unsafe {
            LLVMGetSymbolName(self.symbol)
        };
        if !name.is_null() {
            Some(unsafe { CStr::from_ptr(name) })
        } else {
            None
        }
    }

    pub fn size(&self) -> u64 {
        unsafe {
            LLVMGetSymbolSize(self.symbol)
        }
    }

    pub fn get_address(&self) -> u64 {
        unsafe {
            LLVMGetSymbolAddress(self.symbol)
        }
    }
}
