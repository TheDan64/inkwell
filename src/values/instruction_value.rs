use llvm_sys::core::{LLVMGetInstructionOpcode, LLVMIsTailCall};
use llvm_sys::LLVMOpcode;
use llvm_sys::prelude::LLVMValueRef;

use values::traits::AsValueRef;
use values::Value;

// REVIEW: Metadata on instructions?
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
    // Later versions:
    // CatchPad,
    // CatchRet,
    // CatchSwitch,
    // CleanupPad,
    // CleanupRet,
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
    PHI,
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
            // Newer versions:
            // LLVMOpcode::LLVMCatchPad => InstructionOpcode::CatchPad,
            // LLVMOpcode::LLVMCatchRet => InstructionOpcode::CatchRet,
            // LLVMOpcode::LLVMCatchSwitch => InstructionOpcode::CatchSwitch,
            // LLVMOpcode::LLVMCleanupPad => InstructionOpcode::CleanupPad,
            // LLVMOpcode::LLVMCleanupRet => InstructionOpcode::CleanupRet,
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
            LLVMOpcode::LLVMPHI => InstructionOpcode::PHI,
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
}

#[derive(Debug, PartialEq, Eq)]
pub struct InstructionValue {
    instruction_value: Value,
}

impl InstructionValue {
    pub(crate) fn new(instruction_value: LLVMValueRef) -> Self {
        assert!(!instruction_value.is_null());

        let value = Value::new(instruction_value);

        assert!(value.is_instruction());

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

    // REVIEW: See if necessary to check opcode == Call first.
    // Does it always return false otherwise?
    pub fn is_tail_call(&self) -> bool {
        unsafe {
            LLVMIsTailCall(self.as_value_ref()) == 1
        }
    }
}

impl AsValueRef for InstructionValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.instruction_value.value
    }
}
