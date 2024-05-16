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
#![allow(clippy::missing_safety_doc, clippy::too_many_arguments, clippy::result_unit_err)]
#![cfg_attr(feature = "nightly", feature(doc_cfg))]

#[macro_use]
extern crate inkwell_internals;

#[macro_use]
pub mod support;
#[deny(missing_docs)]
pub mod attributes;
#[deny(missing_docs)]
pub mod basic_block;
pub mod builder;
#[deny(missing_docs)]
#[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
pub mod comdat;
#[deny(missing_docs)]
pub mod context;
pub mod data_layout;
#[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
pub mod debug_info;
pub mod execution_engine;
pub mod intrinsics;
pub mod memory_buffer;
#[deny(missing_docs)]
pub mod module;
pub mod object_file;
pub mod passes;
pub mod targets;
pub mod types;
pub mod values;

// Boilerplate to select a desired llvm_sys version at compile & link time.
#[cfg(feature = "llvm10-0")]
pub extern crate llvm_sys_100 as llvm_sys;
#[cfg(feature = "llvm11-0")]
pub extern crate llvm_sys_110 as llvm_sys;
#[cfg(feature = "llvm12-0")]
pub extern crate llvm_sys_120 as llvm_sys;
#[cfg(feature = "llvm13-0")]
pub extern crate llvm_sys_130 as llvm_sys;
#[cfg(feature = "llvm14-0")]
pub extern crate llvm_sys_140 as llvm_sys;
#[cfg(feature = "llvm15-0")]
pub extern crate llvm_sys_150 as llvm_sys;
#[cfg(feature = "llvm16-0")]
pub extern crate llvm_sys_160 as llvm_sys;
#[cfg(feature = "llvm17-0")]
pub extern crate llvm_sys_170 as llvm_sys;
#[cfg(feature = "llvm18-0")]
pub extern crate llvm_sys_180 as llvm_sys;
#[cfg(feature = "llvm4-0")]
pub extern crate llvm_sys_40 as llvm_sys;
#[cfg(feature = "llvm5-0")]
pub extern crate llvm_sys_50 as llvm_sys;
#[cfg(feature = "llvm6-0")]
pub extern crate llvm_sys_60 as llvm_sys;
#[cfg(feature = "llvm7-0")]
pub extern crate llvm_sys_70 as llvm_sys;
#[cfg(feature = "llvm8-0")]
pub extern crate llvm_sys_80 as llvm_sys;
#[cfg(feature = "llvm9-0")]
pub extern crate llvm_sys_90 as llvm_sys;

use llvm_sys::target_machine::LLVMCodeGenOptLevel;
use llvm_sys::{
    LLVMAtomicOrdering, LLVMAtomicRMWBinOp, LLVMDLLStorageClass, LLVMIntPredicate, LLVMRealPredicate,
    LLVMThreadLocalMode, LLVMVisibility,
};

#[llvm_versions(7..)]
use llvm_sys::LLVMInlineAsmDialect;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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

assert_unique_used_features! {
    "llvm4-0",
    "llvm5-0",
    "llvm6-0",
    "llvm7-0",
    "llvm8-0",
    "llvm9-0",
    "llvm10-0",
    "llvm11-0",
    "llvm12-0",
    "llvm13-0",
    "llvm14-0",
    "llvm15-0",
    "llvm16-0",
    "llvm17-0",
    "llvm18-0"
}

/// Defines the address space in which a global will be inserted.
///
/// The default address space is zero. An address space can always be created from a `u16`:
/// ```no_run
/// inkwell::AddressSpace::from(1u16);
/// ```
///
/// An Address space is a 24-bit number. To convert from a u32, use the `TryFrom` instance
///
/// ```no_run
/// inkwell::AddressSpace::try_from(42u32).expect("fits in 24-bit unsigned int");
/// ```
///
/// # Remarks
/// See also: https://llvm.org/doxygen/NVPTXBaseInfo_8h_source.html
#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
pub struct AddressSpace(u32);

impl From<u16> for AddressSpace {
    fn from(val: u16) -> Self {
        AddressSpace(val as u32)
    }
}

impl TryFrom<u32> for AddressSpace {
    type Error = ();

    fn try_from(val: u32) -> Result<Self, Self::Error> {
        // address space is a 24-bit integer
        if val < 1 << 24 {
            Ok(AddressSpace(val))
        } else {
            Err(())
        }
    }
}

// REVIEW: Maybe this belongs in some sort of prelude?
/// This enum defines how to compare a `left` and `right` `IntValue`.
#[llvm_enum(LLVMIntPredicate)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IntPredicate {
    /// Equal
    #[llvm_variant(LLVMIntEQ)]
    EQ,

    /// Not Equal
    #[llvm_variant(LLVMIntNE)]
    NE,

    /// Unsigned Greater Than
    #[llvm_variant(LLVMIntUGT)]
    UGT,

    /// Unsigned Greater Than or Equal
    #[llvm_variant(LLVMIntUGE)]
    UGE,

    /// Unsigned Less Than
    #[llvm_variant(LLVMIntULT)]
    ULT,

    /// Unsigned Less Than or Equal
    #[llvm_variant(LLVMIntULE)]
    ULE,

    /// Signed Greater Than
    #[llvm_variant(LLVMIntSGT)]
    SGT,

    /// Signed Greater Than or Equal
    #[llvm_variant(LLVMIntSGE)]
    SGE,

    /// Signed Less Than
    #[llvm_variant(LLVMIntSLT)]
    SLT,

    /// Signed Less Than or Equal
    #[llvm_variant(LLVMIntSLE)]
    SLE,
}

// REVIEW: Maybe this belongs in some sort of prelude?
/// Defines how to compare a `left` and `right` `FloatValue`.
#[llvm_enum(LLVMRealPredicate)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FloatPredicate {
    /// Returns true if `left` == `right` and neither are NaN
    #[llvm_variant(LLVMRealOEQ)]
    OEQ,

    /// Returns true if `left` >= `right` and neither are NaN
    #[llvm_variant(LLVMRealOGE)]
    OGE,

    /// Returns true if `left` > `right` and neither are NaN
    #[llvm_variant(LLVMRealOGT)]
    OGT,

    /// Returns true if `left` <= `right` and neither are NaN
    #[llvm_variant(LLVMRealOLE)]
    OLE,

    /// Returns true if `left` < `right` and neither are NaN
    #[llvm_variant(LLVMRealOLT)]
    OLT,

    /// Returns true if `left` != `right` and neither are NaN
    #[llvm_variant(LLVMRealONE)]
    ONE,

    /// Returns true if neither value is NaN
    #[llvm_variant(LLVMRealORD)]
    ORD,

    /// Always returns false
    #[llvm_variant(LLVMRealPredicateFalse)]
    PredicateFalse,

    /// Always returns true
    #[llvm_variant(LLVMRealPredicateTrue)]
    PredicateTrue,

    /// Returns true if `left` == `right` or either is NaN
    #[llvm_variant(LLVMRealUEQ)]
    UEQ,

    /// Returns true if `left` >= `right` or either is NaN
    #[llvm_variant(LLVMRealUGE)]
    UGE,

    /// Returns true if `left` > `right` or either is NaN
    #[llvm_variant(LLVMRealUGT)]
    UGT,

    /// Returns true if `left` <= `right` or either is NaN
    #[llvm_variant(LLVMRealULE)]
    ULE,

    /// Returns true if `left` < `right` or either is NaN
    #[llvm_variant(LLVMRealULT)]
    ULT,

    /// Returns true if `left` != `right` or either is NaN
    #[llvm_variant(LLVMRealUNE)]
    UNE,

    /// Returns true if either value is NaN
    #[llvm_variant(LLVMRealUNO)]
    UNO,
}

// REVIEW: Maybe this belongs in some sort of prelude?
#[llvm_enum(LLVMAtomicOrdering)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AtomicOrdering {
    #[llvm_variant(LLVMAtomicOrderingNotAtomic)]
    NotAtomic,
    #[llvm_variant(LLVMAtomicOrderingUnordered)]
    Unordered,
    #[llvm_variant(LLVMAtomicOrderingMonotonic)]
    Monotonic,
    #[llvm_variant(LLVMAtomicOrderingAcquire)]
    Acquire,
    #[llvm_variant(LLVMAtomicOrderingRelease)]
    Release,
    #[llvm_variant(LLVMAtomicOrderingAcquireRelease)]
    AcquireRelease,
    #[llvm_variant(LLVMAtomicOrderingSequentiallyConsistent)]
    SequentiallyConsistent,
}

#[llvm_enum(LLVMAtomicRMWBinOp)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AtomicRMWBinOp {
    /// Stores to memory and returns the prior value.
    #[llvm_variant(LLVMAtomicRMWBinOpXchg)]
    Xchg,

    /// Adds to the value in memory and returns the prior value.
    #[llvm_variant(LLVMAtomicRMWBinOpAdd)]
    Add,

    /// Subtract a value off the value in memory and returns the prior value.
    #[llvm_variant(LLVMAtomicRMWBinOpSub)]
    Sub,

    /// Bitwise and into memory and returns the prior value.
    #[llvm_variant(LLVMAtomicRMWBinOpAnd)]
    And,

    /// Bitwise nands into memory and returns the prior value.
    #[llvm_variant(LLVMAtomicRMWBinOpNand)]
    Nand,

    /// Bitwise ors into memory and returns the prior value.
    #[llvm_variant(LLVMAtomicRMWBinOpOr)]
    Or,

    /// Bitwise xors into memory and returns the prior value.
    #[llvm_variant(LLVMAtomicRMWBinOpXor)]
    Xor,

    /// Sets memory to the signed-greater of the value provided and the value in memory. Returns the value that was in memory.
    #[llvm_variant(LLVMAtomicRMWBinOpMax)]
    Max,

    /// Sets memory to the signed-lesser of the value provided and the value in memory. Returns the value that was in memory.
    #[llvm_variant(LLVMAtomicRMWBinOpMin)]
    Min,

    /// Sets memory to the unsigned-greater of the value provided and the value in memory. Returns the value that was in memory.
    #[llvm_variant(LLVMAtomicRMWBinOpUMax)]
    UMax,

    /// Sets memory to the unsigned-lesser of the value provided and the value in memory. Returns the value that was in memory.
    #[llvm_variant(LLVMAtomicRMWBinOpUMin)]
    UMin,

    /// Adds to the float-typed value in memory and returns the prior value.
    // Although this was added in LLVM 9, it wasn't exposed to the C API
    // until 10.0.
    #[llvm_versions(10..)]
    #[llvm_variant(LLVMAtomicRMWBinOpFAdd)]
    FAdd,

    /// Subtract a float-typed value off the value in memory and returns the prior value.
    // Although this was added in LLVM 9, it wasn't exposed to the C API
    // until 10.0.
    #[llvm_versions(10..)]
    #[llvm_variant(LLVMAtomicRMWBinOpFSub)]
    FSub,

    /// Sets memory to the greater of the two float-typed values, one provided and one from memory. Returns the value that was in memory.
    #[llvm_versions(15..)]
    #[llvm_variant(LLVMAtomicRMWBinOpFMax)]
    FMax,

    /// Sets memory to the lesser of the two float-typed values, one provided and one from memory. Returns the value that was in memory.
    #[llvm_versions(15..)]
    #[llvm_variant(LLVMAtomicRMWBinOpFMin)]
    FMin,
}

/// Defines the optimization level used to compile a `Module`.
///
/// # Remarks
/// See also: https://llvm.org/doxygen/CodeGen_8h_source.html
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OptimizationLevel {
    None = 0,
    Less = 1,
    Default = 2,
    Aggressive = 3,
}

impl Default for OptimizationLevel {
    /// Returns the default value for `OptimizationLevel`, namely `OptimizationLevel::Default`.
    fn default() -> Self {
        OptimizationLevel::Default
    }
}

impl From<OptimizationLevel> for LLVMCodeGenOptLevel {
    fn from(value: OptimizationLevel) -> Self {
        match value {
            OptimizationLevel::None => LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            OptimizationLevel::Less => LLVMCodeGenOptLevel::LLVMCodeGenLevelLess,
            OptimizationLevel::Default => LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            OptimizationLevel::Aggressive => LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
        }
    }
}

#[llvm_enum(LLVMVisibility)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GlobalVisibility {
    #[llvm_variant(LLVMDefaultVisibility)]
    Default,
    #[llvm_variant(LLVMHiddenVisibility)]
    Hidden,
    #[llvm_variant(LLVMProtectedVisibility)]
    Protected,
}

impl Default for GlobalVisibility {
    /// Returns the default value for `GlobalVisibility`, namely `GlobalVisibility::Default`.
    fn default() -> Self {
        GlobalVisibility::Default
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
            LLVMThreadLocalMode::LLVMNotThreadLocal => None,
        }
    }

    pub(crate) fn as_llvm_mode(self) -> LLVMThreadLocalMode {
        match self {
            ThreadLocalMode::GeneralDynamicTLSModel => LLVMThreadLocalMode::LLVMGeneralDynamicTLSModel,
            ThreadLocalMode::LocalDynamicTLSModel => LLVMThreadLocalMode::LLVMLocalDynamicTLSModel,
            ThreadLocalMode::InitialExecTLSModel => LLVMThreadLocalMode::LLVMInitialExecTLSModel,
            ThreadLocalMode::LocalExecTLSModel => LLVMThreadLocalMode::LLVMLocalExecTLSModel,
            // None => LLVMThreadLocalMode::LLVMNotThreadLocal,
        }
    }
}

#[llvm_enum(LLVMDLLStorageClass)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DLLStorageClass {
    #[llvm_variant(LLVMDefaultStorageClass)]
    Default,
    #[llvm_variant(LLVMDLLImportStorageClass)]
    Import,
    #[llvm_variant(LLVMDLLExportStorageClass)]
    Export,
}

impl Default for DLLStorageClass {
    /// Returns the default value for `DLLStorageClass`, namely `DLLStorageClass::Default`.
    fn default() -> Self {
        DLLStorageClass::Default
    }
}

#[llvm_versions(7..)]
#[llvm_enum(LLVMInlineAsmDialect)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InlineAsmDialect {
    #[llvm_variant(LLVMInlineAsmDialectATT)]
    ATT,
    #[llvm_variant(LLVMInlineAsmDialectIntel)]
    Intel,
}
