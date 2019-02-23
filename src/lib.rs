//! Inkwell documentation is a work in progress.
//!
//! If you have any LLVM knowledge that could be used to improve these docs, we would greatly appreciate you opening an issue and/or a pull request on our [GitHub page](https://github.com/TheDan64/inkwell).
//!
//! Due to a rustdoc issue, this documentation represents only the latest supported LLVM version. We hope that this issue will be resolved in the future so that multiple versions can be documented side by side.
//!
//! # Library Wide Notes
//!
//! * Most functions which take a string slice as input may possibly panic in the unlikely event that a c style string cannot be created based on it. (IE if your slice already has a null byte in it)

#![deny(missing_debug_implementations)]
extern crate either;
#[macro_use]
extern crate enum_methods;
extern crate libc;
extern crate llvm_sys;
#[macro_use]
extern crate inkwell_internal_macros;

#[macro_use]
pub mod support;
#[deny(missing_docs)]
#[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8")))]
pub mod attributes;
#[deny(missing_docs)]
#[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9",
              feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
pub mod comdat;
#[deny(missing_docs)]
pub mod basic_block;
pub mod builder;
#[deny(missing_docs)]
pub mod context;
pub mod data_layout;
pub mod execution_engine;
pub mod memory_buffer;
#[deny(missing_docs)]
pub mod module;
pub mod object_file;
pub mod passes;
pub mod targets;
pub mod types;
pub mod values;

use llvm_sys::{LLVMIntPredicate, LLVMRealPredicate, LLVMVisibility, LLVMThreadLocalMode, LLVMDLLStorageClass, LLVMAtomicOrdering};

// Thanks to kennytm for coming up with assert_unique_features!
// which ensures that the LLVM feature flags are mutually exclusive
macro_rules! assert_unique_features {
    () => {};
    ($first:tt $(,$rest:tt)*) => {
        $(
            #[cfg(all(feature = $first, feature = $rest))]
            compile_error!(concat!("features \"", $first, "\" and \"", $rest, "\" cannot be used together"));
        )*
        assert_unique_features!($($rest),*);
    }
}

// This macro ensures that at least one of the LLVM feature
// flags are provided and prints them out if none are provided
macro_rules! assert_used_features {
    ($($all:tt),*) => {
        #[cfg(not(any($(feature = $all),*)))]
        compile_error!(concat!("One of the LLVM feature flags must be provided: ", $($all, " "),*));
    }
}

macro_rules! assert_unique_used_features {
    ($($all:tt),*) => {
        assert_unique_features!($($all),*);
        assert_used_features!($($all),*);
    }
}

assert_unique_used_features!{"llvm3-6", "llvm3-7", "llvm3-8", "llvm3-9", "llvm4-0", "llvm5-0", "llvm6-0", "llvm7-0"}

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
    Local   = 5,
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
enum_rename!{
    /// This enum defines how to compare a `left` and `right` `IntValue`.
    IntPredicate <=> LLVMIntPredicate {
        /// Equal
        EQ <=> LLVMIntEQ,
        /// Not Equal
        NE <=> LLVMIntNE,
        /// Unsigned Greater Than
        UGT <=> LLVMIntUGT,
        /// Unsigned Greater Than or Equal
        UGE <=> LLVMIntUGE,
        /// Unsigned Less Than
        ULT <=> LLVMIntULT,
        /// Unsigned Less Than or Equal
        ULE <=> LLVMIntULE,
        /// Signed Greater Than
        SGT <=> LLVMIntSGT,
        /// Signed Greater Than or Equal
        SGE <=> LLVMIntSGE,
        /// Signed Less Than
        SLT <=> LLVMIntSLT,
        /// Signed Less Than or Equal
        SLE <=> LLVMIntSLE,
    }
}

// REVIEW: Maybe this belongs in some sort of prelude?
enum_rename!{
    /// Defines how to compare a `left` and `right` `FloatValue`.
    FloatPredicate <=> LLVMRealPredicate {
        /// Returns true if `left` == `right` and neither are NaN
        OEQ <=> LLVMRealOEQ,
        /// Returns true if `left` >= `right` and neither are NaN
        OGE <=> LLVMRealOGE,
        /// Returns true if `left` > `right` and neither are NaN
        OGT <=> LLVMRealOGT,
        /// Returns true if `left` <= `right` and neither are NaN
        OLE <=> LLVMRealOLE,
        /// Returns true if `left` < `right` and neither are NaN
        OLT <=> LLVMRealOLT,
        /// Returns true if `left` != `right` and neither are NaN
        ONE <=> LLVMRealONE,
        /// Returns true if neither value is NaN
        ORD <=> LLVMRealORD,
        /// Always returns false
        PredicateFalse <=> LLVMRealPredicateFalse,
        /// Always returns true
        PredicateTrue <=> LLVMRealPredicateTrue,
        /// Returns true if `left` == `right` or either is NaN
        UEQ <=> LLVMRealUEQ,
        /// Returns true if `left` >= `right` or either is NaN
        UGE <=> LLVMRealUGE,
        /// Returns true if `left` > `right` or either is NaN
        UGT <=> LLVMRealUGT,
        /// Returns true if `left` <= `right` or either is NaN
        ULE <=> LLVMRealULE,
        /// Returns true if `left` < `right` or either is NaN
        ULT <=> LLVMRealULT,
        /// Returns true if `left` != `right` or either is NaN
        UNE <=> LLVMRealUNE,
        /// Returns true if either value is NaN
        UNO <=> LLVMRealUNO,
    }
}

// REVIEW: Maybe this belongs in some sort of prelude?
enum_rename!{
    AtomicOrdering <=> LLVMAtomicOrdering {
        NotAtomic <=> LLVMAtomicOrderingNotAtomic,
        Unordered <=> LLVMAtomicOrderingUnordered,
        Monotonic <=> LLVMAtomicOrderingMonotonic,
        Acquire <=> LLVMAtomicOrderingAcquire,
        Release <=> LLVMAtomicOrderingRelease,
        AcquireRelease <=> LLVMAtomicOrderingAcquireRelease,
        SequentiallyConsistent <=> LLVMAtomicOrderingSequentiallyConsistent,
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

enum_rename!{
    GlobalVisibility <=> LLVMVisibility {
        Default <=> LLVMDefaultVisibility,
        Hidden <=> LLVMHiddenVisibility,
        Protected <=> LLVMProtectedVisibility,
    }
}

impl Default for GlobalVisibility {
    /// Returns the default value for `GlobalVisibility`, namely `GlobalVisibility::Default`.
    fn default() -> Self {
        GlobalVisibility::Default
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

enum_rename! {
    DLLStorageClass <=> LLVMDLLStorageClass {
        Default <=> LLVMDefaultStorageClass,
        Import <=> LLVMDLLImportStorageClass,
        Export <=> LLVMDLLExportStorageClass,
    }
}

impl Default for DLLStorageClass {
    /// Returns the default value for `DLLStorageClass`, namely `DLLStorageClass::Default`.
    fn default() -> Self {
        DLLStorageClass::Default
    }
}
