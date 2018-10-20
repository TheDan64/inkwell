//! A `BasicBlock` is a container of instructions.

use llvm_sys::core::{LLVMGetBasicBlockParent, LLVMGetBasicBlockTerminator, LLVMGetNextBasicBlock, LLVMInsertBasicBlock, LLVMIsABasicBlock, LLVMIsConstant, LLVMMoveBasicBlockAfter, LLVMMoveBasicBlockBefore, LLVMPrintTypeToString, LLVMPrintValueToString, LLVMTypeOf, LLVMDeleteBasicBlock, LLVMGetPreviousBasicBlock, LLVMRemoveBasicBlockFromParent, LLVMGetFirstInstruction, LLVMGetLastInstruction, LLVMGetTypeContext, LLVMBasicBlockAsValue};
#[llvm_versions(3.9 => latest)]
use llvm_sys::core::LLVMGetBasicBlockName;
use llvm_sys::prelude::{LLVMValueRef, LLVMBasicBlockRef};

use context::{Context, ContextRef};
use values::{FunctionValue, InstructionValue};

use std::fmt;
use std::ffi::{CStr, CString};
use std::rc::Rc;

/// A `BasicBlock` is a container of instructions.
///
/// `BasicBlock`s are values because they can be referenced by instructions (ie branching and switches).
///
/// A well formed `BasicBlock` is a list of non terminating instructions followed by a single terminating
/// instruction. `BasicBlock`s are allowed to be malformed prior to running validation because it may be useful
/// when constructing or modifying a program.
#[derive(PartialEq, Eq)]
pub struct BasicBlock {
    pub(crate) basic_block: LLVMBasicBlockRef,
}

impl BasicBlock {
    pub(crate) fn new(basic_block: LLVMBasicBlockRef) -> Option<Self> {
        if basic_block.is_null() {
            return None;
        }

        unsafe {
            // NOTE: There is a LLVMBasicBlockAsValue but it might be the same as casting
            assert!(!LLVMIsABasicBlock(basic_block as LLVMValueRef).is_null())
        }

        Some(BasicBlock { basic_block })
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
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// assert_eq!(basic_block.get_parent().unwrap(), function);
    ///
    /// basic_block.remove_from_function();
    ///
    /// assert!(basic_block.get_parent().is_none());
    /// ```
    pub fn get_parent(&self) -> Option<FunctionValue> {
        let value = unsafe {
            LLVMGetBasicBlockParent(self.basic_block)
        };

        FunctionValue::new(value)
    }

    /// Gets the `BasicBlock` preceeding the current one, in its own scope, if any.
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
    /// let basic_block1 = context.append_basic_block(&function1, "entry");
    ///
    /// assert!(basic_block1.get_previous_basic_block().is_none());
    ///
    /// let function2 = module.add_function("do_nothing", fn_type, None);
    ///
    /// let basic_block2 = context.append_basic_block(&function2, "entry");
    /// let basic_block3 = context.append_basic_block(&function2, "next");
    ///
    /// assert!(basic_block2.get_previous_basic_block().is_none());
    /// assert_eq!(basic_block3.get_previous_basic_block().unwrap(), basic_block2);
    /// ```
    pub fn get_previous_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetPreviousBasicBlock(self.basic_block)
        };

        BasicBlock::new(bb)
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
    /// let basic_block1 = context.append_basic_block(&function1, "entry");
    ///
    /// assert!(basic_block1.get_next_basic_block().is_none());
    ///
    /// let function2 = module.add_function("do_nothing", fn_type, None);
    ///
    /// let basic_block2 = context.append_basic_block(&function2, "entry");
    /// let basic_block3 = context.append_basic_block(&function2, "next");
    ///
    /// assert!(basic_block1.get_next_basic_block().is_none());
    /// assert_eq!(basic_block2.get_next_basic_block().unwrap(), basic_block3);
    /// assert!(basic_block3.get_next_basic_block().is_none());
    /// ```
    pub fn get_next_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetNextBasicBlock(self.basic_block)
        };

        BasicBlock::new(bb)
    }

    /// Prepends one `BasicBlock` before another.
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
    /// let basic_block1 = context.append_basic_block(&function, "entry");
    /// let basic_block2 = context.append_basic_block(&function, "next");
    ///
    /// basic_block2.move_before(&basic_block1);
    ///
    /// assert!(basic_block1.get_next_basic_block().is_none());
    /// assert_eq!(basic_block2.get_next_basic_block().unwrap(), basic_block1);
    /// ```
    // REVIEW: What happens if blocks are from different scopes?
    pub fn move_before(&self, basic_block: &BasicBlock) {
        unsafe {
            LLVMMoveBasicBlockBefore(self.basic_block, basic_block.basic_block)
        }
    }

    /// Appends one `BasicBlock` after another.
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
    /// let basic_block1 = context.append_basic_block(&function, "entry");
    /// let basic_block2 = context.append_basic_block(&function, "next");
    ///
    /// basic_block1.move_after(&basic_block2);
    ///
    /// assert!(basic_block1.get_next_basic_block().is_none());
    /// assert_eq!(basic_block2.get_next_basic_block().unwrap(), basic_block1);
    /// ```
    // REVIEW: What happens if blocks are from different scopes?
    pub fn move_after(&self, basic_block: &BasicBlock) {
        unsafe {
            LLVMMoveBasicBlockAfter(self.basic_block, basic_block.basic_block)
        }
    }

    /// Prepends a new `BasicBlock` before this one.
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
    /// let basic_block1 = context.append_basic_block(&function, "entry");
    /// let basic_block2 = basic_block1.prepend_basic_block("previous");
    ///
    /// assert!(basic_block1.get_next_basic_block().is_none());
    /// assert_eq!(basic_block2.get_next_basic_block().unwrap(), basic_block1);
    /// ```
    pub fn prepend_basic_block(&self, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMInsertBasicBlock(self.basic_block, c_string.as_ptr())
        };

        BasicBlock::new(bb).expect("Prepending basic block should never fail")
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
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// builder.position_at_end(&basic_block);
    /// builder.build_return(None);
    ///
    /// assert_eq!(basic_block.get_first_instruction().unwrap().get_opcode(), InstructionOpcode::Return);
    /// ```
    pub fn get_first_instruction(&self) -> Option<InstructionValue> {
        let value = unsafe {
            LLVMGetFirstInstruction(self.basic_block)
        };

        if value.is_null() {
            return None;
        }

        Some(InstructionValue::new(value))
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
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// builder.position_at_end(&basic_block);
    /// builder.build_return(None);
    ///
    /// assert_eq!(basic_block.get_last_instruction().unwrap().get_opcode(), InstructionOpcode::Return);
    /// ```
    pub fn get_last_instruction(&self) -> Option<InstructionValue> {
        let value = unsafe {
            LLVMGetLastInstruction(self.basic_block)
        };

        if value.is_null() {
            return None;
        }

        Some(InstructionValue::new(value))
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
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// builder.position_at_end(&basic_block);
    /// builder.build_return(None);
    ///
    /// assert_eq!(basic_block.get_terminator().unwrap().get_opcode(), InstructionOpcode::Return);
    /// ```
    // REVIEW: If we wanted the return type could be Option<Either<BasicValueEnum, InstructionValue>>
    // if getting a value over an instruction is preferable
    // TODOC: Every BB must have a terminating instruction or else it is invalid
    // REVIEW: Unclear how this differs from get_last_instruction
    pub fn get_terminator(&self) -> Option<InstructionValue> {
        let value = unsafe {
            LLVMGetBasicBlockTerminator(self.basic_block)
        };

        if value.is_null() {
            return None;
        }

        Some(InstructionValue::new(value))
    }

    /// Removes this `BasicBlock` from its parent `FunctionValue`. Does nothing if it has no parent.
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
    /// let basic_block = context.append_basic_block(&function, "entry");
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
    pub fn remove_from_function(&self) {
        // This method is UB if the parent no longer exists, so we must check for parent (or encode into type system)
        if self.get_parent().is_some() {
            unsafe {
                LLVMRemoveBasicBlockFromParent(self.basic_block)
            }
        }
    }

    /// Removes this `BasicBlock` completely from memory. This is unsafe because you could easily have other references to the same `BasicBlock`.
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
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// unsafe {
    ///     basic_block.delete();
    /// }
    /// assert!(function.get_basic_blocks().is_empty());
    /// ```
    // REVIEW: Could potentially be unsafe if there are existing references. Might need a global ref counter
    pub unsafe fn delete(self) {
        // unsafe {
        LLVMDeleteBasicBlock(self.basic_block)
        // }
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
    /// let basic_block = context.append_basic_block(&function, "entry");
    ///
    /// assert_eq!(context, *basic_block.get_context());
    /// ```
    pub fn get_context(&self) -> ContextRef {
        let context = unsafe {
            LLVMGetTypeContext(LLVMTypeOf(LLVMBasicBlockAsValue(self.basic_block)))
        };

        // REVIEW: This probably should be somehow using the existing context Rc
        ContextRef::new(Context::new(Rc::new(context)))
    }

    /// Gets the name of a `BasicBlock`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use std::ffi::CString;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_val = module.add_function("my_fn", fn_type, None);
    /// let bb = context.append_basic_block(&fn_val, "entry");
    ///
    /// assert_eq!(*bb.get_name(), *CString::new("entry").unwrap());
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn get_name(&self) -> &CStr {
        let ptr = unsafe {
            LLVMGetBasicBlockName(self.basic_block)
        };

        unsafe {
            CStr::from_ptr(ptr)
        }
    }
}

impl fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.basic_block as LLVMValueRef))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.basic_block as LLVMValueRef)))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.basic_block as LLVMValueRef) == 1
        };

        f.debug_struct("BasicBlock")
            .field("address", &self.basic_block)
            .field("is_const", &is_const)
            .field("llvm_value", &llvm_value)
            .field("llvm_type", &llvm_type)
            .finish()
    }
}
