#[deny(missing_docs)]
pub mod error_handling;

use libc::c_char;
use llvm_sys::core::{LLVMCreateMessage, LLVMDisposeMessage};
use llvm_sys::support::LLVMLoadLibraryPermanently;

use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::ffi::{CString, CStr};
use std::ops::Deref;

/// An owned LLVM String. Also known as a LLVM Message
#[derive(Eq)]
pub struct LLVMString {
    pub(crate) ptr: *const c_char,
}

impl LLVMString {
    pub(crate) fn new(ptr: *const c_char) -> Self {
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

    /// Don't use this if it's not necessary. You likely need to allocate
    /// a CString as input and then LLVM will likely allocate their own string
    /// anyway.
    pub(crate) fn create(bytes: *const c_char) -> LLVMString {
        let ptr = unsafe {
            LLVMCreateMessage(bytes)
        };

        LLVMString::new(ptr)
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
        write!(f, "{:?}", self.deref())
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

    fn cause(&self) -> Option<&Error> {
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

// Similar to Cow; however does not provide ability to clone
// since memory is allocated by LLVM. Could use a better name
// too. This is meant to be an internal wrapper only. Maybe
// belongs in a private utils module.
#[derive(Eq)]
pub(crate) enum LLVMStringOrRaw {
    Owned(LLVMString),
    Borrowed(*const c_char),
}

impl LLVMStringOrRaw {
    pub fn as_str(&self) -> &CStr {
        match self {
            LLVMStringOrRaw::Owned(llvm_string) => llvm_string.deref(),
            LLVMStringOrRaw::Borrowed(ptr) => unsafe {
                CStr::from_ptr(*ptr)
            },
        }
    }
}

impl PartialEq for LLVMStringOrRaw {
    fn eq(&self, other: &LLVMStringOrRaw) -> bool {
        self.as_str() == other.as_str()
    }
}

/// This function is very unsafe. Any reference to LLVM data after this function is called will likey segfault.
/// Probably only ever useful to call before your program ends. Might not even be absolutely necessary.
pub unsafe fn shutdown_llvm() {
    use llvm_sys::core::LLVMShutdown;

    LLVMShutdown()
}

pub fn load_library_permanently(filename: &str) -> bool {
    let filename = CString::new(filename).expect("Conversion to CString failed unexpectedly");

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
    #[llvm_versions(3.6 => 3.7)]
    use llvm_sys::core::LLVMEnablePrettyStackTrace;
    #[llvm_versions(3.8 => latest)]
    use llvm_sys::error_handling::LLVMEnablePrettyStackTrace;

    unsafe {
        LLVMEnablePrettyStackTrace()
    }
}

macro_rules! enum_rename {
    ($(#[$enum_attrs:meta])* $enum_name:ident <=> $llvm_enum_name:ident {
        $($(#[$variant_attrs:meta])* $args:ident <=> $llvm_args:ident,)+
    }) => (
        #[derive(Debug, PartialEq, Eq, Copy, Clone)]
        $(#[$enum_attrs])*
        pub enum $enum_name {
            $(
                $(#[$variant_attrs])*
                $args,
            )*
        }

        impl $enum_name {
            #[allow(dead_code)]
            pub(crate) fn new(llvm_enum: $llvm_enum_name) -> Self {
                match llvm_enum {
                    $(
                        $llvm_enum_name::$llvm_args => $enum_name::$args,
                    )*
                }
            }

            pub(crate) fn as_llvm_enum(&self) -> $llvm_enum_name {
                match *self {
                    $(
                        $enum_name::$args => $llvm_enum_name::$llvm_args,
                    )*
                }
            }
        }
    );
}
