#[llvm_versions(14..)]
use llvm_sys::core::LLVMGetGEPSourceElementType;
use llvm_sys::core::{
    LLVMGetAlignment, LLVMGetAllocatedType, LLVMGetAtomicRMWBinOp, LLVMGetFCmpPredicate, LLVMGetICmpPredicate,
    LLVMGetIndices, LLVMGetInstructionOpcode, LLVMGetInstructionParent, LLVMGetMetadata, LLVMGetNextInstruction,
    LLVMGetNumIndices, LLVMGetNumOperands, LLVMGetOperand, LLVMGetOperandUse, LLVMGetOrdering,
    LLVMGetPreviousInstruction, LLVMGetVolatile, LLVMHasMetadata, LLVMInstructionClone, LLVMInstructionEraseFromParent,
    LLVMInstructionRemoveFromParent, LLVMIsATerminatorInst, LLVMIsConditional, LLVMIsTailCall, LLVMSetAlignment,
    LLVMSetMetadata, LLVMSetOperand, LLVMSetOrdering, LLVMSetVolatile, LLVMValueAsBasicBlock, LLVMValueIsBasicBlock,
};
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMOpcode;

use std::{ffi::CStr, fmt, fmt::Display};

use crate::debug_info::DILocation;
use crate::values::{BasicValue, BasicValueEnum, BasicValueUse, MetadataValue, Value};
use crate::AtomicRMWBinOp;
use crate::{basic_block::BasicBlock, types::AnyTypeEnum};
use crate::{error::AlignmentError, values::basic_value_use::Operand};
use crate::{types::BasicTypeEnum, values::traits::AsValueRef};
use crate::{AtomicOrdering, FloatPredicate, IntPredicate};

use super::AnyValue;

/// Errors for atomic operations on load/store instructions.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AtomicError {
    #[error("The release ordering is not valid on load instructions.")]
    ReleaseOnLoad,
    #[error("The acq_rel ordering is not valid on load or store instructions.")]
    AcquireRelease,
    #[error("The acquire ordering is not valid on store instructions.")]
    AcquireOnStore,
    #[error("Only acquire, release, acq_rel and sequentially consistent orderings are valid on fence instructions.")]
    InvalidOrderingOnFence,
    #[error("The not_atomic and unordered orderings are not valid on atomicrmw instructions.")]
    InvalidOrderingOnAtomicRMW,
}

/// Errors for InstructionValue.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InstructionValueError {
    #[error("Cannot set name of a void-type instruction.")]
    CannotNameVoidTypeInst,
    #[error("Value is not a load, store, atomicrmw or cmpxchg instruction.")]
    NotMemoryAccessInst,
    #[error("Value is not an atomic ordering capable memory access instruction.")]
    NotAtomicOrderingInst,
    #[error("Value is not an alloca instruction.")]
    NotAllocaInst,
    #[error("Value is not an icmp instruction.")]
    NotIcmpInst,
    #[error("Value is not an add, sub, mul, shl or trunc instruction.")]
    NotArithInst,
    #[error("Value is not a div or shr instruction.")]
    NotDivOrShrInst,
    #[error("Alignment Error: {0}")]
    AlignmentError(AlignmentError),
    #[error("Not a GEP instruction.")]
    NotGEPInst,
    #[error("Not a fast-math supporting instruction.")]
    NotFastMathInst,
    #[error("Not a call instruction.")]
    NotCallInst,
    #[error("Not a zext instruction.")]
    NotZextInst,
    #[error("Not an or instruction.")]
    NotOrInst,
    #[error("Atomic Error: {0}")]
    AtomicError(AtomicError),
    #[error("Metadata is expected to be a node.")]
    ExpectedNode,
}

// REVIEW: Split up into structs for SubTypes on InstructionValues?
// REVIEW: This should maybe be split up into InstructionOpcode and ConstOpcode?
// see LLVMGetConstOpcode
#[llvm_enum(LLVMOpcode)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionOpcode {
    // Actual Instructions:
    Add,
    AddrSpaceCast,
    Alloca,
    And,
    AShr,
    AtomicCmpXchg,
    AtomicRMW,
    BitCast,
    Br,
    Call,
    CallBr,
    CatchPad,
    CatchRet,
    CatchSwitch,
    CleanupPad,
    CleanupRet,
    ExtractElement,
    ExtractValue,
    FNeg,
    FAdd,
    FCmp,
    FDiv,
    Fence,
    FMul,
    FPExt,
    FPToSI,
    FPToUI,
    FPTrunc,
    Freeze,
    FRem,
    FSub,
    GetElementPtr,
    ICmp,
    IndirectBr,
    InsertElement,
    InsertValue,
    IntToPtr,
    Invoke,
    LandingPad,
    Load,
    LShr,
    Mul,
    Or,
    #[llvm_variant(LLVMPHI)]
    Phi,
    PtrToInt,
    Resume,
    #[llvm_variant(LLVMRet)]
    Return,
    SDiv,
    Select,
    SExt,
    Shl,
    ShuffleVector,
    SIToFP,
    SRem,
    Store,
    Sub,
    Switch,
    Trunc,
    UDiv,
    UIToFP,
    Unreachable,
    URem,
    UserOp1,
    UserOp2,
    VAArg,
    Xor,
    ZExt,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InstructionValue<'ctx> {
    instruction_value: Value<'ctx>,
}

impl<'ctx> InstructionValue<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of type instruction.
    pub unsafe fn new(instruction_value: LLVMValueRef) -> Self {
        debug_assert!(!instruction_value.is_null());

        let value = Value::new(instruction_value);

        debug_assert!(value.is_instruction());

        InstructionValue {
            instruction_value: value,
        }
    }

    /// Creates a clone of this `InstructionValue`, and returns it.
    /// The clone will have no parent, and no name.
    pub fn explicit_clone(&self) -> Self {
        unsafe { Self::new(LLVMInstructionClone(self.as_value_ref())) }
    }

    /// Get name of the `InstructionValue`.
    pub fn get_name(&self) -> Option<&CStr> {
        if self.get_type().is_void_type() {
            None
        } else {
            Some(self.instruction_value.get_name())
        }
    }

    /// Get a instruction with it's name
    /// Compares against all instructions after self, and self.
    pub fn get_instruction_with_name(&self, name: &str) -> Option<InstructionValue<'ctx>> {
        if let Some(ins_name) = self.get_name() {
            if ins_name.to_str() == Ok(name) {
                return Some(*self);
            }
        }
        self.get_next_instruction()?.get_instruction_with_name(name)
    }

    /// Set name of the `InstructionValue`.
    pub fn set_name(&self, name: &str) -> Result<(), InstructionValueError> {
        if self.get_type().is_void_type() {
            Err(InstructionValueError::CannotNameVoidTypeInst)
        } else {
            self.instruction_value.set_name(name);
            Ok(())
        }
    }

    /// Get type of the current InstructionValue
    pub fn get_type(self) -> AnyTypeEnum<'ctx> {
        unsafe { AnyTypeEnum::new(self.instruction_value.get_type()) }
    }

    pub fn get_opcode(self) -> InstructionOpcode {
        let opcode = unsafe { LLVMGetInstructionOpcode(self.as_value_ref()) };

        InstructionOpcode::new(opcode)
    }

    pub fn get_previous_instruction(self) -> Option<Self> {
        let value = unsafe { LLVMGetPreviousInstruction(self.as_value_ref()) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(InstructionValue::new(value)) }
    }

    pub fn get_next_instruction(self) -> Option<Self> {
        let value = unsafe { LLVMGetNextInstruction(self.as_value_ref()) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(InstructionValue::new(value)) }
    }

    // REVIEW: Potentially unsafe if parent BB or grandparent fn were removed?
    pub fn erase_from_basic_block(self) {
        unsafe { LLVMInstructionEraseFromParent(self.as_value_ref()) }
    }

    // REVIEW: Potentially unsafe if parent BB or grandparent fn were removed?
    pub fn remove_from_basic_block(self) {
        unsafe { LLVMInstructionRemoveFromParent(self.as_value_ref()) }
    }

    // REVIEW: Potentially unsafe is parent BB or grandparent fn was deleted
    // REVIEW: Should this *not* be an option? Parent should always exist,
    // but I doubt LLVM returns null if the parent BB (or grandparent FN)
    // was deleted... Invalid memory is more likely. Cloned IV will have no
    // parent?
    pub fn get_parent(self) -> Option<BasicBlock<'ctx>> {
        unsafe { BasicBlock::new(LLVMGetInstructionParent(self.as_value_ref())) }
    }

    /// Returns if the instruction is a terminator
    pub fn is_terminator(self) -> bool {
        unsafe { !LLVMIsATerminatorInst(self.as_value_ref()).is_null() }
    }

    // SubTypes: Only apply to branch instructions
    /// Returns `Some` with whether or not the branch is conditional,
    /// otherwise `None` if not a branch instruction
    pub fn is_conditional(self) -> Option<bool> {
        if self.get_opcode() == InstructionOpcode::Br {
            Some(unsafe { LLVMIsConditional(self.as_value_ref()) == 1 })
        } else {
            None
        }
    }

    // SubTypes: Only apply to call instructions
    /// Returns `Some` with whether or not the call is a tail call,
    /// otherwise `None` if not a call instruction
    pub fn is_tail_call(self) -> Option<bool> {
        if self.get_opcode() == InstructionOpcode::Call {
            Some(unsafe { LLVMIsTailCall(self.as_value_ref()) == 1 })
        } else {
            None
        }
    }

    // SubTypes: Only apply to call instructions
    /// Returns tail call kind of the call instruction
    #[llvm_versions(18..)]
    pub fn get_tail_call_kind(self) -> Result<super::LLVMTailCallKind, InstructionValueError> {
        if self.get_opcode() == InstructionOpcode::Call {
            Ok(unsafe { llvm_sys::core::LLVMGetTailCallKind(self.as_value_ref()) })
        } else {
            Err(InstructionValueError::NotCallInst)
        }
    }

    // SubTypes: Only apply to call instructions
    /// Sets tail call kind of the call instruction
    #[llvm_versions(18..)]
    pub fn set_tail_call_kind(self, kind: super::LLVMTailCallKind) -> Result<(), InstructionValueError> {
        if self.get_opcode() == InstructionOpcode::Call {
            unsafe { llvm_sys::core::LLVMSetTailCallKind(self.as_value_ref(), kind) };
            Ok(())
        } else {
            Err(InstructionValueError::NotCallInst)
        }
    }

    /// Check whether this instructions supports [fast math flags][0].
    ///
    /// [0]: https://llvm.org/docs/LangRef.html#fast-math-flags
    #[llvm_versions(18..)]
    pub fn can_use_fast_math_flags(self) -> bool {
        unsafe { llvm_sys::core::LLVMCanValueUseFastMathFlags(self.as_value_ref()) == 1 }
    }

    // SubTypes: Only apply to fast-math supporting instructions
    /// Return the [`FastMathFlags`] on supported instructions.
    #[llvm_versions(18..)]
    pub fn get_fast_math_flags(self) -> Result<FastMathFlags, InstructionValueError> {
        if self.can_use_fast_math_flags() {
            let raw = unsafe { llvm_sys::core::LLVMGetFastMathFlags(self.as_value_ref()) };
            Ok(FastMathFlags::from_bits_retain(raw))
        } else {
            Err(InstructionValueError::NotFastMathInst)
        }
    }

    // SubTypes: Only apply to fast-math supporting instructions
    /// Set [`FastMathFlags`] on supported instructions.
    #[llvm_versions(18..)]
    pub fn set_fast_math_flags(self, flags: FastMathFlags) -> Result<(), InstructionValueError> {
        if self.can_use_fast_math_flags() {
            unsafe { llvm_sys::core::LLVMSetFastMathFlags(self.as_value_ref(), flags.bits()) };
            Ok(())
        } else {
            Err(InstructionValueError::NotFastMathInst)
        }
    }

    // SubTypes: Only apply to zext instructions
    /// Returns the non-negative flag on zext instructions.
    #[llvm_versions(18..)]
    pub fn get_non_negative_flag(self) -> Result<bool, InstructionValueError> {
        if self.get_opcode() == InstructionOpcode::ZExt {
            Ok(unsafe { llvm_sys::core::LLVMGetNNeg(self.as_value_ref()) == 1 })
        } else {
            Err(InstructionValueError::NotZextInst)
        }
    }

    // SubTypes: Only apply to zext instructions
    /// Set the non-negative flag on zext instructions.
    #[llvm_versions(18..)]
    pub fn set_non_negative_flag(self, flag: bool) -> Result<(), InstructionValueError> {
        if self.get_opcode() == InstructionOpcode::ZExt {
            unsafe { llvm_sys::core::LLVMSetNNeg(self.as_value_ref(), flag as i32) };
            Ok(())
        } else {
            Err(InstructionValueError::NotZextInst)
        }
    }

    // SubTypes: Only apply to or instructions
    /// Returns the disjoint flag on or instructions.
    #[llvm_versions(18..)]
    pub fn get_disjoint_flag(self) -> Result<bool, InstructionValueError> {
        if self.get_opcode() == InstructionOpcode::Or {
            Ok(unsafe { llvm_sys::core::LLVMGetIsDisjoint(self.as_value_ref()) == 1 })
        } else {
            Err(InstructionValueError::NotOrInst)
        }
    }

    // SubTypes: Only apply to or instructions
    /// Set the disjoint flag on or instructions.
    #[llvm_versions(18..)]
    pub fn set_disjoint_flag(self, flag: bool) -> Result<(), InstructionValueError> {
        if self.get_opcode() == InstructionOpcode::Or {
            unsafe { llvm_sys::core::LLVMSetIsDisjoint(self.as_value_ref(), flag as i32) };
            Ok(())
        } else {
            Err(InstructionValueError::NotOrInst)
        }
    }

    // SubTypes: Only apply to specific arithmetic instructions
    /// Returns whether or not an arithmetic instruction has the no signed wrap flag set.
    #[llvm_versions(17..)]
    pub fn get_no_signed_wrap_flag(self) -> Result<bool, InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::Add
            | InstructionOpcode::Sub
            | InstructionOpcode::Mul
            | InstructionOpcode::Shl
            | InstructionOpcode::Trunc => Ok(unsafe { llvm_sys::core::LLVMGetNSW(self.as_value_ref()) == 1 }),
            _ => Err(InstructionValueError::NotArithInst),
        }
    }

    // SubTypes: Only apply to specific arithmetic instructions
    /// Sets whether or not an arithmetic instruction is no signed wrap.
    #[llvm_versions(17..)]
    pub fn set_no_signed_wrap_flag(self, flag: bool) -> Result<(), InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::Add
            | InstructionOpcode::Sub
            | InstructionOpcode::Mul
            | InstructionOpcode::Shl
            | InstructionOpcode::Trunc => {
                unsafe { llvm_sys::core::LLVMSetNSW(self.as_value_ref(), flag as i32) };
                Ok(())
            },
            _ => Err(InstructionValueError::NotArithInst),
        }
    }

    // SubTypes: Only apply to specific arithmetic instructions
    /// Returns whether or not an arithmetic instruction has the no unsigned wrap flag set.
    #[llvm_versions(17..)]
    pub fn get_no_unsigned_wrap_flag(self) -> Result<bool, InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::Add
            | InstructionOpcode::Sub
            | InstructionOpcode::Mul
            | InstructionOpcode::Shl
            | InstructionOpcode::Trunc => Ok(unsafe { llvm_sys::core::LLVMGetNUW(self.as_value_ref()) == 1 }),
            _ => Err(InstructionValueError::NotArithInst),
        }
    }

    // SubTypes: Only apply to specific arithmetic instructions
    /// Sets whether or not an arithmetic instruction is no unsigned wrap.
    #[llvm_versions(17..)]
    pub fn set_no_unsigned_wrap_flag(self, flag: bool) -> Result<(), InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::Add
            | InstructionOpcode::Sub
            | InstructionOpcode::Mul
            | InstructionOpcode::Shl
            | InstructionOpcode::Trunc => {
                unsafe { llvm_sys::core::LLVMSetNUW(self.as_value_ref(), flag as i32) };
                Ok(())
            },
            _ => Err(InstructionValueError::NotArithInst),
        }
    }

    // SubTypes: Only apply to division and shift right instructions
    /// Returns whether or not an instruction has the exact flag set.
    #[llvm_versions(17..)]
    pub fn get_exact_flag(self) -> Result<bool, InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::SDiv | InstructionOpcode::UDiv | InstructionOpcode::AShr | InstructionOpcode::LShr => {
                Ok(unsafe { llvm_sys::core::LLVMGetExact(self.as_value_ref()) == 1 })
            },
            _ => Err(InstructionValueError::NotDivOrShrInst),
        }
    }

    // SubTypes: Only apply to division and shift right instructions
    /// Sets whether or not an instruction is exact.
    #[llvm_versions(17..)]
    pub fn set_exact_flag(self, flag: bool) -> Result<(), InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::SDiv | InstructionOpcode::UDiv | InstructionOpcode::AShr | InstructionOpcode::LShr => {
                unsafe { llvm_sys::core::LLVMSetExact(self.as_value_ref(), flag as i32) };
                Ok(())
            },
            _ => Err(InstructionValueError::NotDivOrShrInst),
        }
    }

    // SubTypes: Only apply to integer comparison instruction
    /// Returns whether or not an instruction has the same sign flag set.
    #[llvm_versions(21..)]
    pub fn get_same_sign_flag(self) -> Result<bool, InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::ICmp => Ok(unsafe { llvm_sys::core::LLVMGetICmpSameSign(self.as_value_ref()) == 1 }),
            _ => Err(InstructionValueError::NotIcmpInst),
        }
    }

    // SubTypes: Only apply to integer comparison instruction
    /// Sets whether or not an instruction is same sign.
    #[llvm_versions(21..)]
    pub fn set_same_sign_flag(self, flag: bool) -> Result<(), InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::ICmp => {
                unsafe { llvm_sys::core::LLVMSetICmpSameSign(self.as_value_ref(), flag as i32) };
                Ok(())
            },
            _ => Err(InstructionValueError::NotIcmpInst),
        }
    }

    pub fn replace_all_uses_with(self, other: &InstructionValue<'ctx>) {
        self.instruction_value.replace_all_uses_with(other.as_value_ref())
    }

    // SubTypes: Only apply to memory access instructions
    /// Returns whether or not a memory access instruction is volatile.
    pub fn get_volatile(self) -> Result<bool, InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::Load
            | InstructionOpcode::Store
            | InstructionOpcode::AtomicRMW
            | InstructionOpcode::AtomicCmpXchg => Ok(unsafe { LLVMGetVolatile(self.as_value_ref()) } == 1),
            _ => Err(InstructionValueError::NotMemoryAccessInst),
        }
    }

    // SubTypes: Only apply to memory access instructions
    /// Sets whether or not a memory access instruction is volatile.
    pub fn set_volatile(self, volatile: bool) -> Result<(), InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::Load
            | InstructionOpcode::Store
            | InstructionOpcode::AtomicRMW
            | InstructionOpcode::AtomicCmpXchg => {
                unsafe { LLVMSetVolatile(self.as_value_ref(), volatile as i32) };
                Ok(())
            },
            _ => Err(InstructionValueError::NotMemoryAccessInst),
        }
    }

    // SubTypes: Only apply to alloca instruction
    /// Returns the type that is allocated by the alloca instruction.
    pub fn get_allocated_type(self) -> Result<BasicTypeEnum<'ctx>, InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::Alloca => Ok(unsafe { BasicTypeEnum::new(LLVMGetAllocatedType(self.as_value_ref())) }),
            _ => Err(InstructionValueError::NotAllocaInst),
        }
    }

    // SubTypes: Only apply to GetElementPtr instruction
    /// Returns the source element type of the given GEP.
    #[llvm_versions(14..)]
    pub fn get_gep_source_element_type(self) -> Result<BasicTypeEnum<'ctx>, InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::GetElementPtr => {
                Ok(unsafe { BasicTypeEnum::new(LLVMGetGEPSourceElementType(self.as_value_ref())) })
            },
            _ => Err(InstructionValueError::NotGEPInst),
        }
    }

    // SubTypes: Only apply to GetElementPtr instruction
    /// Returns whether or not the GEP is in bounds.
    pub fn get_in_bounds_flag(self) -> Result<bool, InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::GetElementPtr => Ok(unsafe { llvm_sys::core::LLVMIsInBounds(self.as_value_ref()) == 1 }),
            _ => Err(InstructionValueError::NotGEPInst),
        }
    }

    // SubTypes: Only apply to GetElementPtr instruction
    /// Sets the given GEP to be in bounds or not.
    pub fn set_in_bounds_flag(self, flag: bool) -> Result<(), InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::GetElementPtr => {
                unsafe {
                    llvm_sys::core::LLVMSetIsInBounds(self.as_value_ref(), flag as i32);
                }
                Ok(())
            },
            _ => Err(InstructionValueError::NotGEPInst),
        }
    }

    // SubTypes: Only apply to memory access and alloca instructions
    /// Returns alignment on a memory access instruction or alloca.
    pub fn get_alignment(self) -> Result<u32, InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::Alloca | InstructionOpcode::Load | InstructionOpcode::Store => {
                Ok(unsafe { LLVMGetAlignment(self.as_value_ref()) })
            },
            _ => Err(InstructionValueError::AlignmentError(
                AlignmentError::UnalignedInstruction,
            )),
        }
    }

    // SubTypes: Only apply to memory access and alloca instructions
    /// Sets alignment on a memory access instruction or alloca.
    pub fn set_alignment(self, alignment: u32) -> Result<(), InstructionValueError> {
        // Zero check is unnecessary as 0 is not a power of two.
        if !alignment.is_power_of_two() {
            return Err(InstructionValueError::AlignmentError(AlignmentError::NonPowerOfTwo(
                alignment,
            )));
        }

        match self.get_opcode() {
            InstructionOpcode::Alloca | InstructionOpcode::Load | InstructionOpcode::Store => {
                unsafe { LLVMSetAlignment(self.as_value_ref(), alignment) };
                Ok(())
            },
            _ => Err(InstructionValueError::AlignmentError(
                AlignmentError::UnalignedInstruction,
            )),
        }
    }

    // SubTypes: Only apply to memory access instructions
    /// Returns atomic ordering on a memory access instruction.
    pub fn get_atomic_ordering(self) -> Result<AtomicOrdering, InstructionValueError> {
        match self.get_opcode() {
            InstructionOpcode::Load | InstructionOpcode::Store | InstructionOpcode::AtomicRMW => {
                Ok(unsafe { LLVMGetOrdering(self.as_value_ref()) }.into())
            },
            #[cfg(any(
                feature = "llvm18-1",
                feature = "llvm19-1",
                feature = "llvm20-1",
                feature = "llvm21-1"
            ))]
            InstructionOpcode::Fence => Ok(unsafe { LLVMGetOrdering(self.as_value_ref()) }.into()),
            _ => Err(InstructionValueError::NotAtomicOrderingInst),
        }
    }

    // SubTypes: Only apply to memory access instructions
    /// Sets atomic ordering on a memory access instruction.
    pub fn set_atomic_ordering(self, ordering: AtomicOrdering) -> Result<(), InstructionValueError> {
        // Although fence and atomicrmw both have an ordering, the LLVM C API
        // does not support them (for LLVM < 18). The cmpxchg instruction has two orderings and
        // does not work with this API
        match (self.get_opcode(), ordering) {
            (InstructionOpcode::Load, AtomicOrdering::Release) => {
                Err(InstructionValueError::AtomicError(AtomicError::ReleaseOnLoad))
            },
            (InstructionOpcode::Store, AtomicOrdering::Acquire) => {
                Err(InstructionValueError::AtomicError(AtomicError::AcquireOnStore))
            },
            (InstructionOpcode::Load | InstructionOpcode::Store, AtomicOrdering::AcquireRelease) => {
                Err(InstructionValueError::AtomicError(AtomicError::AcquireRelease))
            },
            (InstructionOpcode::Load | InstructionOpcode::Store, _) => {
                unsafe { LLVMSetOrdering(self.as_value_ref(), ordering.into()) };
                Ok(())
            },
            #[cfg(any(
                feature = "llvm18-1",
                feature = "llvm19-1",
                feature = "llvm20-1",
                feature = "llvm21-1"
            ))]
            (
                InstructionOpcode::Fence,
                AtomicOrdering::Acquire
                | AtomicOrdering::Release
                | AtomicOrdering::AcquireRelease
                | AtomicOrdering::SequentiallyConsistent,
            ) => {
                unsafe { LLVMSetOrdering(self.as_value_ref(), ordering.into()) };
                Ok(())
            },
            #[cfg(any(
                feature = "llvm18-1",
                feature = "llvm19-1",
                feature = "llvm20-1",
                feature = "llvm21-1"
            ))]
            (InstructionOpcode::Fence, _) => {
                Err(InstructionValueError::AtomicError(AtomicError::InvalidOrderingOnFence))
            },
            #[cfg(any(
                feature = "llvm18-1",
                feature = "llvm19-1",
                feature = "llvm20-1",
                feature = "llvm21-1"
            ))]
            (InstructionOpcode::AtomicRMW, AtomicOrdering::NotAtomic | AtomicOrdering::Unordered) => Err(
                InstructionValueError::AtomicError(AtomicError::InvalidOrderingOnAtomicRMW),
            ),
            #[cfg(any(
                feature = "llvm18-1",
                feature = "llvm19-1",
                feature = "llvm20-1",
                feature = "llvm21-1"
            ))]
            (InstructionOpcode::AtomicRMW, _) => {
                unsafe { LLVMSetOrdering(self.as_value_ref(), ordering.into()) };
                Ok(())
            },
            (_, _) => Err(InstructionValueError::NotAtomicOrderingInst),
        }
    }

    /// Obtains the number of operands an `InstructionValue` has.
    /// An operand is a `BasicValue` used in an IR instruction.
    ///
    /// The following example,
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("ivs");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let f32_type = context.f32_type();
    /// #[cfg(feature = "typed-pointers")]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(not(feature = "typed-pointers"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val).unwrap();
    /// let free_instruction = builder.build_free(arg1).unwrap();
    /// let return_instruction = builder.build_return(None).unwrap();
    ///
    /// assert_eq!(store_instruction.get_num_operands(), 2);
    /// assert_eq!(free_instruction.get_num_operands(), 2);
    /// assert_eq!(return_instruction.get_num_operands(), 0);
    /// ```
    ///
    /// will generate LLVM IR roughly like (varying slightly across LLVM versions):
    ///
    /// ```ir
    /// ; ModuleID = 'ivs'
    /// source_filename = "ivs"
    ///
    /// define void @take_f32_ptr(float* %0) {
    /// entry:
    ///   store float 0x400921FB60000000, float* %0
    ///   %1 = bitcast float* %0 to i8*
    ///   tail call void @free(i8* %1)
    ///   ret void
    /// }
    ///
    /// declare void @free(i8*)
    /// ```
    ///
    /// which makes the number of instruction operands clear:
    /// 1) Store has two: a const float and a variable float pointer %0
    /// 2) Bitcast has one: a variable float pointer %0
    /// 3) Function call has two: i8 pointer %1 argument, and the free function itself
    /// 4) Void return has zero: void is not a value and does not count as an operand
    ///    even though the return instruction can take values.
    pub fn get_num_operands(self) -> u32 {
        unsafe { LLVMGetNumOperands(self.as_value_ref()) as u32 }
    }

    /// Obtains the operand an `InstructionValue` has at a given index if any.
    /// An operand is a `BasicValue` used in an IR instruction.
    ///
    /// The following example,
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("ivs");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let f32_type = context.f32_type();
    /// #[cfg(feature = "typed-pointers")]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(not(feature = "typed-pointers"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val).unwrap();
    /// let free_instruction = builder.build_free(arg1).unwrap();
    /// let return_instruction = builder.build_return(None).unwrap();
    ///
    /// assert!(store_instruction.get_operand(0).is_some());
    /// assert!(store_instruction.get_operand(1).is_some());
    /// assert!(store_instruction.get_operand(2).is_none());
    /// assert!(free_instruction.get_operand(0).is_some());
    /// assert!(free_instruction.get_operand(1).is_some());
    /// assert!(free_instruction.get_operand(2).is_none());
    /// assert!(return_instruction.get_operand(0).is_none());
    /// assert!(return_instruction.get_operand(1).is_none());
    /// ```
    ///
    /// will generate LLVM IR roughly like (varying slightly across LLVM versions):
    ///
    /// ```ir
    /// ; ModuleID = 'ivs'
    /// source_filename = "ivs"
    ///
    /// define void @take_f32_ptr(float* %0) {
    /// entry:
    ///   store float 0x400921FB60000000, float* %0
    ///   %1 = bitcast float* %0 to i8*
    ///   tail call void @free(i8* %1)
    ///   ret void
    /// }
    ///
    /// declare void @free(i8*)
    /// ```
    ///
    /// which makes the instruction operands clear:
    /// 1) Store has two: a const float and a variable float pointer %0
    /// 2) Bitcast has one: a variable float pointer %0
    /// 3) Function call has two: i8 pointer %1 argument, and the free function itself
    /// 4) Void return has zero: void is not a value and does not count as an operand
    ///    even though the return instruction can take values.
    pub fn get_operand(self, index: u32) -> Option<Operand<'ctx>> {
        let num_operands = self.get_num_operands();

        if index >= num_operands {
            return None;
        }

        unsafe { self.get_operand_unchecked(index) }
    }

    /// Get the operand of an `InstructionValue`.
    ///
    /// # Safety
    ///
    /// The index must be less than [InstructionValue::get_num_operands].
    pub unsafe fn get_operand_unchecked(self, index: u32) -> Option<Operand<'ctx>> {
        let operand = unsafe { LLVMGetOperand(self.as_value_ref(), index) };

        if operand.is_null() {
            return None;
        }

        let is_basic_block = unsafe { LLVMValueIsBasicBlock(operand) == 1 };

        if is_basic_block {
            let bb = unsafe { BasicBlock::new(LLVMValueAsBasicBlock(operand)) };

            Some(Operand::Block(bb.expect("BasicBlock should always be valid")))
        } else {
            Some(Operand::Value(unsafe { BasicValueEnum::new(operand) }))
        }
    }

    /// Get an instruction value operand iterator.
    pub fn get_operands(self) -> OperandIter<'ctx> {
        OperandIter {
            iv: self,
            i: 0,
            count: self.get_num_operands(),
        }
    }

    /// Sets the operand an `InstructionValue` has at a given index if possible.
    /// An operand is a `BasicValue` used in an IR instruction.
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("ivs");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let f32_type = context.f32_type();
    /// #[cfg(feature = "typed-pointers")]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(not(feature = "typed-pointers"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val).unwrap();
    /// let free_instruction = builder.build_free(arg1).unwrap();
    /// let return_instruction = builder.build_return(None).unwrap();
    ///
    /// // This will produce invalid IR:
    /// free_instruction.set_operand(0, f32_val);
    ///
    /// assert_eq!(free_instruction.get_operand(0).unwrap().unwrap_value(), f32_val);
    /// ```
    pub fn set_operand<BV: BasicValue<'ctx>>(self, index: u32, val: BV) -> bool {
        let num_operands = self.get_num_operands();

        if index >= num_operands {
            return false;
        }

        unsafe { LLVMSetOperand(self.as_value_ref(), index, val.as_value_ref()) }

        true
    }

    /// Gets the use of an operand(`BasicValue`), if any.
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    /// use inkwell::values::BasicValue;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("ivs");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let f32_type = context.f32_type();
    /// #[cfg(feature = "typed-pointers")]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(not(feature = "typed-pointers"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val).unwrap();
    /// let free_instruction = builder.build_free(arg1).unwrap();
    /// let return_instruction = builder.build_return(None).unwrap();
    ///
    /// assert_eq!(store_instruction.get_operand_use(1), arg1.get_first_use());
    /// ```
    pub fn get_operand_use(self, index: u32) -> Option<BasicValueUse<'ctx>> {
        let num_operands = self.get_num_operands();

        if index >= num_operands {
            return None;
        }

        unsafe { self.get_operand_use_unchecked(index) }
    }

    /// Gets the use of an operand(`BasicValue`), if any.
    ///
    /// # Safety
    ///
    /// The index must be smaller than [InstructionValue::get_num_operands].
    pub unsafe fn get_operand_use_unchecked(self, index: u32) -> Option<BasicValueUse<'ctx>> {
        let use_ = unsafe { LLVMGetOperandUse(self.as_value_ref(), index) };

        if use_.is_null() {
            return None;
        }

        unsafe { Some(BasicValueUse::new(use_)) }
    }

    /// Get an instruction value operand use iterator.
    pub fn get_operand_uses(self) -> OperandUseIter<'ctx> {
        OperandUseIter {
            iv: self,
            i: 0,
            count: self.get_num_operands(),
        }
    }

    /// Obtains the number of indices an `InstructionValue` has.
    /// An index is used in `ExtractValue` and `InsertValue` instructions to specify
    /// which field or element to access in an aggregate type (struct or array).
    ///
    /// Returns 0 for instructions that are not `ExtractValue` or `InsertValue`.
    ///
    /// The following example,
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::values::BasicValue;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("ivs");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let i32_type = context.i32_type();
    /// let struct_type = context.struct_type(&[i32_type.into(), i32_type.into()], false);
    /// let fn_type = void_type.fn_type(&[], false);
    ///
    /// let function = module.add_function("test", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    ///
    /// let struct_val = struct_type.get_undef();
    /// let extract_instruction = builder.build_extract_value(struct_val, 0, "extract").unwrap()
    ///     .as_instruction_value().unwrap();
    ///
    /// assert_eq!(extract_instruction.get_num_indices(), 1);
    /// ```
    pub fn get_num_indices(self) -> u32 {
        let opcode = self.get_opcode();
        if opcode != InstructionOpcode::ExtractValue && opcode != InstructionOpcode::InsertValue {
            return 0;
        }
        unsafe { LLVMGetNumIndices(self.as_value_ref()) }
    }

    /// Obtains the indices an `InstructionValue` has as a vector.
    /// An index is used in `ExtractValue` and `InsertValue` instructions to specify
    /// which field or element to access in an aggregate type (struct or array).
    ///
    /// Returns an empty vector for instructions that are not `ExtractValue` or `InsertValue`.
    ///
    /// The following example,
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::values::BasicValue;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("ivs");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let i32_type = context.i32_type();
    /// let struct_type = context.struct_type(&[i32_type.into(), i32_type.into()], false);
    /// let fn_type = void_type.fn_type(&[], false);
    ///
    /// let function = module.add_function("test", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    ///
    /// let struct_val = struct_type.get_undef();
    /// let extract_instruction = builder.build_extract_value(struct_val, 0, "extract").unwrap()
    ///     .as_instruction_value().unwrap();
    ///
    /// assert_eq!(extract_instruction.get_indices(), vec![0]);
    /// ```
    pub fn get_indices(self) -> Vec<u32> {
        let num_indices = self.get_num_indices();
        if num_indices == 0 {
            return Vec::new();
        }

        unsafe {
            let indices_ptr = LLVMGetIndices(self.as_value_ref());
            std::slice::from_raw_parts(indices_ptr, num_indices as usize).to_vec()
        }
    }

    /// Gets the first use of an `InstructionValue` if any.
    ///
    /// The following example,
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    /// use inkwell::values::BasicValue;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("ivs");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let f32_type = context.f32_type();
    /// #[cfg(feature = "typed-pointers")]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(not(feature = "typed-pointers"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val).unwrap();
    /// let free_instruction = builder.build_free(arg1).unwrap();
    /// let return_instruction = builder.build_return(None).unwrap();
    ///
    /// assert!(arg1.get_first_use().is_some());
    /// ```
    pub fn get_first_use(self) -> Option<BasicValueUse<'ctx>> {
        self.instruction_value.get_first_use()
    }

    /// Gets the predicate of an `ICmp` `InstructionValue`.
    /// For instance, in the LLVM instruction
    /// `%3 = icmp slt i32 %0, %1`
    /// this gives the `slt`.
    ///
    /// If the instruction is not an `ICmp`, this returns None.
    pub fn get_icmp_predicate(self) -> Option<IntPredicate> {
        // REVIEW: this call to get_opcode() can be inefficient;
        // what happens if we don't perform this check, and just call
        // LLVMGetICmpPredicate() regardless?
        if self.get_opcode() == InstructionOpcode::ICmp {
            let pred = unsafe { LLVMGetICmpPredicate(self.as_value_ref()) };
            Some(IntPredicate::new(pred))
        } else {
            None
        }
    }

    /// Gets the predicate of an `FCmp` `InstructionValue`.
    /// For instance, in the LLVM instruction
    /// `%3 = fcmp olt float %0, %1`
    /// this gives the `olt`.
    ///
    /// If the instruction is not an `FCmp`, this returns None.
    pub fn get_fcmp_predicate(self) -> Option<FloatPredicate> {
        // REVIEW: this call to get_opcode() can be inefficient;
        // what happens if we don't perform this check, and just call
        // LLVMGetFCmpPredicate() regardless?
        if self.get_opcode() == InstructionOpcode::FCmp {
            let pred = unsafe { LLVMGetFCmpPredicate(self.as_value_ref()) };
            Some(FloatPredicate::new(pred))
        } else {
            None
        }
    }

    /// Gets the binary operation of an `AtomicRMW` `InstructionValue`.
    /// For instance, in the LLVM instruction
    /// `%3 = atomicrmw add i32* %ptr, i32 %val monotonic`
    /// this gives the `add`.
    ///
    /// If the instruction is not an `AtomicRMW`, this returns None.
    pub fn get_atomic_rmw_bin_op(self) -> Option<AtomicRMWBinOp> {
        if self.get_opcode() == InstructionOpcode::AtomicRMW {
            let bin_op = unsafe { LLVMGetAtomicRMWBinOp(self.as_value_ref()) };
            Some(AtomicRMWBinOp::new(bin_op))
        } else {
            None
        }
    }

    /// Determines whether or not this `Instruction` has any associated metadata.
    pub fn has_metadata(self) -> bool {
        unsafe { LLVMHasMetadata(self.instruction_value.value) == 1 }
    }

    /// Gets the `MetadataValue` associated with this `Instruction` at a specific
    /// `kind_id`.
    pub fn get_metadata(self, kind_id: u32) -> Option<MetadataValue<'ctx>> {
        let metadata_value = unsafe { LLVMGetMetadata(self.instruction_value.value, kind_id) };

        if metadata_value.is_null() {
            return None;
        }

        unsafe { Some(MetadataValue::new(metadata_value)) }
    }

    /// Determines whether or not this `Instruction` has any associated metadata
    /// `kind_id`.
    pub fn set_metadata(self, metadata: MetadataValue<'ctx>, kind_id: u32) -> Result<(), InstructionValueError> {
        if !metadata.is_node() {
            return Err(InstructionValueError::ExpectedNode);
        }

        unsafe {
            LLVMSetMetadata(self.instruction_value.value, kind_id, metadata.as_value_ref());
        }

        Ok(())
    }

    /// Get the debug location for this instruction.
    pub fn get_debug_location(self) -> Option<DILocation<'ctx>> {
        // https://github.com/llvm/llvm-project/blob/e83cc896e7c2378914a391f942c188d454b517d2/llvm/include/llvm/IR/Instruction.h#L513
        let metadata_ref = unsafe { llvm_sys::debuginfo::LLVMInstructionGetDebugLoc(self.as_value_ref()) };
        if metadata_ref.is_null() {
            None
        } else {
            Some(DILocation {
                metadata_ref,
                _marker: std::marker::PhantomData,
            })
        }
    }

    /// Set the debug location for this instruction.
    pub fn set_debug_location(self, location: Option<DILocation<'_>>) {
        // https://github.com/llvm/llvm-project/blob/e83cc896e7c2378914a391f942c188d454b517d2/llvm/include/llvm/IR/Instruction.h#L510
        let metadata_ref = location.map_or(std::ptr::null_mut(), |loc| loc.metadata_ref);
        unsafe {
            llvm_sys::debuginfo::LLVMInstructionSetDebugLoc(self.as_value_ref(), metadata_ref);
        }
    }
}

unsafe impl AsValueRef for InstructionValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.instruction_value.value
    }
}

impl Display for InstructionValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

/// Iterate over all the operands of an instruction value.
#[derive(Debug)]
pub struct OperandIter<'ctx> {
    iv: InstructionValue<'ctx>,
    i: u32,
    count: u32,
}

impl<'ctx> Iterator for OperandIter<'ctx> {
    type Item = Option<Operand<'ctx>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.count {
            let result = unsafe { self.iv.get_operand_unchecked(self.i) };
            self.i += 1;
            Some(result)
        } else {
            None
        }
    }
}

/// Iterate over all the operands of an instruction value.
#[derive(Debug)]
pub struct OperandUseIter<'ctx> {
    iv: InstructionValue<'ctx>,
    i: u32,
    count: u32,
}

impl<'ctx> Iterator for OperandUseIter<'ctx> {
    type Item = Option<BasicValueUse<'ctx>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.count {
            let result = unsafe { self.iv.get_operand_use_unchecked(self.i) };
            self.i += 1;
            Some(result)
        } else {
            None
        }
    }
}

#[llvm_versions(18..)]
bitflags::bitflags! {
    /// Fast math flags to enable otherwise unsafe floating-point transformations.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FastMathFlags: u32 {
        /// Allows all non-strict floating-point transforms.
        const AllowReassoc = llvm_sys::LLVMFastMathAllowReassoc;
        /// Arguments and results are assumed not-NaN.
        const NoNaNs = llvm_sys::LLVMFastMathNoNaNs;
        /// Arguments and results are assumed not-infinite.
        const NoInfs = llvm_sys::LLVMFastMathNoInfs;
        /// Can ignore the sign of zero.
        const NoSignedZeros = llvm_sys::LLVMFastMathNoSignedZeros;
        /// Can use reciprocal multiply instead of division.
        const AllowReciprocal = llvm_sys::LLVMFastMathAllowReciprocal;
        /// Can be floating-point contracted (FMA).
        const AllowContract = llvm_sys::LLVMFastMathAllowContract;
        /// Allows approximations of math library functions or intrinsics
        const ApproxFunc = llvm_sys::LLVMFastMathApproxFunc;
    }
}
