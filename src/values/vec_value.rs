#[llvm_versions(..=16)]
use llvm_sys::core::LLVMConstSelect;
#[allow(deprecated)]
use llvm_sys::core::LLVMGetElementAsConstant;
use llvm_sys::core::{
    LLVMConstExtractElement, LLVMConstInsertElement, LLVMConstShuffleVector, LLVMIsAConstantDataVector,
    LLVMIsAConstantVector,
};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;
use std::fmt::{self, Display};

use crate::types::VectorType;
use crate::values::traits::AsValueRef;
use crate::values::{BasicValue, BasicValueEnum, InstructionValue, IntValue, Value};

use super::AnyValue;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct VectorValue<'ctx> {
    vec_value: Value<'ctx>,
}

impl<'ctx> VectorValue<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of type vector.
    pub unsafe fn new(vector_value: LLVMValueRef) -> Self {
        assert!(!vector_value.is_null());

        VectorValue {
            vec_value: Value::new(vector_value),
        }
    }

    /// Determines whether or not a `VectorValue` is a constant.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_vec_type = i8_type.vec_type(3);
    /// let i8_vec_zero = i8_vec_type.const_zero();
    ///
    /// assert!(i8_vec_zero.is_const());
    /// ```
    pub fn is_const(self) -> bool {
        self.vec_value.is_const()
    }

    pub fn is_constant_vector(self) -> bool {
        unsafe { !LLVMIsAConstantVector(self.as_value_ref()).is_null() }
    }

    pub fn is_constant_data_vector(self) -> bool {
        unsafe { !LLVMIsAConstantDataVector(self.as_value_ref()).is_null() }
    }

    pub fn print_to_stderr(self) {
        self.vec_value.print_to_stderr()
    }

    /// Gets the name of a `VectorValue`. If the value is a constant, this will
    /// return an empty string.
    pub fn get_name(&self) -> &CStr {
        self.vec_value.get_name()
    }

    /// Set name of the `VectorValue`.
    pub fn set_name(&self, name: &str) {
        self.vec_value.set_name(name)
    }

    pub fn get_type(self) -> VectorType<'ctx> {
        unsafe { VectorType::new(self.vec_value.get_type()) }
    }

    pub fn is_null(self) -> bool {
        self.vec_value.is_null()
    }

    pub fn is_undef(self) -> bool {
        self.vec_value.is_undef()
    }

    pub fn as_instruction(self) -> Option<InstructionValue<'ctx>> {
        self.vec_value.as_instruction()
    }

    pub fn const_extract_element(self, index: IntValue<'ctx>) -> BasicValueEnum<'ctx> {
        unsafe { BasicValueEnum::new(LLVMConstExtractElement(self.as_value_ref(), index.as_value_ref())) }
    }

    // SubTypes: value should really be T in self: VectorValue<T> I think
    pub fn const_insert_element<BV: BasicValue<'ctx>>(self, index: IntValue<'ctx>, value: BV) -> BasicValueEnum<'ctx> {
        unsafe {
            BasicValueEnum::new(LLVMConstInsertElement(
                self.as_value_ref(),
                value.as_value_ref(),
                index.as_value_ref(),
            ))
        }
    }

    pub fn replace_all_uses_with(self, other: VectorValue<'ctx>) {
        self.vec_value.replace_all_uses_with(other.as_value_ref())
    }

    // TODOC: Value seems to be zero initialized if index out of bounds
    // SubType: VectorValue<BV> -> BV
    #[allow(deprecated)]
    pub fn get_element_as_constant(self, index: u32) -> BasicValueEnum<'ctx> {
        unsafe { BasicValueEnum::new(LLVMGetElementAsConstant(self.as_value_ref(), index)) }
    }

    // SubTypes: self can only be VectoValue<IntValue<bool>>
    #[llvm_versions(..=16)]
    pub fn const_select<BV: BasicValue<'ctx>>(self, then: BV, else_: BV) -> BasicValueEnum<'ctx> {
        unsafe {
            BasicValueEnum::new(LLVMConstSelect(
                self.as_value_ref(),
                then.as_value_ref(),
                else_.as_value_ref(),
            ))
        }
    }

    // SubTypes: <V: VectorValue<T, Const>> self: V, right: V, mask: V -> V
    pub fn const_shuffle_vector(self, right: VectorValue<'ctx>, mask: VectorValue<'ctx>) -> VectorValue<'ctx> {
        unsafe {
            VectorValue::new(LLVMConstShuffleVector(
                self.as_value_ref(),
                right.as_value_ref(),
                mask.as_value_ref(),
            ))
        }
    }
}

unsafe impl AsValueRef for VectorValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.vec_value.value
    }
}

impl Display for VectorValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
