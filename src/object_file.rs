use llvm_sys::object::{
    LLVMBinaryCopyMemoryBuffer, LLVMBinaryGetType, LLVMBinaryRef, LLVMDisposeBinary, LLVMDisposeRelocationIterator,
    LLVMDisposeSectionIterator, LLVMDisposeSymbolIterator, LLVMGetRelocationOffset, LLVMGetRelocationSymbol,
    LLVMGetRelocationType, LLVMGetRelocationTypeName, LLVMGetRelocationValueString, LLVMGetRelocations,
    LLVMGetSectionAddress, LLVMGetSectionContainsSymbol, LLVMGetSectionContents, LLVMGetSectionName,
    LLVMGetSectionSize, LLVMGetSymbolAddress, LLVMGetSymbolName, LLVMGetSymbolSize, LLVMIsRelocationIteratorAtEnd,
    LLVMMoveToContainingSection, LLVMMoveToNextRelocation, LLVMMoveToNextSection, LLVMMoveToNextSymbol,
    LLVMObjectFileCopySectionIterator, LLVMObjectFileCopySymbolIterator, LLVMObjectFileIsSectionIteratorAtEnd,
    LLVMObjectFileIsSymbolIteratorAtEnd, LLVMRelocationIteratorRef, LLVMSectionIteratorRef, LLVMSymbolIteratorRef,
};

pub use llvm_sys::object::LLVMBinaryType;

use std::ffi::CStr;
use std::marker::PhantomData;

use crate::memory_buffer::MemoryBuffer;
use crate::support::LLVMString;

#[derive(Debug)]
pub struct BinaryFile<'a> {
    binary_file: LLVMBinaryRef,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> BinaryFile<'a> {
    pub unsafe fn new(binary_file: LLVMBinaryRef) -> Self {
        assert!(!binary_file.is_null());

        Self {
            binary_file,
            _phantom: PhantomData,
        }
    }

    pub fn as_mut_ptr(&self) -> LLVMBinaryRef {
        self.binary_file
    }

    pub fn get_binary_type(&self) -> LLVMBinaryType {
        unsafe { LLVMBinaryGetType(self.as_mut_ptr()) }
    }

    // the backing buffer must outlive 'a, hence never dangling
    pub fn get_memory_buffer(&self) -> MemoryBuffer<'a> {
        unsafe { MemoryBuffer::new(LLVMBinaryCopyMemoryBuffer(self.binary_file)) }
    }

    pub fn get_sections(&self) -> Option<Sections<'_>> {
        let section_iterator = unsafe { LLVMObjectFileCopySectionIterator(self.binary_file) };

        if section_iterator.is_null() {
            return None;
        }

        Some(unsafe { Sections::new(section_iterator, self.binary_file) })
    }

    pub fn get_symbols(&self) -> Option<Symbols<'_>> {
        let symbol_iterator = unsafe { LLVMObjectFileCopySymbolIterator(self.binary_file) };

        if symbol_iterator.is_null() {
            return None;
        }

        Some(unsafe { Symbols::new(symbol_iterator, self.binary_file) })
    }
}

impl<'a> Drop for BinaryFile<'a> {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBinary(self.binary_file);
        }
    }
}

#[derive(Debug)]
pub struct Sections<'a> {
    section_iterator: LLVMSectionIteratorRef,
    binary_file: LLVMBinaryRef,
    at_start: bool,
    at_end: bool,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Sections<'a> {
    pub unsafe fn new(section_iterator: LLVMSectionIteratorRef, binary_file: LLVMBinaryRef) -> Self {
        assert!(!section_iterator.is_null());
        assert!(!binary_file.is_null());

        Sections {
            section_iterator,
            binary_file,
            at_start: true,
            at_end: false,
            _phantom: PhantomData,
        }
    }

    pub fn as_mut_ptr(&self) -> (LLVMSectionIteratorRef, LLVMBinaryRef) {
        (self.section_iterator, self.binary_file)
    }

    // Here we cannot use the `Iterator`` trait since `Section` depends on the lifetime of self to
    // ensure the section cannot be used after another call to `next_section`. If it can be used
    // after another call, the underlying iterator would have moved to the next section already, and
    // thus function calls to the old section would return results of the new section.
    //
    // This is similar to the `LendingIterator` trait.
    pub fn next_section(&mut self) -> Option<Section<'_>> {
        if self.at_end {
            return None;
        }

        if !self.at_start {
            unsafe {
                LLVMMoveToNextSection(self.section_iterator);
            }
        }
        self.at_start = false;

        self.at_end = unsafe { LLVMObjectFileIsSectionIteratorAtEnd(self.binary_file, self.section_iterator) == 1 };
        if self.at_end {
            return None;
        }

        let section = unsafe { Section::new(self.section_iterator, self.binary_file) };
        Some(section)
    }

    // call `next_section` to get the containing section.
    pub fn move_to_containing_section(&mut self, symbol: &Symbol<'_>) {
        self.at_start = true;
        self.at_end = false;
        unsafe {
            LLVMMoveToContainingSection(self.section_iterator, symbol.symbol);
        }
    }
}

impl<'a> Drop for Sections<'a> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeSectionIterator(self.section_iterator) }
    }
}

#[derive(Debug)]
pub struct Section<'a> {
    section: LLVMSectionIteratorRef,
    binary_file: LLVMBinaryRef,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Section<'a> {
    pub unsafe fn new(section: LLVMSectionIteratorRef, binary_file: LLVMBinaryRef) -> Self {
        assert!(!section.is_null());
        assert!(!binary_file.is_null());

        Self {
            section,
            binary_file,
            _phantom: PhantomData,
        }
    }

    pub unsafe fn as_mut_ptr(&self) -> (LLVMSectionIteratorRef, LLVMBinaryRef) {
        (self.section, self.binary_file)
    }

    pub fn get_name(&self) -> Option<&CStr> {
        let name = unsafe { LLVMGetSectionName(self.section) };
        if !name.is_null() {
            Some(unsafe { CStr::from_ptr(name) })
        } else {
            None
        }
    }

    pub fn get_size(&self) -> u64 {
        unsafe { LLVMGetSectionSize(self.section) }
    }

    pub fn get_contents(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                LLVMGetSectionContents(self.section) as *const u8,
                self.get_size() as usize,
            )
        }
    }

    pub fn get_address(&self) -> u64 {
        unsafe { LLVMGetSectionAddress(self.section) }
    }

    pub fn contains_symbol(&self, symbol: &Symbol<'_>) -> bool {
        unsafe { LLVMGetSectionContainsSymbol(self.section, symbol.symbol) == 1 }
    }

    pub fn get_relocations(&self) -> Relocations<'_> {
        let relocation_iterator = unsafe { LLVMGetRelocations(self.section) };

        unsafe { Relocations::new(relocation_iterator, self.section, self.binary_file) }
    }
}

#[derive(Debug)]
pub struct Relocations<'a> {
    relocation_iterator: LLVMRelocationIteratorRef,
    section_iterator: LLVMSectionIteratorRef,
    binary_file: LLVMBinaryRef,
    at_start: bool,
    at_end: bool,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Relocations<'a> {
    pub unsafe fn new(
        relocation_iterator: LLVMRelocationIteratorRef,
        section_iterator: LLVMSectionIteratorRef,
        binary_file: LLVMBinaryRef,
    ) -> Self {
        assert!(!relocation_iterator.is_null());
        assert!(!section_iterator.is_null());
        assert!(!binary_file.is_null());

        Self {
            relocation_iterator,
            section_iterator,
            binary_file,
            at_start: true,
            at_end: false,
            _phantom: PhantomData,
        }
    }

    pub fn as_mut_ptr(&self) -> (LLVMRelocationIteratorRef, LLVMSectionIteratorRef, LLVMBinaryRef) {
        (self.relocation_iterator, self.section_iterator, self.binary_file)
    }

    pub fn next_relocation(&mut self) -> Option<Relocation<'_>> {
        if self.at_end {
            return None;
        }

        if !self.at_start {
            unsafe {
                LLVMMoveToNextRelocation(self.relocation_iterator);
            }
        }
        self.at_start = false;

        self.at_end = unsafe { LLVMIsRelocationIteratorAtEnd(self.section_iterator, self.relocation_iterator) == 1 };
        if self.at_end {
            return None;
        }

        let relocation = unsafe { Relocation::new(self.relocation_iterator, self.binary_file) };
        Some(relocation)
    }
}

impl<'a> Drop for Relocations<'a> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeRelocationIterator(self.relocation_iterator) }
    }
}

#[derive(Debug)]
pub struct Relocation<'a> {
    relocation: LLVMRelocationIteratorRef,
    binary_file: LLVMBinaryRef,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Relocation<'a> {
    pub unsafe fn new(relocation: LLVMRelocationIteratorRef, binary_file: LLVMBinaryRef) -> Self {
        assert!(!relocation.is_null());
        assert!(!binary_file.is_null());

        Self {
            relocation,
            binary_file,
            _phantom: PhantomData,
        }
    }

    pub fn as_mut_ptr(&self) -> (LLVMRelocationIteratorRef, LLVMBinaryRef) {
        (self.relocation, self.binary_file)
    }

    pub fn get_offset(&self) -> u64 {
        unsafe { LLVMGetRelocationOffset(self.relocation) }
    }

    pub fn get_type(&self) -> (u64, LLVMString) {
        let type_int = unsafe { LLVMGetRelocationType(self.relocation) };
        let type_name = unsafe { LLVMString::new(LLVMGetRelocationTypeName(self.relocation)) };

        (type_int, type_name)
    }

    pub fn get_value(&self) -> LLVMString {
        unsafe { LLVMString::new(LLVMGetRelocationValueString(self.relocation)) }
    }

    pub fn get_symbol(&self) -> Symbol<'_> {
        let symbol = unsafe { LLVMGetRelocationSymbol(self.relocation) };

        unsafe { Symbol::new(symbol) }
    }
}

#[derive(Debug)]
pub struct Symbols<'a> {
    symbol_iterator: LLVMSymbolIteratorRef,
    binary_file: LLVMBinaryRef,
    at_start: bool,
    at_end: bool,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Symbols<'a> {
    pub unsafe fn new(symbol_iterator: LLVMSymbolIteratorRef, binary_file: LLVMBinaryRef) -> Self {
        assert!(!symbol_iterator.is_null());
        assert!(!binary_file.is_null());

        Self {
            symbol_iterator,
            binary_file,
            at_start: true,
            at_end: false,
            _phantom: PhantomData,
        }
    }

    pub fn as_mut_ptr(&self) -> (LLVMSymbolIteratorRef, LLVMBinaryRef) {
        (self.symbol_iterator, self.binary_file)
    }

    pub fn next_symbol(&mut self) -> Option<Symbol<'_>> {
        if self.at_end {
            return None;
        }

        if !self.at_start {
            unsafe {
                LLVMMoveToNextSymbol(self.symbol_iterator);
            }
        }
        self.at_start = false;

        self.at_end = unsafe { LLVMObjectFileIsSymbolIteratorAtEnd(self.binary_file, self.symbol_iterator) == 1 };
        if self.at_end {
            return None;
        }

        let symbol = unsafe { Symbol::new(self.symbol_iterator) };
        Some(symbol)
    }
}

impl<'a> Drop for Symbols<'a> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeSymbolIterator(self.symbol_iterator) }
    }
}

#[derive(Debug)]
pub struct Symbol<'a> {
    symbol: LLVMSymbolIteratorRef,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Symbol<'a> {
    pub unsafe fn new(symbol: LLVMSymbolIteratorRef) -> Self {
        assert!(!symbol.is_null());

        Self {
            symbol,
            _phantom: PhantomData,
        }
    }

    pub fn as_mut_ptr(&self) -> LLVMSymbolIteratorRef {
        self.symbol
    }

    pub fn get_name(&self) -> Option<&CStr> {
        let name = unsafe { LLVMGetSymbolName(self.symbol) };
        if !name.is_null() {
            Some(unsafe { CStr::from_ptr(name) })
        } else {
            None
        }
    }

    pub fn get_size(&self) -> u64 {
        unsafe { LLVMGetSymbolSize(self.symbol) }
    }

    pub fn get_address(&self) -> u64 {
        unsafe { LLVMGetSymbolAddress(self.symbol) }
    }
}
