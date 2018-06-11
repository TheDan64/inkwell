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
pub mod support;
pub mod targets;
pub mod types;
pub mod values;

use llvm_sys::{LLVMIntPredicate, LLVMRealPredicate, LLVMVisibility, LLVMThreadLocalMode, LLVMDLLStorageClass};

#[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9", feature = "llvm4-0",
              feature = "llvm5-0", feature = "llvm6-0")))]
compile_error!("A LLVM feature flag must be provided. See the README for more details.");

// TODO: Probably move into error handling module
pub fn enable_llvm_pretty_stack_trace() {
    #[cfg(any(feature = "llvm3-6", feature = "llvm3-7"))]
    use llvm_sys::core::LLVMEnablePrettyStackTrace;
    #[cfg(any(feature = "llvm3-8", feature = "llvm3-9", feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0"))]
    use llvm_sys::error_handling::LLVMEnablePrettyStackTrace;

    unsafe {
        LLVMEnablePrettyStackTrace()
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

// Initializer (new) strategy:
// assert!(!val.is_null()); where null is not expected to ever occur, but Option<Self>
// when null is expected to be passed at some point
