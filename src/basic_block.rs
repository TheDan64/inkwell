//! A `BasicBlock` is a container of instructions.

use llvm_sys::core::{
    LLVMBasicBlockAsValue, LLVMBlockAddress, LLVMDeleteBasicBlock, LLVMGetBasicBlockName, LLVMGetBasicBlockParent,
    LLVMGetBasicBlockTerminator, LLVMGetFirstInstruction, LLVMGetFirstUse, LLVMGetLastInstruction,
    LLVMGetNextBasicBlock, LLVMGetPreviousBasicBlock, LLVMGetTypeContext, LLVMIsABasicBlock, LLVMIsConstant,
    LLVMMoveBasicBlockAfter, LLVMMoveBasicBlockBefore, LLVMPrintTypeToString, LLVMPrintValueToString,
    LLVMRemoveBasicBlockFromParent, LLVMReplaceAllUsesWith, LLVMSetValueName2, LLVMTypeOf,
};
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMValueRef};

use crate::context::ContextRef;
use crate::support::to_c_str;
use crate::values::{AsValueRef, BasicValueUse, FunctionValue, InstructionValue, PointerValue};

use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;

/// A `BasicBlock` is a container of instructions.
///
/// `BasicBlock`s are values because they can be referenced by instructions (ie branching and switches).
///
/// A well formed `BasicBlock` is a list of non terminating instructions followed by a single terminating
/// instruction. `BasicBlock`s are allowed to be malformed prior to running validation because it may be useful
/// when constructing or modifying a program.
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct BasicBlock<'ctx> {
    pub(crate) basic_block: LLVMBasicBlockRef,
    _marker: PhantomData<&'ctx ()>,
}

impl<'ctx> BasicBlock<'ctx> {
    pub(crate) unsafe fn new(basic_block: LLVMBasicBlockRef) -> Option<Self> {
        if basic_block.is_null() {
            return None;
        }

        // NOTE: There is a LLVMBasicBlockAsValue but it might be the same as casting
        assert!(!LLVMIsABasicBlock(basic_block as LLVMValueRef).is_null());

        Some(BasicBlock {
            basic_block,
            _marker: PhantomData,
        })
    }

    /// Acquires the underlying raw pointer belonging to this `BasicBlock` type.
    pub fn as_mut_ptr(&self) -> LLVMBasicBlockRef {
        self.basic_block
    }

    /// Obtains the `FunctionValue` that this `BasicBlock` belongs to, if any.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function = module.add_function("do_nothing", fn_type, None);
    ///
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// assert_eq!(basic_block.get_parent().unwrap(), function);
    ///
    /// basic_block.remove_from_function();
    ///
    /// assert!(basic_block.get_parent().is_none());
    /// ```
    pub fn get_parent(self) -> Option<FunctionValue<'ctx>> {
        unsafe { FunctionValue::new(LLVMGetBasicBlockParent(self.basic_block)) }
    }

    /// Gets the `BasicBlock` preceding the current one, in its own scope, if any.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function1 = module.add_function("do_nothing", fn_type, None);
    ///
    /// let basic_block1 = context.append_basic_block(function1, "entry");
    ///
    /// assert!(basic_block1.get_previous_basic_block().is_none());
    ///
    /// let function2 = module.add_function("do_nothing", fn_type, None);
    ///
    /// let basic_block2 = context.append_basic_block(function2, "entry");
    /// let basic_block3 = context.append_basic_block(function2, "next");
    ///
    /// assert!(basic_block2.get_previous_basic_block().is_none());
    /// assert_eq!(basic_block3.get_previous_basic_block().unwrap(), basic_block2);
    /// ```
    pub fn get_previous_basic_block(self) -> Option<BasicBlock<'ctx>> {
        self.get_parent()?;

        unsafe { BasicBlock::new(LLVMGetPreviousBasicBlock(self.basic_block)) }
    }

    /// Gets the `BasicBlock` succeeding the current one, in its own scope, if any.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function1 = module.add_function("do_nothing", fn_type, None);
    ///
    /// let basic_block1 = context.append_basic_block(function1, "entry");
    ///
    /// assert!(basic_block1.get_next_basic_block().is_none());
    ///
    /// let function2 = module.add_function("do_nothing", fn_type, None);
    ///
    /// let basic_block2 = context.append_basic_block(function2, "entry");
    /// let basic_block3 = context.append_basic_block(function2, "next");
    ///
    /// assert!(basic_block1.get_next_basic_block().is_none());
    /// assert_eq!(basic_block2.get_next_basic_block().unwrap(), basic_block3);
    /// assert!(basic_block3.get_next_basic_block().is_none());
    /// ```
    pub fn get_next_basic_block(self) -> Option<BasicBlock<'ctx>> {
        self.get_parent()?;

        unsafe { BasicBlock::new(LLVMGetNextBasicBlock(self.basic_block)) }
    }

    /// Prepends one `BasicBlock` before another.
    /// It returns `Err(())` when either `BasicBlock` has no parent, as LLVM assumes they both have parents.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function = module.add_function("do_nothing", fn_type, None);
    ///
    /// let basic_block1 = context.append_basic_block(function, "entry");
    /// let basic_block2 = context.append_basic_block(function, "next");
    ///
    /// basic_block2.move_before(basic_block1);
    ///
    /// assert!(basic_block1.get_next_basic_block().is_none());
    /// assert_eq!(basic_block2.get_next_basic_block().unwrap(), basic_block1);
    /// ```
    // REVIEW: What happens if blocks are from different scopes?
    pub fn move_before(self, basic_block: BasicBlock<'ctx>) -> Result<(), ()> {
        // This method is UB if the parent no longer exists, so we must check for parent (or encode into type system)
        if self.get_parent().is_none() || basic_block.get_parent().is_none() {
            return Err(());
        }

        unsafe { LLVMMoveBasicBlockBefore(self.basic_block, basic_block.basic_block) }

        Ok(())
    }

    /// Appends one `BasicBlock` after another.
    /// It returns `Err(())` when either `BasicBlock` has no parent, as LLVM assumes they both have parents.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function = module.add_function("do_nothing", fn_type, None);
    ///
    /// let basic_block1 = context.append_basic_block(function, "entry");
    /// let basic_block2 = context.append_basic_block(function, "next");
    ///
    /// basic_block1.move_after(basic_block2);
    ///
    /// assert!(basic_block1.get_next_basic_block().is_none());
    /// assert_eq!(basic_block2.get_next_basic_block().unwrap(), basic_block1);
    /// ```
    // REVIEW: What happens if blocks are from different scopes?
    pub fn move_after(self, basic_block: BasicBlock<'ctx>) -> Result<(), ()> {
        // This method is UB if the parent no longer exists, so we must check for parent (or encode into type system)
        if self.get_parent().is_none() || basic_block.get_parent().is_none() {
            return Err(());
        }

        unsafe { LLVMMoveBasicBlockAfter(self.basic_block, basic_block.basic_block) }

        Ok(())
    }

    /// Obtains the first `InstructionValue` in this `BasicBlock`, if any.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    /// use inkwell::values::InstructionOpcode;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function = module.add_function("do_nothing", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    /// builder.build_return(None);
    ///
    /// assert_eq!(basic_block.get_first_instruction().unwrap().get_opcode(), InstructionOpcode::Return);
    /// ```
    pub fn get_first_instruction(self) -> Option<InstructionValue<'ctx>> {
        let value = unsafe { LLVMGetFirstInstruction(self.basic_block) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(InstructionValue::new(value)) }
    }

    /// Obtains the last `InstructionValue` in this `BasicBlock`, if any. A `BasicBlock` must have a last instruction to be valid.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    /// use inkwell::values::InstructionOpcode;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function = module.add_function("do_nothing", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    /// builder.build_return(None);
    ///
    /// assert_eq!(basic_block.get_last_instruction().unwrap().get_opcode(), InstructionOpcode::Return);
    /// ```
    pub fn get_last_instruction(self) -> Option<InstructionValue<'ctx>> {
        let value = unsafe { LLVMGetLastInstruction(self.basic_block) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(InstructionValue::new(value)) }
    }

    /// Performs a linear lookup to obtain a instruction based on the name
    ///
    /// # Example
    /// ```rust
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("ret");
    /// let builder = context.create_builder();
    ///
    /// let void_type = context.void_type();
    /// let i32_type = context.i32_type();
    /// #[cfg(feature = "typed-pointers")]
    /// let i32_ptr_type = i32_type.ptr_type(AddressSpace::default());
    /// #[cfg(not(feature = "typed-pointers"))]
    /// let i32_ptr_type = context.ptr_type(AddressSpace::default());
    ///
    /// let fn_type = void_type.fn_type(&[i32_ptr_type.into()], false);
    /// let fn_value = module.add_function("ret", fn_type, None);
    /// let entry = context.append_basic_block(fn_value, "entry");
    /// builder.position_at_end(entry);
    ///
    /// let var = builder.build_alloca(i32_type, "some_number").unwrap();
    /// builder.build_store(var, i32_type.const_int(1 as u64, false)).unwrap();
    /// builder.build_return(None).unwrap();
    ///
    /// let block = fn_value.get_first_basic_block().unwrap();
    /// let some_number = block.get_instruction_with_name("some_number");
    ///
    /// assert!(some_number.is_some());
    /// assert_eq!(some_number.unwrap().get_name().unwrap().to_str(), Ok("some_number"))
    /// ```
    pub fn get_instruction_with_name(self, name: &str) -> Option<InstructionValue<'ctx>> {
        let instruction = self.get_first_instruction()?;
        instruction.get_instruction_with_name(name)
    }

    /// Obtains the terminating `InstructionValue` in this `BasicBlock`, if any. A `BasicBlock` must have a terminating instruction to be valid.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    /// use inkwell::values::InstructionOpcode;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function = module.add_function("do_nothing", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// builder.position_at_end(basic_block);
    /// builder.build_return(None);
    ///
    /// assert_eq!(basic_block.get_terminator().unwrap().get_opcode(), InstructionOpcode::Return);
    /// ```
    // REVIEW: If we wanted the return type could be Option<Either<BasicValueEnum, InstructionValue>>
    // if getting a value over an instruction is preferable
    // TODOC: Every BB must have a terminating instruction or else it is invalid
    // REVIEW: Unclear how this differs from get_last_instruction
    pub fn get_terminator(self) -> Option<InstructionValue<'ctx>> {
        let value = unsafe { LLVMGetBasicBlockTerminator(self.basic_block) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(InstructionValue::new(value)) }
    }

    /// Get an instruction iterator
    pub fn get_instructions(self) -> InstructionIter<'ctx> {
        InstructionIter(self.get_first_instruction())
    }

    /// Removes this `BasicBlock` from its parent `FunctionValue`.
    /// It returns `Err(())` when it has no parent to remove from.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function = module.add_function("do_nothing", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// assert_eq!(basic_block.get_parent().unwrap(), function);
    ///
    /// basic_block.remove_from_function();
    ///
    /// assert!(basic_block.get_parent().is_none());
    /// ```
    // SubTypes: Don't need to call get_parent for a BasicBlock<HasParent> and would return BasicBlock<Orphan>
    // by taking ownership of self (though BasicBlock's are not uniquely obtained...)
    // might have to make some methods do something like -> Result<..., BasicBlock<Orphan>> for BasicBlock<HasParent>
    // and would move_before/after make it no longer orphaned? etc..
    pub fn remove_from_function(self) -> Result<(), ()> {
        // This method is UB if the parent no longer exists, so we must check for parent (or encode into type system)
        if self.get_parent().is_none() {
            return Err(());
        }

        unsafe { LLVMRemoveBasicBlockFromParent(self.basic_block) }

        Ok(())
    }

    /// Removes this `BasicBlock` completely from memory. This is unsafe because you could easily have other references to the same `BasicBlock`.
    /// It returns `Err(())` when it has no parent to delete from, as LLVM assumes it has a parent.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function = module.add_function("do_nothing", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// unsafe {
    ///     basic_block.delete();
    /// }
    /// assert!(function.get_basic_blocks().is_empty());
    /// ```
    pub unsafe fn delete(self) -> Result<(), ()> {
        // This method is UB if the parent no longer exists, so we must check for parent (or encode into type system)
        if self.get_parent().is_none() {
            return Err(());
        }

        LLVMDeleteBasicBlock(self.basic_block);

        Ok(())
    }

    /// Obtains the `ContextRef` this `BasicBlock` belongs to.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::builder::Builder;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let function = module.add_function("do_nothing", fn_type, None);
    /// let basic_block = context.append_basic_block(function, "entry");
    ///
    /// assert_eq!(context, basic_block.get_context());
    /// ```
    pub fn get_context(self) -> ContextRef<'ctx> {
        unsafe { ContextRef::new(LLVMGetTypeContext(LLVMTypeOf(LLVMBasicBlockAsValue(self.basic_block)))) }
    }

    /// Gets the name of a `BasicBlock`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_val = module.add_function("my_fn", fn_type, None);
    /// let bb = context.append_basic_block(fn_val, "entry");
    ///
    /// assert_eq!(bb.get_name().to_str(), Ok("entry"));
    /// ```
    pub fn get_name(&self) -> &CStr {
        let ptr = unsafe { LLVMGetBasicBlockName(self.basic_block) };

        unsafe { CStr::from_ptr(ptr) }
    }

    /// Set name of the `BasicBlock`.
    pub fn set_name(&self, name: &str) {
        let c_string = to_c_str(name);

        unsafe {
            LLVMSetValueName2(
                LLVMBasicBlockAsValue(self.basic_block),
                c_string.as_ptr(),
                c_string.to_bytes().len(),
            )
        };
    }

    /// Replaces all uses of this basic block with another.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_val = module.add_function("my_fn", fn_type, None);
    /// let entry = context.append_basic_block(fn_val, "entry");
    /// let bb1 = context.append_basic_block(fn_val, "bb1");
    /// let bb2 = context.append_basic_block(fn_val, "bb2");
    /// builder.position_at_end(entry);
    /// let branch_inst = builder.build_unconditional_branch(bb1).unwrap();
    ///
    /// bb1.replace_all_uses_with(&bb2);
    ///
    /// assert_eq!(branch_inst.get_operand(0).unwrap().right().unwrap(), bb2);
    /// ```
    pub fn replace_all_uses_with(self, other: &BasicBlock<'ctx>) {
        let value = unsafe { LLVMBasicBlockAsValue(self.basic_block) };
        let other = unsafe { LLVMBasicBlockAsValue(other.basic_block) };

        // LLVM may infinite-loop when they aren't distinct, which is UB in C++.
        if value != other {
            unsafe {
                LLVMReplaceAllUsesWith(value, other);
            }
        }
    }

    /// Gets the first use of this `BasicBlock` if any.
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
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_val = module.add_function("my_fn", fn_type, None);
    /// let entry = context.append_basic_block(fn_val, "entry");
    /// let bb1 = context.append_basic_block(fn_val, "bb1");
    /// let bb2 = context.append_basic_block(fn_val, "bb2");
    /// builder.position_at_end(entry);
    /// let branch_inst = builder.build_unconditional_branch(bb1);
    ///
    /// assert!(bb2.get_first_use().is_none());
    /// assert!(bb1.get_first_use().is_some());
    /// ```
    pub fn get_first_use(self) -> Option<BasicValueUse<'ctx>> {
        let use_ = unsafe { LLVMGetFirstUse(LLVMBasicBlockAsValue(self.basic_block)) };

        if use_.is_null() {
            return None;
        }

        unsafe { Some(BasicValueUse::new(use_)) }
    }

    /// Gets the address of this `BasicBlock` if possible. Returns `None` if `self` is the entry block to a function.
    ///
    /// # Safety
    ///
    /// The returned PointerValue may only be used for `call` and `indirect_branch` instructions
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_val = module.add_function("my_fn", fn_type, None);
    /// let entry_bb = context.append_basic_block(fn_val, "entry");
    /// let next_bb = context.append_basic_block(fn_val, "next");
    ///
    /// assert!(unsafe { entry_bb.get_address() }.is_none());
    /// assert!(unsafe { next_bb.get_address() }.is_some());
    /// ```
    pub unsafe fn get_address(self) -> Option<PointerValue<'ctx>> {
        let parent = self.get_parent()?;

        // Taking the address of the entry block is illegal.
        self.get_previous_basic_block()?;

        let value = PointerValue::new(LLVMBlockAddress(parent.as_value_ref(), self.basic_block));

        if value.is_null() {
            return None;
        }

        Some(value)
    }
}

impl fmt::Debug for BasicBlock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe { CStr::from_ptr(LLVMPrintValueToString(self.basic_block as LLVMValueRef)) };
        let llvm_type = unsafe { CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.basic_block as LLVMValueRef))) };
        let is_const = unsafe { LLVMIsConstant(self.basic_block as LLVMValueRef) == 1 };

        f.debug_struct("BasicBlock")
            .field("address", &self.basic_block)
            .field("is_const", &is_const)
            .field("llvm_value", &llvm_value)
            .field("llvm_type", &llvm_type)
            .finish()
    }
}

/// Iterate over all `InstructionValue`s in a basic block.
#[derive(Debug)]
pub struct InstructionIter<'ctx>(Option<InstructionValue<'ctx>>);

impl<'ctx> Iterator for InstructionIter<'ctx> {
    type Item = InstructionValue<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(instr) = self.0 {
            self.0 = instr.get_next_instruction();
            Some(instr)
        } else {
            None
        }
    }
}
