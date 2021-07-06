use either::{Either, Either::{Left, Right}};
use llvm_sys::core::{LLVMGetAlignment, LLVMSetAlignment, LLVMGetInstructionOpcode, LLVMIsTailCall, LLVMGetPreviousInstruction, LLVMGetNextInstruction, LLVMGetInstructionParent, LLVMInstructionEraseFromParent, LLVMInstructionClone, LLVMSetVolatile, LLVMGetVolatile, LLVMGetNumOperands, LLVMGetOperand, LLVMGetOperandUse, LLVMSetOperand, LLVMValueAsBasicBlock, LLVMIsABasicBlock, LLVMGetICmpPredicate, LLVMGetFCmpPredicate, LLVMIsAAllocaInst, LLVMIsALoadInst, LLVMIsAStoreInst, LLVMGetMetadata, LLVMHasMetadata, LLVMSetMetadata};
#[llvm_versions(3.8..=latest)]
use llvm_sys::core::{LLVMGetOrdering, LLVMSetOrdering};
#[llvm_versions(3.9..=latest)]
use llvm_sys::core::LLVMInstructionRemoveFromParent;
#[llvm_versions(10.0..=latest)]
use llvm_sys::core::{LLVMIsAAtomicRMWInst, LLVMIsAAtomicCmpXchgInst};
use llvm_sys::LLVMOpcode;
use llvm_sys::prelude::LLVMValueRef;

use crate::basic_block::BasicBlock;
use crate::values::traits::AsValueRef;
use crate::values::{BasicValue, BasicValueEnum, BasicValueUse, Value, MetadataValue};
use crate::{AtomicOrdering, IntPredicate, FloatPredicate};

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
    #[llvm_versions(9.0..=latest)]
    CallBr,
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
    #[llvm_versions(10.0..=latest)]
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

#[derive(Debug, PartialEq, Eq, Copy, Hash)]
pub struct InstructionValue<'ctx> {
    instruction_value: Value<'ctx>,
}

impl<'ctx> InstructionValue<'ctx> {
    fn is_a_load_inst(self) -> bool {
        !unsafe { LLVMIsALoadInst(self.as_value_ref()) }.is_null()
    }
    fn is_a_store_inst(self) -> bool {
        !unsafe { LLVMIsAStoreInst(self.as_value_ref()) }.is_null()
    }
    fn is_a_alloca_inst(self) -> bool {
        !unsafe { LLVMIsAAllocaInst(self.as_value_ref()) }.is_null()
    }
    #[llvm_versions(10.0..=latest)]
    fn is_a_atomicrmw_inst(self) -> bool {
        !unsafe { LLVMIsAAtomicRMWInst(self.as_value_ref()) }.is_null()
    }
    #[llvm_versions(10.0..=latest)]
    fn is_a_cmpxchg_inst(self) -> bool {
        !unsafe { LLVMIsAAtomicCmpXchgInst(self.as_value_ref()) }.is_null()
    }

    pub(crate) unsafe fn new(instruction_value: LLVMValueRef) -> Self {
        debug_assert!(!instruction_value.is_null());

        let value = Value::new(instruction_value);

        debug_assert!(value.is_instruction());

        InstructionValue {
            instruction_value: value,
        }
    }

    pub fn get_opcode(self) -> InstructionOpcode {
        let opcode = unsafe {
            LLVMGetInstructionOpcode(self.as_value_ref())
        };

        InstructionOpcode::new(opcode)
    }

    pub fn get_previous_instruction(self) -> Option<Self> {
        let value = unsafe {
            LLVMGetPreviousInstruction(self.as_value_ref())
        };

        if value.is_null() {
            return None;
        }

        unsafe {
            Some(InstructionValue::new(value))
        }
    }

    pub fn get_next_instruction(self) -> Option<Self> {
        let value = unsafe {
            LLVMGetNextInstruction(self.as_value_ref())
        };

        if value.is_null() {
            return None;
        }

        unsafe {
            Some(InstructionValue::new(value))
        }
    }

    // REVIEW: Potentially unsafe if parent BB or grandparent fn were removed?
    pub fn erase_from_basic_block(self) {
        unsafe {
            LLVMInstructionEraseFromParent(self.as_value_ref())
        }
    }

    // REVIEW: Potentially unsafe if parent BB or grandparent fn were removed?
    #[llvm_versions(3.9..=latest)]
    pub fn remove_from_basic_block(self) {
        unsafe {
            LLVMInstructionRemoveFromParent(self.as_value_ref())
        }
    }

    // REVIEW: Potentially unsafe is parent BB or grandparent fn was deleted
    // REVIEW: Should this *not* be an option? Parent should always exist,
    // but I doubt LLVM returns null if the parent BB (or grandparent FN)
    // was deleted... Invalid memory is more likely. Cloned IV will have no
    // parent?
    pub fn get_parent(self) -> Option<BasicBlock<'ctx>> {
        unsafe {
            BasicBlock::new(LLVMGetInstructionParent(self.as_value_ref()))
        }
    }

    pub fn is_tail_call(self) -> bool {
        // LLVMIsTailCall has UB if the value is not an llvm::CallInst*.
        if self.get_opcode() == InstructionOpcode::Call {
            unsafe {
                LLVMIsTailCall(self.as_value_ref()) == 1
            }
        } else {
            false
        }
    }

    pub fn replace_all_uses_with(self, other: &InstructionValue<'ctx>) {
        self.instruction_value.replace_all_uses_with(other.as_value_ref())
    }

    // SubTypes: Only apply to memory access instructions
    /// Returns whether or not a memory access instruction is volatile.
    #[llvm_versions(3.6..=9.0)]
    pub fn get_volatile(self) -> Result<bool, &'static str> {
        // Although cmpxchg and atomicrmw can have volatile, LLVM's C API
        // does not export that functionality until 10.0.
        if !self.is_a_load_inst() && !self.is_a_store_inst() {
            return Err("Value is not a load or store.");
        }
        Ok(unsafe { LLVMGetVolatile(self.as_value_ref()) } == 1)
    }

    // SubTypes: Only apply to memory access instructions
    /// Returns whether or not a memory access instruction is volatile.
    #[llvm_versions(10.0..=latest)]
    pub fn get_volatile(self) -> Result<bool, &'static str> {
        if !self.is_a_load_inst() && !self.is_a_store_inst() &&
           !self.is_a_atomicrmw_inst() && !self.is_a_cmpxchg_inst() {
            return Err("Value is not a load, store, atomicrmw or cmpxchg.");
        }
        Ok(unsafe { LLVMGetVolatile(self.as_value_ref()) } == 1)
    }

    // SubTypes: Only apply to memory access instructions
    /// Sets whether or not a memory access instruction is volatile.
    #[llvm_versions(3.6..=9.0)]
    pub fn set_volatile(self, volatile: bool) -> Result<(), &'static str> {
        // Although cmpxchg and atomicrmw can have volatile, LLVM's C API
        // does not export that functionality until 10.0.
        if !self.is_a_load_inst() && !self.is_a_store_inst() {
            return Err("Value is not a load or store.");
        }
        Ok(unsafe { LLVMSetVolatile(self.as_value_ref(), volatile as i32) })
    }

    // SubTypes: Only apply to memory access instructions
    /// Sets whether or not a memory access instruction is volatile.
    #[llvm_versions(10.0..=latest)]
    pub fn set_volatile(self, volatile: bool) -> Result<(), &'static str> {
        if !self.is_a_load_inst() && !self.is_a_store_inst() &&
           !self.is_a_atomicrmw_inst() && !self.is_a_cmpxchg_inst() {
            return Err("Value is not a load, store, atomicrmw or cmpxchg.");
        }
        Ok(unsafe { LLVMSetVolatile(self.as_value_ref(), volatile as i32) })
    }

    // SubTypes: Only apply to memory access and alloca instructions
    /// Returns alignment on a memory access instruction or alloca.
    pub fn get_alignment(self) -> Result<u32, &'static str> {
        if !self.is_a_alloca_inst() && !self.is_a_load_inst() && !self.is_a_store_inst() {
            return Err("Value is not an alloca, load or store.");
        }
        Ok(unsafe { LLVMGetAlignment(self.as_value_ref()) })
    }

    // SubTypes: Only apply to memory access and alloca instructions
    /// Sets alignment on a memory access instruction or alloca.
    pub fn set_alignment(self, alignment: u32) -> Result<(), &'static str> {
        #[cfg(any(feature = "llvm11-0", feature = "llvm12-0"))]
        {
            if alignment == 0 {
                return Err("Alignment cannot be 0");
            }
        }
        //The alignment = 0 check above covers LLVM >= 11, the != 0 check here keeps older versions compatible
        if !alignment.is_power_of_two() && alignment != 0 {
            return Err("Alignment is not a power of 2!");
        }
        if !self.is_a_alloca_inst() && !self.is_a_load_inst() && !self.is_a_store_inst() {
            return Err("Value is not an alloca, load or store.");
        }
        Ok(unsafe { LLVMSetAlignment(self.as_value_ref(), alignment) })
    }

    // SubTypes: Only apply to memory access instructions
    /// Returns atomic ordering on a memory access instruction.
    #[llvm_versions(3.8..=latest)]
    pub fn get_atomic_ordering(self) -> Result<AtomicOrdering, &'static str> {
        if !self.is_a_load_inst() && !self.is_a_store_inst() {
            return Err("Value is not a load or store.");
        }
        Ok(unsafe { LLVMGetOrdering(self.as_value_ref()) }.into())
    }

    // SubTypes: Only apply to memory access instructions
    /// Sets atomic ordering on a memory access instruction.
    #[llvm_versions(3.8..=latest)]
    pub fn set_atomic_ordering(self, ordering: AtomicOrdering) -> Result<(), &'static str> {
        // Although fence and atomicrmw both have an ordering, the LLVM C API
        // does not support them. The cmpxchg instruction has two orderings and
        // does not work with this API.
        if !self.is_a_load_inst() && !self.is_a_store_inst() {
            return Err("Value is not a load or store instruction.");
        }
        match ordering {
            AtomicOrdering::Release if self.is_a_load_inst() =>
                return Err("The release ordering is not valid on load instructions."),
            AtomicOrdering::AcquireRelease =>
                return Err("The acq_rel ordering is not valid on load or store instructions."),
            AtomicOrdering::Acquire if self.is_a_store_inst() =>
                return Err("The acquire ordering is not valid on store instructions."),
            _ => { },
        };
        Ok(unsafe { LLVMSetOrdering(self.as_value_ref(), ordering.into()) })
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
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
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
    pub fn get_num_operands(self) -> u32 {
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
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
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
    pub fn get_operand(self, index: u32) -> Option<Either<BasicValueEnum<'ctx>, BasicBlock<'ctx>>> {
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
            let bb = unsafe {
                BasicBlock::new(LLVMValueAsBasicBlock(operand))
            };

            Some(Right(bb.expect("BasicBlock should always be valid")))
        } else {
            Some(Left(unsafe { BasicValueEnum::new(operand) }))
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
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
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
    pub fn set_operand<BV: BasicValue<'ctx>>(self, index: u32, val: BV) -> bool {
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
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(::std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val);
    /// let free_instruction = builder.build_free(arg1);
    /// let return_instruction = builder.build_return(None);
    ///
    /// assert_eq!(store_instruction.get_operand_use(1), arg1.get_first_use());
    /// ```
    pub fn get_operand_use(self, index: u32) -> Option<BasicValueUse<'ctx>> {
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

        unsafe {
            Some(BasicValueUse::new(use_))
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
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    /// let fn_type = void_type.fn_type(&[f32_ptr_type.into()], false);
    ///
    /// let function = module.add_function("take_f32_ptr", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    ///
    /// let arg1 = function.get_first_param().unwrap().into_pointer_value();
    /// let f32_val = f32_type.const_float(::std::f64::consts::PI);
    /// let store_instruction = builder.build_store(arg1, f32_val);
    /// let free_instruction = builder.build_free(arg1);
    /// let return_instruction = builder.build_return(None);
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
    pub fn get_fcmp_predicate(self) -> Option<FloatPredicate> {
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

    /// Determines whether or not this `Instruction` has any associated metadata.
    pub fn has_metadata(self) -> bool {
        unsafe {
            LLVMHasMetadata(self.instruction_value.value) == 1
        }
    }

    /// Gets the `MetadataValue` associated with this `Instruction` at a specific
    /// `kind_id`.
    pub fn get_metadata(self, kind_id: u32) -> Option<MetadataValue<'ctx>> {
        let metadata_value = unsafe {
            LLVMGetMetadata(self.instruction_value.value, kind_id)
        };

        if metadata_value.is_null() {
            return None;
        }

        unsafe {
            Some(MetadataValue::new(metadata_value))
        }
    }

    /// Determines whether or not this `Instruction` has any associated metadata
    /// `kind_id`.
    pub fn set_metadata(self, metadata: MetadataValue<'ctx>, kind_id: u32) -> Result<(), &'static str> {
        if !metadata.is_node() {
            return Err("metadata is expected to be a node.")
        }

        unsafe {
            LLVMSetMetadata(self.instruction_value.value, kind_id, metadata.as_value_ref());
        }

        Ok(())
    }
}

impl Clone for InstructionValue<'_> {
    /// Creates a clone of this `InstructionValue`, and returns it.
    /// The clone will have no parent, and no name.
    fn clone(&self) -> Self {
        unsafe {
            InstructionValue::new(LLVMInstructionClone(self.as_value_ref()))
        }
    }
}

impl AsValueRef for InstructionValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.instruction_value.value
    }
}
