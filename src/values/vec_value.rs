use llvm_sys::core::{LLVMIsAConstantVector, LLVMIsAConstantDataVector, LLVMConstInsertElement, LLVMConstExtractElement, LLVMIsConstantString, LLVMGetElementAsConstant, LLVMGetAsString, LLVMConstSelect, LLVMConstShuffleVector};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use crate::support::LLVMString;
use crate::types::{VectorType};
use crate::values::traits::AsValueRef;
use crate::values::{BasicValueEnum, BasicValue, InstructionValue, Value, IntValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct VectorValue<'ctx> {
    vec_value: Value<'ctx>,
}

impl<'ctx> VectorValue<'ctx> {
    pub(crate) fn new(vector_value: LLVMValueRef) -> Self {
        assert!(!vector_value.is_null());

        VectorValue {
            vec_value: Value::new(vector_value)
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
    pub fn is_const(&self) -> bool {
        self.vec_value.is_const()
    }

    pub fn is_constant_vector(&self) -> bool {
        unsafe {
            !LLVMIsAConstantVector(self.as_value_ref()).is_null()
        }
    }

    pub fn is_constant_data_vector(&self) -> bool {
        unsafe {
            !LLVMIsAConstantDataVector(self.as_value_ref()).is_null()
        }
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.vec_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.vec_value.print_to_stderr()
    }

    pub fn get_name(&self) -> &CStr {
        self.vec_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.vec_value.set_name(name);
    }

    pub fn get_type(&self) -> VectorType<'ctx> {
        VectorType::new(self.vec_value.get_type())
    }

    pub fn is_null(&self) -> bool {
        self.vec_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.vec_value.is_undef()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue<'ctx>> {
        self.vec_value.as_instruction()
    }

    pub fn const_extract_element(&self, index: IntValue<'ctx>) -> BasicValueEnum<'ctx> {
        let value = unsafe {
            LLVMConstExtractElement(self.as_value_ref(), index.as_value_ref())
        };

        BasicValueEnum::new(value)
    }

    // SubTypes: value should really be T in self: VectorValue<T> I think
    pub fn const_insert_element<BV: BasicValue<'ctx>>(&self, index: IntValue<'ctx>, value: BV) -> BasicValueEnum<'ctx> {
        let value = unsafe {
            LLVMConstInsertElement(self.as_value_ref(), value.as_value_ref(), index.as_value_ref())
        };

        BasicValueEnum::new(value)
    }

    pub fn replace_all_uses_with(&self, other: VectorValue<'ctx>) {
        self.vec_value.replace_all_uses_with(other.as_value_ref())
    }

    /// Creates a const string which may be null terminated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let string = context.const_string(b"my_string", false);
    ///
    /// assert!(string.is_const_string());
    /// ```
    // SubTypes: Impl only for VectorValue<IntValue<i8>>
    pub fn is_const_string(&self) -> bool {
        unsafe {
            LLVMIsConstantString(self.as_value_ref()) == 1
        }
    }

    // SubTypes: Impl only for VectorValue<IntValue<i8>>
    pub fn get_string_constant(&self) -> &CStr {
        // REVIEW: Maybe need to check is_const_string?

        let mut len = 0;
        let ptr = unsafe {
            LLVMGetAsString(self.as_value_ref(), &mut len)
        };

        unsafe {
            CStr::from_ptr(ptr)
        }
    }

    // TODOC: Value seems to be zero initialized if index out of bounds
    // SubType: VectorValue<BV> -> BV
    pub fn get_element_as_constant(&self, index: u32) -> BasicValueEnum<'ctx> {
        let ptr = unsafe {
            LLVMGetElementAsConstant(self.as_value_ref(), index)
        };

        BasicValueEnum::new(ptr)
    }

    // SubTypes: self can only be VectoValue<IntValue<bool>>
    pub fn const_select<BV: BasicValue<'ctx>>(&self, then: BV, else_: BV) -> BasicValueEnum<'ctx> {
        let value = unsafe {
            LLVMConstSelect(self.as_value_ref(), then.as_value_ref(), else_.as_value_ref())
        };

        BasicValueEnum::new(value)
    }

    // SubTypes: <V: VectorValue<T, Const>> self: V, right: V, mask: V -> V
    pub fn const_shuffle_vector(&self, right: VectorValue<'ctx>, mask: VectorValue<'ctx>) -> VectorValue<'ctx> {
        let value = unsafe {
            LLVMConstShuffleVector(self.as_value_ref(), right.as_value_ref(), mask.as_value_ref())
        };

        VectorValue::new(value)
    }
}

impl AsValueRef for VectorValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.vec_value.value
    }
}
