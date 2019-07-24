use either::{Either, Either::{Left, Right}};
use llvm_sys::core::{LLVMGetInstructionOpcode, LLVMIsTailCall, LLVMGetPreviousInstruction, LLVMGetNextInstruction, LLVMGetInstructionParent, LLVMInstructionEraseFromParent, LLVMInstructionClone, LLVMSetVolatile, LLVMGetVolatile, LLVMGetNumOperands, LLVMGetOperand, LLVMGetOperandUse, LLVMSetOperand, LLVMValueAsBasicBlock, LLVMIsABasicBlock, LLVMGetICmpPredicate, LLVMGetFCmpPredicate};
#[llvm_versions(3.9..=latest)]
use llvm_sys::core::LLVMInstructionRemoveFromParent;
use llvm_sys::LLVMOpcode;
use llvm_sys::prelude::LLVMValueRef;

use crate::basic_block::BasicBlock;
use crate::values::traits::AsValueRef;
use crate::values::{BasicValue, BasicValueEnum, BasicValueUse, Value};
use crate::{IntPredicate, FloatPredicate};

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
    #[llvm_versions(3.8..=latest)]
    CatchPad,
    #[llvm_versions(3.8..=latest)]
    CatchRet,
    #[llvm_versions(3.8..=latest)]
    CatchSwitch,
    #[llvm_versions(3.8..=latest)]
    CleanupPad,
    #[llvm_versions(3.8..=latest)]
    CleanupRet,
    ExtractElement,
    ExtractValue,
    #[llvm_versions(8.0..=latest)]
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

#[derive(Debug, PartialEq, Eq, Copy, Hash)]
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
    #[llvm_versions(3.9..=latest)]
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
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// builder.position_at_end(&basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(::std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val);
    /// let free_instruction = builder.build_free(arg1);
    /// let return_instruction = builder.build_return(None);
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
    /// even though the return instruction can take values.
    pub fn get_num_operands(&self) -> u32 {
        unsafe {
            LLVMGetNumOperands(self.as_value_ref()) as u32
        }
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
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// builder.position_at_end(&basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(::std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val);
    /// let free_instruction = builder.build_free(arg1);
    /// let return_instruction = builder.build_return(None);
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
    /// even though the return instruction can take values.
    pub fn get_operand(&self, index: u32) -> Option<Either<BasicValueEnum, BasicBlock>> {
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

        let is_basic_block = unsafe {
            !LLVMIsABasicBlock(operand).is_null()
        };

        if is_basic_block {
            let operand = unsafe {
                LLVMValueAsBasicBlock(operand)
            };

            Some(Right(BasicBlock::new(operand).expect("BasicBlock should be valid")))
        } else {
            Some(Left(BasicValueEnum::new(operand)))
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
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// builder.position_at_end(&basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(::std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val);
    /// let free_instruction = builder.build_free(arg1);
    /// let return_instruction = builder.build_return(None);
    ///
    /// // This will produce invalid IR:
    /// free_instruction.set_operand(0, f32_val);
    ///
    /// assert_eq!(free_instruction.get_operand(0).unwrap().left().unwrap(), f32_val);
    /// ```
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
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// builder.position_at_end(&basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(::std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val);
    /// let free_instruction = builder.build_free(arg1);
    /// let return_instruction = builder.build_return(None);
    ///
    /// assert_eq!(store_instruction.get_operand_use(1), arg1.get_first_use());
    /// ```
    pub fn get_operand_use(&self, index: u32) -> Option<BasicValueUse> {
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

        Some(BasicValueUse::new(use_))
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
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// builder.position_at_end(&basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(::std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val);
    /// let free_instruction = builder.build_free(arg1);
    /// let return_instruction = builder.build_return(None);
    ///
    /// assert!(arg1.get_first_use().is_some());
    /// ```
    pub fn get_first_use(&self) -> Option<BasicValueUse> {
        self.instruction_value.get_first_use()
    }

    /// Gets the predicate of an `ICmp` `InstructionValue`.
    /// For instance, in the LLVM instruction
    /// `%3 = icmp slt i32 %0, %1`
    /// this gives the `slt`.
    ///
    /// If the instruction is not an `ICmp`, this returns None.
    pub fn get_icmp_predicate(&self) -> Option<IntPredicate> {
        // REVIEW: this call to get_opcode() can be inefficient;
        // what happens if we don't perform this check, and just call
        // LLVMGetICmpPredicate() regardless?
        if self.get_opcode() == InstructionOpcode::ICmp {
            let pred = unsafe {
                LLVMGetICmpPredicate(self.as_value_ref())
            };
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
    pub fn get_fcmp_predicate(&self) -> Option<FloatPredicate> {
        // REVIEW: this call to get_opcode() can be inefficient;
        // what happens if we don't perform this check, and just call
        // LLVMGetFCmpPredicate() regardless?
        if self.get_opcode() == InstructionOpcode::FCmp {
            let pred = unsafe {
                LLVMGetFCmpPredicate(self.as_value_ref())
            };
            Some(FloatPredicate::new(pred))
        } else {
            None
        }
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
