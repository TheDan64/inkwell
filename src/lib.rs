extern crate either;
#[macro_use]
extern crate enum_methods;
extern crate libc;
extern crate llvm_sys;

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
use llvm_sys::core::LLVMResetFatalErrorHandler;
use llvm_sys::support::LLVMLoadLibraryPermanently;

use std::ffi::CString;

// TODO: Probably move into error handling module
pub fn enable_llvm_pretty_stack_trace() {
    // use llvm_sys::error_handling::LLVMEnablePrettyStackTrace; // v3.8
    use llvm_sys::core::LLVMEnablePrettyStackTrace;

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
// pub fn install_fatal_error_hanlder<H: Fn(&CStr)>(handler: H) {
//     extern "C" fn handler_wrapper(message: *const i8) {
//         let c_string = CStr::from_ptr(message);

//         handler(&c_string);
//     }

//     let handler_wrapper = |message: *const i8| {
//         let c_string = CStr::from_ptr(message);

//         handler(&c_string);
//     };

//     unsafe {
//         LLVMInstallFatalErrorHandler(handler_wrapper as LLVMFatalErrorHandler)
//     }
// }

/// Resets LLVM's fatal error handler back to the default
pub fn reset_fatal_error_handler() {
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

// Misc Notes
// Always pass a c_string.as_ptr() call into the function call directly and never
// before hand. Seems to make a huge difference (stuff stops working) otherwise

// Initializer (new) strategy:
// assert!(!val.is_null()); where null is not expected to ever occur, but Option<Self>
// when null is expected to be passed at some point
