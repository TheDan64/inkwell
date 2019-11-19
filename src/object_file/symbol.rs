use std::marker::PhantomData;
use std::ffi::CStr;
use std::iter::FusedIterator;

use llvm_sys::object::LLVMSymbolIteratorRef;

use super::{Binary, BinaryRef, Relocation};

#[derive(Debug)]
pub struct Symbol<'a> {
    symbol: LLVMSymbolIteratorRef,
    _marker: PhantomData<&'a Relocation<'a>>,
}
impl<'a> Symbol<'a> {
    pub(super) fn new(symbol: LLVMSymbolIteratorRef) -> Self {
        Self {
            symbol,
            _marker: PhantomData,
        }
    }

    pub fn name(&self) -> &CStr {
        use llvm_sys::object::LLVMGetSymbolName;

        unsafe {
            CStr::from_ptr(LLVMGetSymbolName(self.symbol))
        }
    }

    pub fn size(&self) -> u64 {
        use llvm_sys::object::LLVMGetSymbolSize;

        unsafe {
            LLVMGetSymbolSize(self.symbol)
        }
    }

    pub fn address(&self) -> u64 {
        use llvm_sys::object::LLVMGetSymbolAddress;

        unsafe {
            LLVMGetSymbolAddress(self.symbol)
        }
    }
}

#[derive(Debug)]
pub struct SymbolIterator<'a> {
    iter: LLVMSymbolIteratorRef,
    binary: BinaryRef,
    done: bool,
    _marker: PhantomData<&'a Binary<'a>>,
}
impl<'a> SymbolIterator<'a> {
    #[llvm_versions(9.0..=latest)]
    pub(super) fn new(binary: BinaryRef) -> Self {
        use llvm_sys::object::LLVMObjectFileCopySymbolIterator;

        assert!(!binary.is_null());

        let iter = unsafe {
            LLVMObjectFileCopySymbolIterator(binary)
        };

        let done = iter.is_null();

        Self {
            iter,
            binary,
            done,
            _marker: PhantomData,
        }
    }

    #[llvm_versions(3.6..9.0)]
    pub(super) fn new(binary: BinaryRef) -> Self {
        use llvm_sys::object::LLVMGetSymbols;

        assert!(!binary.is_null());

        let iter = unsafe {
            LLVMGetSymbols(binary)
        };
        let done = iter.is_null();

        Self {
            iter,
            binary,
            done,
            _marker: PhantomData,
        }
    }
}
impl<'a> Iterator for SymbolIterator<'a> {
    type Item = Symbol<'a>;

    #[llvm_versions(9.0..=latest)]
    fn next(&mut self) -> Option<Self::Item> {
        use llvm_sys::object::{LLVMObjectFileIsSymbolIteratorAtEnd, LLVMMoveToNextSymbol};

        if self.done {
            return None;
        }

        let at_end = unsafe {
            LLVMObjectFileIsSymbolIteratorAtEnd(self.binary, self.iter) == 1
        };

        if at_end {
            self.done = true;
            return None;
        }

        let symbol = Symbol::new(self.iter);

        unsafe {
            LLVMMoveToNextSymbol(self.iter);
        }

        Some(symbol)
    }

    #[llvm_versions(3.6..9.0)]
    fn next(&mut self) -> Option<Self::Item> {
        use llvm_sys::object::{LLVMIsSymbolIteratorAtEnd, LLVMMoveToNextSymbol};

        if self.done {
            return None;
        }

        let at_end = unsafe {
            LLVMIsSymbolIteratorAtEnd(self.binary, self.iter) == 1
        };

        if at_end {
            self.done = true;
            return None;
        }

        let symbol = Symbol::new(self.iter);

        unsafe {
            LLVMMoveToNextSymbol(self.iter);
        }

        Some(symbol)
    }
}
impl<'a> FusedIterator for SymbolIterator<'a> {}
impl<'a> Drop for SymbolIterator<'a> {
    fn drop(&mut self) {
        use llvm_sys::object::LLVMDisposeSymbolIterator;

        if self.iter.is_null() {
            return;
        }

        unsafe {
            LLVMDisposeSymbolIterator(self.iter);
        }
    }
}
