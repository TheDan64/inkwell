//! Inkwell documentation is a work in progress.
//!
//! If you have any LLVM knowledge that could be used to improve these docs, we would greatly appreciate you opening an issue and/or a pull request on our [GitHub page](https://github.com/TheDan64/inkwell).
//!
//! Due to a rustdoc issue, this documentation represents only the latest supported LLVM version. We hope that this issue will be resolved in the future so that multiple versions can be documented side by side.

#![deny(missing_debug_implementations)]
extern crate either;
#[macro_use]
extern crate enum_methods;
extern crate libc;
extern crate llvm_sys;

#[deny(missing_docs)]
pub mod basic_block;
pub mod builder;
pub mod context;
pub mod data_layout;
pub mod execution_engine;
pub mod memory_buffer;
pub mod module;
pub mod object_file;
pub mod passes;
pub mod targets;
pub mod types;
pub mod values;

use llvm_sys::{LLVMIntPredicate, LLVMRealPredicate, LLVMVisibility, LLVMThreadLocalMode, LLVMDLLStorageClass};
use llvm_sys::core::LLVMDisposeMessage;
use llvm_sys::support::LLVMLoadLibraryPermanently;

use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::ffi::{CString, CStr};
use std::ops::Deref;

#[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9", feature = "llvm4-0",
              feature = "llvm5-0")))]
compile_error!("A LLVM feature flag must be provided. See the README for more details.");

// TODO: Probably move into error handling module
pub fn enable_llvm_pretty_stack_trace() {
    #[cfg(any(feature = "llvm3-6", feature = "llvm3-7"))]
    use llvm_sys::core::LLVMEnablePrettyStackTrace;
    #[cfg(any(feature = "llvm3-8", feature = "llvm3-9", feature = "llvm4-0", feature = "llvm5-0"))]
    use llvm_sys::error_handling::LLVMEnablePrettyStackTrace;

    unsafe {
        LLVMEnablePrettyStackTrace()
    }
}

// TODO: Move
pub fn is_multithreaded() -> bool {
    use llvm_sys::core::LLVMIsMultithreaded;

    unsafe {
        LLVMIsMultithreaded() == 1
    }
}

// TODO: Move
pub fn load_library_permanently(filename: &str) -> bool {
    let filename = CString::new(filename).expect("Conversion to CString failed unexpectedly");

    unsafe {
        LLVMLoadLibraryPermanently(filename.as_ptr()) == 1
    }
}

// TODO: Move to better location?
// REVIEW: Not sure how safe this is. What happens when you make an llvm call after
// shutdown_llvm has been called?
pub fn shutdown_llvm() {
    use llvm_sys::core::LLVMShutdown;

    unsafe {
        LLVMShutdown()
    }
}

// Installs an error handler to be called before LLVM exits
// REVIEW: Maybe it's possible to have a safe wrapper? If we can
// wrap the provided function input ptr into a &CStr somehow
// TODOC: Can be used like this:
// extern "C" fn print_before_exit(msg: *const i8) {
//    let c_str = unsafe { ::std::ffi::CStr::from_ptr(msg) };
//
//    panic!("LLVM fatally errored: {:?}", c_str);
// }
// unsafe {
//     install_fatal_error_handler(print_before_exit);
// }
// and will be called before LLVM calls C exit(). IIRC
// it's safe to panic from C in newer versions of rust
pub unsafe fn install_fatal_error_handler(handler: extern "C" fn(*const i8)) {
    #[cfg(any(feature = "llvm3-6", feature = "llvm3-7"))]
    use llvm_sys::core::LLVMInstallFatalErrorHandler;
    #[cfg(any(feature = "llvm3-8", feature = "llvm3-9", feature = "llvm4-0", feature = "llvm5-0"))]
    use llvm_sys::error_handling::LLVMInstallFatalErrorHandler;

    LLVMInstallFatalErrorHandler(handler)
}

/// Resets LLVM's fatal error handler back to the default
pub fn reset_fatal_error_handler() {
    #[cfg(any(feature = "llvm3-6", feature = "llvm3-7"))]
    use llvm_sys::core::LLVMResetFatalErrorHandler;
    #[cfg(any(feature = "llvm3-8", feature = "llvm3-9", feature = "llvm4-0", feature = "llvm5-0"))]
    use llvm_sys::error_handling::LLVMResetFatalErrorHandler;

    unsafe {
        LLVMResetFatalErrorHandler()
    }
}

/// Defines the address space in which a global will be inserted.
///
/// # Remarks
/// See also: https://llvm.org/doxygen/NVPTXBaseInfo_8h_source.html
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum AddressSpace {
    Generic = 0,
    Global  = 1,
    Shared  = 3,
    Const   = 4,
    Local   = 5
}

impl From<u32> for AddressSpace {
    fn from(val: u32) -> Self {
        match val {
            0 => AddressSpace::Generic,
            1 => AddressSpace::Global,
            2 => AddressSpace::Shared,
            3 => AddressSpace::Const,
            4 => AddressSpace::Local,
            _ => unreachable!("Invalid value for AddressSpace"),
        }
    }
}

// REVIEW: Maybe this belongs in some sort of prelude?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntPredicate {
    EQ,
    NE,
    UGT,
    UGE,
    ULT,
    ULE,
    SGT,
    SGE,
    SLT,
    SLE,
}

impl IntPredicate {
    pub(crate) fn as_llvm_predicate(&self) -> LLVMIntPredicate {
        match *self {
            IntPredicate::EQ => LLVMIntPredicate::LLVMIntEQ,
            IntPredicate::NE => LLVMIntPredicate::LLVMIntNE,
            IntPredicate::UGT => LLVMIntPredicate::LLVMIntUGT,
            IntPredicate::UGE => LLVMIntPredicate::LLVMIntUGE,
            IntPredicate::ULT => LLVMIntPredicate::LLVMIntULT,
            IntPredicate::ULE => LLVMIntPredicate::LLVMIntULE,
            IntPredicate::SGT => LLVMIntPredicate::LLVMIntSGT,
            IntPredicate::SGE => LLVMIntPredicate::LLVMIntSGE,
            IntPredicate::SLT => LLVMIntPredicate::LLVMIntSLT,
            IntPredicate::SLE => LLVMIntPredicate::LLVMIntSLE,
        }
    }
}

// REVIEW: Maybe this belongs in some sort of prelude?
/// Defines how to compare a `left` and `right` float value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatPredicate {
    /// Returns true if `left` == `right` and neither are NaN
    OEQ,
    /// Returns true if `left` >= `right` and neither are NaN
    OGE,
    /// Returns true if `left` > `right` and neither are NaN
    OGT,
    /// Returns true if `left` <= `right` and neither are NaN
    OLE,
    /// Returns true if `left` < `right` and neither are NaN
    OLT,
    /// Returns true if `left` != `right` and neither are NaN
    ONE,
    /// Returns true if neither value is NaN
    ORD,
    /// Always returns false
    PredicateFalse,
    /// Always returns true
    PredicateTrue,
    /// Returns true if `left` == `right` or either is NaN
    UEQ,
    /// Returns true if `left` >= `right` or either is NaN
    UGE,
    /// Returns true if `left` > `right` or either is NaN
    UGT,
    /// Returns true if `left` <= `right` or either is NaN
    ULE,
    /// Returns true if `left` < `right` or either is NaN
    ULT,
    /// Returns true if `left` != `right` or either is NaN
    UNE,
    /// Returns true if either value is NaN
    UNO,
}

impl FloatPredicate {
    pub(crate) fn as_llvm_predicate(&self) -> LLVMRealPredicate {
        match *self {
            FloatPredicate::PredicateFalse => LLVMRealPredicate::LLVMRealPredicateFalse,
            FloatPredicate::OEQ => LLVMRealPredicate::LLVMRealOEQ,
            FloatPredicate::OGT => LLVMRealPredicate::LLVMRealOGT,
            FloatPredicate::OGE => LLVMRealPredicate::LLVMRealOGE,
            FloatPredicate::OLT => LLVMRealPredicate::LLVMRealOLT,
            FloatPredicate::OLE => LLVMRealPredicate::LLVMRealOLE,
            FloatPredicate::ONE => LLVMRealPredicate::LLVMRealONE,
            FloatPredicate::ORD => LLVMRealPredicate::LLVMRealORD,
            FloatPredicate::UNO => LLVMRealPredicate::LLVMRealUNO,
            FloatPredicate::UEQ => LLVMRealPredicate::LLVMRealUEQ,
            FloatPredicate::UGT => LLVMRealPredicate::LLVMRealUGT,
            FloatPredicate::UGE => LLVMRealPredicate::LLVMRealUGE,
            FloatPredicate::ULT => LLVMRealPredicate::LLVMRealULT,
            FloatPredicate::ULE => LLVMRealPredicate::LLVMRealULE,
            FloatPredicate::UNE => LLVMRealPredicate::LLVMRealUNE,
            FloatPredicate::PredicateTrue => LLVMRealPredicate::LLVMRealPredicateTrue,
        }
    }
}


/// Defines the optimization level used to compile a `Module`.
///
/// # Remarks
/// See also: https://llvm.org/doxygen/CodeGen_8h_source.html
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum OptimizationLevel {
    None       = 0,
    Less       = 1,
    Default    = 2,
    Aggressive = 3
}

impl Default for OptimizationLevel {
    /// Returns the default value for `OptimizationLevel`, namely `OptimizationLevel::Default`.
    fn default() -> Self {
        OptimizationLevel::Default
    }
}

// REVIEW: Maybe this belongs in some sort of prelude?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlobalVisibility {
    Default,
    Hidden,
    Protected,
}

impl Default for GlobalVisibility {
    /// Returns the default value for `GlobalVisibility`, namely `GlobalVisibility::Default`.
    fn default() -> Self {
        GlobalVisibility::Default
    }
}

impl GlobalVisibility {
    pub(crate) fn new(visibility: LLVMVisibility) -> Self {
        match visibility {
            LLVMVisibility::LLVMDefaultVisibility => GlobalVisibility::Default,
            LLVMVisibility::LLVMHiddenVisibility => GlobalVisibility::Hidden,
            LLVMVisibility::LLVMProtectedVisibility => GlobalVisibility::Protected,
        }
    }

    pub(crate) fn as_llvm_visibility(&self) -> LLVMVisibility {
        match *self {
            GlobalVisibility::Default => LLVMVisibility::LLVMDefaultVisibility,
            GlobalVisibility::Hidden => LLVMVisibility::LLVMHiddenVisibility,
            GlobalVisibility::Protected => LLVMVisibility::LLVMProtectedVisibility,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThreadLocalMode {
    GeneralDynamicTLSModel,
    LocalDynamicTLSModel,
    InitialExecTLSModel,
    LocalExecTLSModel,
}

impl ThreadLocalMode {
    pub(crate) fn new(thread_local_mode: LLVMThreadLocalMode) -> Option<Self> {
        match thread_local_mode {
            LLVMThreadLocalMode::LLVMGeneralDynamicTLSModel => Some(ThreadLocalMode::GeneralDynamicTLSModel),
            LLVMThreadLocalMode::LLVMLocalDynamicTLSModel => Some(ThreadLocalMode::LocalDynamicTLSModel),
            LLVMThreadLocalMode::LLVMInitialExecTLSModel => Some(ThreadLocalMode::InitialExecTLSModel),
            LLVMThreadLocalMode::LLVMLocalExecTLSModel => Some(ThreadLocalMode::LocalExecTLSModel),
            LLVMThreadLocalMode::LLVMNotThreadLocal => None
        }
    }

    pub(crate) fn as_llvm_mode(&self) -> LLVMThreadLocalMode {
        match *self {
            ThreadLocalMode::GeneralDynamicTLSModel => LLVMThreadLocalMode::LLVMGeneralDynamicTLSModel,
            ThreadLocalMode::LocalDynamicTLSModel => LLVMThreadLocalMode::LLVMLocalDynamicTLSModel,
            ThreadLocalMode::InitialExecTLSModel => LLVMThreadLocalMode::LLVMInitialExecTLSModel,
            ThreadLocalMode::LocalExecTLSModel => LLVMThreadLocalMode::LLVMLocalExecTLSModel,
            // None => LLVMThreadLocalMode::LLVMNotThreadLocal,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DLLStorageClass {
    Default,
    Import,
    Export,
}

impl Default for DLLStorageClass {
    /// Returns the default value for `DLLStorageClass`, namely `DLLStorageClass::Default`.
    fn default() -> Self {
        DLLStorageClass::Default
    }
}

impl DLLStorageClass {
    pub(crate) fn new(dll_storage_class: LLVMDLLStorageClass) -> Self {
        match dll_storage_class {
            LLVMDLLStorageClass::LLVMDefaultStorageClass => DLLStorageClass::Default,
            LLVMDLLStorageClass::LLVMDLLImportStorageClass => DLLStorageClass::Import,
            LLVMDLLStorageClass::LLVMDLLExportStorageClass => DLLStorageClass::Export,
        }
    }

    pub(crate) fn as_llvm_class(&self) -> LLVMDLLStorageClass {
        match *self {
            DLLStorageClass::Default => LLVMDLLStorageClass::LLVMDefaultStorageClass,
            DLLStorageClass::Import => LLVMDLLStorageClass::LLVMDLLImportStorageClass,
            DLLStorageClass::Export => LLVMDLLStorageClass::LLVMDLLExportStorageClass,
        }
    }
}

// TODO: impl Debug, Display
/// This is also known as an LLVMMessage
#[derive(Eq)]
pub struct LLVMString {
    ptr: *const i8,
}

impl LLVMString {
    pub(crate) fn new(ptr: *const i8) -> Self {
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
        write!(f, "{:?}", *self)
    }
}

impl Display for LLVMString {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", *self)
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
    Borrowed(*const i8),
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

// Misc Notes

// Initializer (new) strategy:
// assert!(!val.is_null()); where null is not expected to ever occur, but Option<Self>
// when null is expected to be passed at some point
