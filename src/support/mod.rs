#[deny(missing_docs)]
pub mod error_handling;

use libc::c_char;
use llvm_sys::core::{LLVMCreateMessage, LLVMDisposeMessage};
use llvm_sys::support::LLVMLoadLibraryPermanently;

use std::borrow::Cow;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr;

/// An owned LLVM String. Also known as a LLVM Message
#[derive(Eq)]
pub struct LLVMString {
    pub(crate) ptr: *const c_char,
}

impl LLVMString {
    pub(crate) unsafe fn new(ptr: *const c_char) -> Self {
        LLVMString {
            ptr,
        }
    }

    /// This is a convenience method for creating a Rust `String`,
    /// however; it *will* reallocate. `LLVMString` should be used
    /// as much as possible to save memory since it is allocated by
    /// LLVM. It's essentially a `CString` with a custom LLVM
    /// deallocator
    pub fn to_string(&self) -> String {
        (*self).to_string_lossy().into_owned()
    }

    /// This method will allocate a c string through LLVM
    pub(crate) fn create_from_c_str(string: &CStr) -> LLVMString {
        unsafe {
            LLVMString::new(LLVMCreateMessage(string.as_ptr() as *const _))
        }
    }

    /// This method will allocate a c string through LLVM
    pub(crate) fn create_from_str(string: &str) -> LLVMString {
        debug_assert_eq!(string.as_bytes()[string.as_bytes().len() - 1], 0);

        unsafe {
            LLVMString::new(LLVMCreateMessage(string.as_ptr() as *const _))
        }
    }
}

impl Deref for LLVMString {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        unsafe {
            CStr::from_ptr(self.ptr)
        }
    }
}

impl Debug for LLVMString {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.deref())
    }
}

impl Display for LLVMString {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.deref().to_string_lossy())
    }
}

impl PartialEq for LLVMString {
    fn eq(&self, other: &LLVMString) -> bool {
        **self == **other
    }
}

impl Error for LLVMString {
    fn description(&self) -> &str {
        self.to_str().expect("Could not convert LLVMString to str (likely invalid unicode)")
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl Drop for LLVMString {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMessage(self.ptr as *mut _);
        }
    }
}

impl OwnedPtr for LLVMString {
    type Ptr = *const c_char;

    #[inline]
    fn as_ptr(&self) -> Self::Ptr {
        self.ptr
    }

    unsafe fn transfer_ownership_to_llvm(mut self) {
        self.ptr = ptr::null_mut();
    }
}

// Similar to Cow; however does not provide ability to clone
// since memory is allocated by LLVM. Could use a better name
// too. This is meant to be an internal wrapper only. Maybe
// belongs in a private utils module.
pub(crate) struct LLVMStringOrRaw<'a> {
    string: OwnedOrBorrowedPtr<'a, LLVMString>,
}

impl<'a> LLVMStringOrRaw<'a> {
    pub(crate) fn owned(string: LLVMString) -> LLVMStringOrRaw<'static> {
        LLVMStringOrRaw {
            string: OwnedOrBorrowedPtr::Owned(string),
        }
    }

    pub(crate) unsafe fn borrowed(string: *const c_char) -> Self {
        LLVMStringOrRaw {
            string: OwnedOrBorrowedPtr::borrowed(string),
        }
    }

    pub fn as_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.string.as_ptr()) }
    }

    pub(crate) fn as_ptr(&self) -> *const c_char {
        return self.string.as_ptr();
    }
}

impl<'a> PartialEq for LLVMStringOrRaw<'a> {
    fn eq(&self, other: &LLVMStringOrRaw<'a>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for LLVMStringOrRaw<'_> {}

impl Debug for LLVMStringOrRaw<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.string {
            OwnedOrBorrowedPtr::Owned(..) => f.debug_tuple("Owned").field(&self.as_str()).finish(),
            OwnedOrBorrowedPtr::Borrowed(..) => {
                f.debug_tuple("Borrowed").field(&self.as_str()).finish()
            }
        }
    }
}

impl Display for LLVMStringOrRaw<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str().to_string_lossy())
    }
}

pub(crate) trait OwnedPtr {
    type Ptr;

    fn as_ptr(&self) -> Self::Ptr;
    unsafe fn transfer_ownership_to_llvm(self);
}

macro_rules! impl_owned_ptr {
    ($vis:vis $name:ident, $ty:ty, $destroy_fn:ident) => {
        #[derive(Debug, PartialEq)]
        $vis struct $name($ty);

        impl $crate::support::OwnedPtr for $name {
            type Ptr = $ty;

            #[inline]
            fn as_ptr(&self) -> Self::Ptr {
                self.0
            }

            unsafe fn transfer_ownership_to_llvm(mut self) {
                self.0 = std::ptr::null_mut();
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                if !self.0.is_null() {
                    unsafe {
                        $destroy_fn(self.0 as *mut _);
                    }
                }
            }
        }
    };
}

pub(crate) enum OwnedOrBorrowedPtr<'a, T>
where
    T: OwnedPtr,
    <T as OwnedPtr>::Ptr: Copy,
{
    Owned(T),
    Borrowed(<T as OwnedPtr>::Ptr, PhantomData<&'a ()>),
}

impl<T> OwnedOrBorrowedPtr<'_, T>
where
    T: OwnedPtr,
    <T as OwnedPtr>::Ptr: Copy,
{
    pub fn borrowed(ptr: <T as OwnedPtr>::Ptr) -> Self {
        OwnedOrBorrowedPtr::Borrowed(ptr, PhantomData)
    }

    pub(crate) unsafe fn transfer_ownership_to_llvm(self) -> <T as OwnedPtr>::Ptr {
        match self {
            OwnedOrBorrowedPtr::Owned(o) => {
                let ptr = o.as_ptr();
                o.transfer_ownership_to_llvm();
                ptr
            }
            OwnedOrBorrowedPtr::Borrowed(b, _) => b,
        }
    }
    pub(crate) fn as_ptr(&self) -> <T as OwnedPtr>::Ptr {
        match self {
            OwnedOrBorrowedPtr::Owned(o) => o.as_ptr(),
            OwnedOrBorrowedPtr::Borrowed(b, _) => *b,
        }
    }
}

impl<T> Debug for OwnedOrBorrowedPtr<'_, T>
where
    T: OwnedPtr + Debug,
    <T as OwnedPtr>::Ptr: Copy + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owned(o) => f.debug_tuple("Owned").field(o).finish(),
            Self::Borrowed(b, _) => f.debug_tuple("Borrowed").field(b).finish(),
        }
    }
}

impl<'a, T> PartialEq for OwnedOrBorrowedPtr<'a, T>
where
    T: OwnedPtr + PartialEq,
    <T as OwnedPtr>::Ptr: Copy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

/// This function is very unsafe. Any reference to LLVM data after this function is called will likey segfault.
/// Probably only ever useful to call before your program ends. Might not even be absolutely necessary.
pub unsafe fn shutdown_llvm() {
    use llvm_sys::core::LLVMShutdown;

    LLVMShutdown()
}

pub fn load_library_permanently(filename: &str) -> bool {
    let filename = to_c_str(filename);

    unsafe {
        LLVMLoadLibraryPermanently(filename.as_ptr()) == 1
    }
}

/// Determines whether or not LLVM has been configured to run in multithreaded mode. (Inkwell currently does
/// not officially support multithreaded mode)
pub fn is_multithreaded() -> bool {
    use llvm_sys::core::LLVMIsMultithreaded;

    unsafe {
        LLVMIsMultithreaded() == 1
    }
}

pub fn enable_llvm_pretty_stack_trace() {
    #[llvm_versions(3.6..=3.7)]
    use llvm_sys::core::LLVMEnablePrettyStackTrace;
    #[llvm_versions(3.8..=latest)]
    use llvm_sys::error_handling::LLVMEnablePrettyStackTrace;

    unsafe {
        LLVMEnablePrettyStackTrace()
    }
}

/// This function takes in a Rust string and either:
///
/// A) Finds a terminating null byte in the Rust string and can reference it directly like a C string.
///
/// B) Finds no null byte and allocates a new C string based on the input Rust string.
pub(crate) fn to_c_str<'s>(mut s: &'s str) -> Cow<'s, CStr> {
    if s.is_empty() {
        s = "\0";
    }

    // Start from the end of the string as it's the most likely place to find a null byte
    if s.chars().rev().find(|&ch| ch == '\0').is_none() {
        return Cow::from(CString::new(s).expect("unreachable since null bytes are checked"));
    }

    unsafe {
        Cow::from(CStr::from_ptr(s.as_ptr() as *const _))
    }
}

#[test]
fn test_to_c_str() {
    // TODO: If we raise our MSRV to >= 1.42 we can use matches!() here or
    // is_owned()/is_borrowed() if it ever gets stabilized.
    if let Cow::Borrowed(_) = to_c_str("my string") {
        panic!();
    }

    if let Cow::Owned(_) = to_c_str("my string\0") {
        panic!();
    }
}
