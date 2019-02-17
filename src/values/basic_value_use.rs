use either::{Either, Either::{Left, Right}};
use llvm_sys::core::{LLVMGetNextUse, LLVMGetUser, LLVMGetUsedValue, LLVMIsABasicBlock, LLVMValueAsBasicBlock};
use llvm_sys::prelude::LLVMUseRef;

use basic_block::BasicBlock;
use values::{BasicValueEnum, InstructionValue};

/// A usage of a `BasicValue` in an `InstructionValue`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BasicValueUse(LLVMUseRef);

impl BasicValueUse {
    pub(crate) fn new(use_: LLVMUseRef) -> Self {
        debug_assert!(!use_.is_null());

        BasicValueUse(use_)
    }

    /// Gets the next use of an `InstructionValue` or `BasicValue` if any.
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
    /// let arg1_first_use = arg1.get_first_use().unwrap();
    ///
    /// assert!(arg1_first_use.get_next_use().is_some());
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
    /// which makes the arg1 (%0) uses clear:
    /// 1) In the store instruction
    /// 2) In the pointer bitcast
    pub fn get_next_use(&self) -> Option<Self> {
        let use_ = unsafe {
            LLVMGetNextUse(self.0)
        };

        if use_.is_null() {
            return None;
        }

        Some(Self::new(use_))
    }

    /// Gets the user(an `InstructionValue`) of this use.
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
    /// let store_operand_use0 = store_instruction.get_operand_use(0).unwrap();
    /// let store_operand_use1 = store_instruction.get_operand_use(1).unwrap();
    ///
    /// assert_eq!(store_operand_use0.get_user(), store_instruction);
    /// assert_eq!(store_operand_use1.get_user(), store_instruction);
    /// ```
    pub fn get_user(&self) -> InstructionValue {
        let user = unsafe {
            LLVMGetUser(self.0)
        };

        InstructionValue::new(user)
    }

    /// Gets the used value(a `BasicValueEnum` or `BasicBlock`) of this use.
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
    /// let free_operand0 = free_instruction.get_operand(0).unwrap().left().unwrap();
    /// let free_operand0_instruction = free_operand0.as_instruction_value().unwrap();
    /// let bitcast_use_value = free_operand0_instruction
    ///     .get_first_use()
    ///     .unwrap()
    ///     .get_used_value()
    ///     .left()
    ///     .unwrap();
    ///
    /// assert_eq!(bitcast_use_value, free_operand0);
    /// ```
    pub fn get_used_value(&self) -> Either<BasicValueEnum, BasicBlock> {
        let used_value = unsafe {
            LLVMGetUsedValue(self.0)
        };

        let is_basic_block = unsafe {
            !LLVMIsABasicBlock(used_value).is_null()
        };

        if is_basic_block {
            let used_value = unsafe {
                LLVMValueAsBasicBlock(used_value)
            };

            Right(BasicBlock::new(used_value).expect("BasicBlock should be valid"))
        } else {
            Left(BasicValueEnum::new(used_value))
        }
    }
}
