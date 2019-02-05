use llvm_sys::core::{LLVMGetInstructionOpcode, LLVMIsTailCall, LLVMGetPreviousInstruction, LLVMGetNextInstruction, LLVMGetInstructionParent, LLVMInstructionEraseFromParent, LLVMInstructionClone, LLVMSetVolatile, LLVMGetVolatile, LLVMGetNumOperands, LLVMGetOperand, LLVMGetOperandUse, LLVMSetOperand};
#[llvm_versions(3.9 => latest)]
use llvm_sys::core::LLVMInstructionRemoveFromParent;
use llvm_sys::LLVMOpcode;
use llvm_sys::prelude::LLVMValueRef;

use basic_block::BasicBlock;
use value_use::ValueUse;
use values::traits::AsValueRef;
use values::{BasicValue, BasicValueEnum, Value};

// REVIEW: Split up into structs for SubTypes on InstructionValues?
// REVIEW: This should maybe be split up into InstructionOpcode and ConstOpcode?
// see LLVMGetConstOpcode
#[derive(Debug, PartialEq, Eq)]
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
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
    CatchPad,
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
    CatchRet,
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
    CatchSwitch,
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
    CleanupPad,
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
    CleanupRet,
    ExtractElement,
    ExtractValue,
    FAdd,
    FCmp,
    FDiv,
    Fence,
    FMul,
    FPExt,
    FPToSI,
    FPToUI,
    FPTrunc,
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
    Phi,
    PtrToInt,
    Resume,
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

impl InstructionOpcode {
    fn new(opcode: LLVMOpcode) -> Self {
        match opcode {
            LLVMOpcode::LLVMAdd => InstructionOpcode::Add,
            LLVMOpcode::LLVMAddrSpaceCast => InstructionOpcode::AddrSpaceCast,
            LLVMOpcode::LLVMAlloca => InstructionOpcode::Alloca,
            LLVMOpcode::LLVMAnd => InstructionOpcode::And,
            LLVMOpcode::LLVMAShr => InstructionOpcode::AShr,
            LLVMOpcode::LLVMAtomicCmpXchg => InstructionOpcode::AtomicCmpXchg,
            LLVMOpcode::LLVMAtomicRMW => InstructionOpcode::AtomicRMW,
            LLVMOpcode::LLVMBitCast => InstructionOpcode::BitCast,
            LLVMOpcode::LLVMBr => InstructionOpcode::Br,
            LLVMOpcode::LLVMCall => InstructionOpcode::Call,
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            LLVMOpcode::LLVMCatchPad => InstructionOpcode::CatchPad,
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            LLVMOpcode::LLVMCatchRet => InstructionOpcode::CatchRet,
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            LLVMOpcode::LLVMCatchSwitch => InstructionOpcode::CatchSwitch,
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            LLVMOpcode::LLVMCleanupPad => InstructionOpcode::CleanupPad,
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            LLVMOpcode::LLVMCleanupRet => InstructionOpcode::CleanupRet,
            LLVMOpcode::LLVMExtractElement => InstructionOpcode::ExtractElement,
            LLVMOpcode::LLVMExtractValue => InstructionOpcode::ExtractValue,
            LLVMOpcode::LLVMFAdd => InstructionOpcode::FAdd,
            LLVMOpcode::LLVMFCmp => InstructionOpcode::FCmp,
            LLVMOpcode::LLVMFDiv => InstructionOpcode::FDiv,
            LLVMOpcode::LLVMFence => InstructionOpcode::Fence,
            LLVMOpcode::LLVMFMul => InstructionOpcode::FMul,
            LLVMOpcode::LLVMFPExt => InstructionOpcode::FPExt,
            LLVMOpcode::LLVMFPToSI => InstructionOpcode::FPToSI,
            LLVMOpcode::LLVMFPToUI => InstructionOpcode::FPToUI,
            LLVMOpcode::LLVMFPTrunc => InstructionOpcode::FPTrunc,
            LLVMOpcode::LLVMFRem => InstructionOpcode::FRem,
            LLVMOpcode::LLVMFSub => InstructionOpcode::FSub,
            LLVMOpcode::LLVMGetElementPtr => InstructionOpcode::GetElementPtr,
            LLVMOpcode::LLVMICmp => InstructionOpcode::ICmp,
            LLVMOpcode::LLVMIndirectBr => InstructionOpcode::IndirectBr,
            LLVMOpcode::LLVMInsertElement => InstructionOpcode::InsertElement,
            LLVMOpcode::LLVMInsertValue => InstructionOpcode::InsertValue,
            LLVMOpcode::LLVMIntToPtr => InstructionOpcode::IntToPtr,
            LLVMOpcode::LLVMInvoke => InstructionOpcode::Invoke,
            LLVMOpcode::LLVMLandingPad => InstructionOpcode::LandingPad,
            LLVMOpcode::LLVMLoad => InstructionOpcode::Load,
            LLVMOpcode::LLVMLShr => InstructionOpcode::LShr,
            LLVMOpcode::LLVMMul => InstructionOpcode::Mul,
            LLVMOpcode::LLVMOr => InstructionOpcode::Or,
            LLVMOpcode::LLVMPHI => InstructionOpcode::Phi,
            LLVMOpcode::LLVMPtrToInt => InstructionOpcode::PtrToInt,
            LLVMOpcode::LLVMResume => InstructionOpcode::Resume,
            LLVMOpcode::LLVMRet => InstructionOpcode::Return,
            LLVMOpcode::LLVMSDiv => InstructionOpcode::SDiv,
            LLVMOpcode::LLVMSelect => InstructionOpcode::Select,
            LLVMOpcode::LLVMSExt => InstructionOpcode::SExt,
            LLVMOpcode::LLVMShl => InstructionOpcode::Shl,
            LLVMOpcode::LLVMShuffleVector => InstructionOpcode::ShuffleVector,
            LLVMOpcode::LLVMSIToFP => InstructionOpcode::SIToFP,
            LLVMOpcode::LLVMSRem => InstructionOpcode::SRem,
            LLVMOpcode::LLVMStore => InstructionOpcode::Store,
            LLVMOpcode::LLVMSub => InstructionOpcode::Sub,
            LLVMOpcode::LLVMSwitch => InstructionOpcode::Switch,
            LLVMOpcode::LLVMTrunc => InstructionOpcode::Trunc,
            LLVMOpcode::LLVMUDiv => InstructionOpcode::UDiv,
            LLVMOpcode::LLVMUIToFP => InstructionOpcode::UIToFP,
            LLVMOpcode::LLVMUnreachable => InstructionOpcode::Unreachable,
            LLVMOpcode::LLVMURem => InstructionOpcode::URem,
            LLVMOpcode::LLVMUserOp1 => InstructionOpcode::UserOp1,
            LLVMOpcode::LLVMUserOp2 => InstructionOpcode::UserOp2,
            LLVMOpcode::LLVMVAArg => InstructionOpcode::VAArg,
            LLVMOpcode::LLVMXor => InstructionOpcode::Xor,
            LLVMOpcode::LLVMZExt => InstructionOpcode::ZExt,
        }
    }

    pub(crate) fn as_llvm_opcode(&self) -> LLVMOpcode {
        match *self {
            InstructionOpcode::Add => LLVMOpcode::LLVMAdd,
            InstructionOpcode::AddrSpaceCast => LLVMOpcode::LLVMAddrSpaceCast,
            InstructionOpcode::Alloca => LLVMOpcode::LLVMAlloca,
            InstructionOpcode::And => LLVMOpcode::LLVMAnd,
            InstructionOpcode::AShr => LLVMOpcode::LLVMAShr,
            InstructionOpcode::AtomicCmpXchg => LLVMOpcode::LLVMAtomicCmpXchg,
            InstructionOpcode::AtomicRMW => LLVMOpcode::LLVMAtomicRMW,
            InstructionOpcode::BitCast => LLVMOpcode::LLVMBitCast,
            InstructionOpcode::Br => LLVMOpcode::LLVMBr,
            InstructionOpcode::Call => LLVMOpcode::LLVMCall,
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            InstructionOpcode::CatchPad => LLVMOpcode::LLVMCatchPad,
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            InstructionOpcode::CatchRet => LLVMOpcode::LLVMCatchRet,
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            InstructionOpcode::CatchSwitch => LLVMOpcode::LLVMCatchSwitch,
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            InstructionOpcode::CleanupPad => LLVMOpcode::LLVMCleanupPad,
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            InstructionOpcode::CleanupRet => LLVMOpcode::LLVMCleanupRet,
            InstructionOpcode::ExtractElement => LLVMOpcode::LLVMExtractElement,
            InstructionOpcode::ExtractValue => LLVMOpcode::LLVMExtractValue,
            InstructionOpcode::FAdd => LLVMOpcode::LLVMFAdd,
            InstructionOpcode::FCmp => LLVMOpcode::LLVMFCmp,
            InstructionOpcode::FDiv => LLVMOpcode::LLVMFDiv,
            InstructionOpcode::Fence => LLVMOpcode::LLVMFence,
            InstructionOpcode::FMul => LLVMOpcode::LLVMFMul,
            InstructionOpcode::FPExt => LLVMOpcode::LLVMFPExt,
            InstructionOpcode::FPToSI => LLVMOpcode::LLVMFPToSI,
            InstructionOpcode::FPToUI => LLVMOpcode::LLVMFPToUI,
            InstructionOpcode::FPTrunc => LLVMOpcode::LLVMFPTrunc,
            InstructionOpcode::FRem => LLVMOpcode::LLVMFRem,
            InstructionOpcode::FSub => LLVMOpcode::LLVMFSub,
            InstructionOpcode::GetElementPtr => LLVMOpcode::LLVMGetElementPtr,
            InstructionOpcode::ICmp => LLVMOpcode::LLVMICmp,
            InstructionOpcode::IndirectBr => LLVMOpcode::LLVMIndirectBr,
            InstructionOpcode::InsertElement => LLVMOpcode::LLVMInsertElement,
            InstructionOpcode::InsertValue => LLVMOpcode::LLVMInsertValue,
            InstructionOpcode::IntToPtr => LLVMOpcode::LLVMIntToPtr,
            InstructionOpcode::Invoke => LLVMOpcode::LLVMInvoke,
            InstructionOpcode::LandingPad => LLVMOpcode::LLVMLandingPad,
            InstructionOpcode::Load => LLVMOpcode::LLVMLoad,
            InstructionOpcode::LShr => LLVMOpcode::LLVMLShr,
            InstructionOpcode::Mul => LLVMOpcode::LLVMMul,
            InstructionOpcode::Or => LLVMOpcode::LLVMOr,
            InstructionOpcode::Phi => LLVMOpcode::LLVMPHI,
            InstructionOpcode::PtrToInt => LLVMOpcode::LLVMPtrToInt,
            InstructionOpcode::Resume => LLVMOpcode::LLVMResume,
            InstructionOpcode::Return => LLVMOpcode::LLVMRet,
            InstructionOpcode::SDiv => LLVMOpcode::LLVMSDiv,
            InstructionOpcode::Select => LLVMOpcode::LLVMSelect,
            InstructionOpcode::SExt => LLVMOpcode::LLVMSExt,
            InstructionOpcode::Shl => LLVMOpcode::LLVMShl,
            InstructionOpcode::ShuffleVector => LLVMOpcode::LLVMShuffleVector,
            InstructionOpcode::SIToFP => LLVMOpcode::LLVMSIToFP,
            InstructionOpcode::SRem => LLVMOpcode::LLVMSRem,
            InstructionOpcode::Store => LLVMOpcode::LLVMStore,
            InstructionOpcode::Sub => LLVMOpcode::LLVMSub,
            InstructionOpcode::Switch => LLVMOpcode::LLVMSwitch,
            InstructionOpcode::Trunc => LLVMOpcode::LLVMTrunc,
            InstructionOpcode::UDiv => LLVMOpcode::LLVMUDiv,
            InstructionOpcode::UIToFP => LLVMOpcode::LLVMUIToFP,
            InstructionOpcode::Unreachable => LLVMOpcode::LLVMUnreachable,
            InstructionOpcode::URem => LLVMOpcode::LLVMURem,
            InstructionOpcode::UserOp1 => LLVMOpcode::LLVMUserOp1,
            InstructionOpcode::UserOp2 => LLVMOpcode::LLVMUserOp2,
            InstructionOpcode::VAArg => LLVMOpcode::LLVMVAArg,
            InstructionOpcode::Xor => LLVMOpcode::LLVMXor,
            InstructionOpcode::ZExt => LLVMOpcode::LLVMZExt,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy)]
pub struct InstructionValue {
    instruction_value: Value,
}

impl InstructionValue {
    pub(crate) fn new(instruction_value: LLVMValueRef) -> Self {
        debug_assert!(!instruction_value.is_null());

        let value = Value::new(instruction_value);

        debug_assert!(value.is_instruction());

        InstructionValue {
            instruction_value: value,
        }
    }

    pub fn get_opcode(&self) -> InstructionOpcode {
        let opcode = unsafe {
            LLVMGetInstructionOpcode(self.as_value_ref())
        };

        InstructionOpcode::new(opcode)
    }

    pub fn get_previous_instruction(&self) -> Option<Self> {
        let value = unsafe {
            LLVMGetPreviousInstruction(self.as_value_ref())
        };

        if value.is_null() {
            return None;
        }

        Some(InstructionValue::new(value))
    }

    pub fn get_next_instruction(&self) -> Option<Self> {
        let value = unsafe {
            LLVMGetNextInstruction(self.as_value_ref())
        };

        if value.is_null() {
            return None;
        }

        Some(InstructionValue::new(value))
    }

    // REVIEW: Potentially unsafe if parent BB or grandparent fn were removed?
    pub fn erase_from_basic_block(&self) {
        unsafe {
            LLVMInstructionEraseFromParent(self.as_value_ref())
        }
    }

    // REVIEW: Potentially unsafe if parent BB or grandparent fn were removed?
    #[llvm_versions(3.9 => latest)]
    pub fn remove_from_basic_block(&self) {
        unsafe {
            LLVMInstructionRemoveFromParent(self.as_value_ref())
        }
    }

    // REVIEW: Potentially unsafe is parent BB or grandparent fn was deleted
    // REVIEW: Should this *not* be an option? Parent should always exist,
    // but I doubt LLVM returns null if the parent BB (or grandparent FN)
    // was deleted... Invalid memory is more likely. Cloned IV will have no
    // parent?
    pub fn get_parent(&self) -> Option<BasicBlock> {
        let value = unsafe {
            LLVMGetInstructionParent(self.as_value_ref())
        };

        BasicBlock::new(value)
    }

    // REVIEW: See if necessary to check opcode == Call first.
    // Does it always return false otherwise?
    pub fn is_tail_call(&self) -> bool {
        unsafe {
            LLVMIsTailCall(self.as_value_ref()) == 1
        }
    }

    pub fn replace_all_uses_with(&self, other: &InstructionValue) {
        self.instruction_value.replace_all_uses_with(other.as_value_ref())
    }

    // SubTypes: Only apply to memory access instructions
    /// Returns whether or not a memory access instruction is volatile.
    pub fn get_volatile(&self) -> bool {
        unsafe {
            LLVMGetVolatile(self.as_value_ref()) == 1
        }
    }

    // SubTypes: Only apply to memory access instructions
    /// Sets whether or not a memory access instruction is volatile.
    pub fn set_volatile(&self, volatile: bool) {
        unsafe {
            LLVMSetVolatile(self.as_value_ref(), volatile as i32)
        }
    }

    pub fn get_num_operands(&self) -> u32 {
        unsafe {
            LLVMGetNumOperands(self.as_value_ref()) as u32
        }
    }

    pub fn get_operand(&self, index: u32) -> Option<BasicValueEnum> {
        let num_operands = self.get_num_operands();

        if index >= num_operands {
            return None;
        }

        let operand = unsafe {
            LLVMGetOperand(self.as_value_ref(), index)
        };

        if operand.is_null() {
            return None;
        }

        Some(BasicValueEnum::new(operand))
    }

    pub fn set_operand<BV: BasicValue>(&self, index: u32, val: BV) -> bool {
        let num_operands = self.get_num_operands();

        if index >= num_operands {
            return false;
        }

        unsafe {
            LLVMSetOperand(self.as_value_ref(), index, val.as_value_ref())
        }

        true
    }

    pub fn get_operand_use(&self, index: u32) -> Option<ValueUse> {
        let num_operands = self.get_num_operands();

        if index >= num_operands {
            return None;
        }

        let use_ = unsafe {
            LLVMGetOperandUse(self.as_value_ref(), index)
        };

        if use_.is_null() {
            return None;
        }

        Some(ValueUse::new(use_))
    }

    pub fn get_first_use(&self) -> Option<ValueUse> {
        self.instruction_value.get_first_use()
    }
}

impl Clone for InstructionValue {
    /// Creates a clone of this `InstructionValue`, and returns it.
    /// The clone will have no parent, and no name.
    fn clone(&self) -> Self {
        let value = unsafe {
            LLVMInstructionClone(self.as_value_ref())
        };

        InstructionValue::new(value)
    }
}

impl AsValueRef for InstructionValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.instruction_value.value
    }
}
