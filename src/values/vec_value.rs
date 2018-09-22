use llvm_sys::core::{LLVMIsAConstantVector, LLVMIsAConstantDataVector, LLVMConstInsertElement, LLVMConstExtractElement, LLVMIsConstantString, LLVMConstString, LLVMGetElementAsConstant, LLVMGetAsString, LLVMConstSelect, LLVMConstShuffleVector};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use support::LLVMString;
use types::{VectorType};
use values::traits::AsValueRef;
use values::{BasicValueEnum, BasicValue, InstructionValue, Value, IntValue, MetadataValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VectorValue {
    vec_value: Value,
}

impl VectorValue {
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
    /// let i8_vec_null = i8_vec_type.const_null();
    ///
    /// assert!(i8_vec_null.is_const());
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

    pub fn get_type(&self) -> VectorType {
        VectorType::new(self.vec_value.get_type())
    }

    pub fn is_null(&self) -> bool {
        self.vec_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.vec_value.is_undef()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.vec_value.as_instruction()
    }

    pub fn const_extract_element(&self, index: IntValue) -> BasicValueEnum {
        let value = unsafe {
            LLVMConstExtractElement(self.as_value_ref(), index.as_value_ref())
        };

        BasicValueEnum::new(value)
    }

    // SubTypes: value should really be T in self: VectorValue<T> I think
    pub fn const_insert_element<BV: BasicValue>(&self, index: IntValue, value: BV) -> BasicValueEnum {
        let value = unsafe {
            LLVMConstInsertElement(self.as_value_ref(), value.as_value_ref(), index.as_value_ref())
        };

        BasicValueEnum::new(value)
    }

    pub fn has_metadata(&self) -> bool {
        self.vec_value.has_metadata()
    }

    pub fn get_metadata(&self, kind_id: u32) -> Option<MetadataValue> {
        self.vec_value.get_metadata(kind_id)
    }

    pub fn set_metadata(&self, metadata: MetadataValue, kind_id: u32) {
        self.vec_value.set_metadata(metadata, kind_id)
    }

    pub fn replace_all_uses_with(&self, other: VectorValue) {
        self.vec_value.replace_all_uses_with(other.as_value_ref())
    }

    /// Creates a const string which may be null terminated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::values::VectorValue;
    ///
    /// let string = VectorValue::const_string("my_string", false);
    ///
    /// assert_eq!(string.print_to_string().to_string(), "[9 x i8] c\"my_string\"");
    /// ```
    // SubTypes: Should return VectorValue<IntValue<i8>>
    pub fn const_string(string: &str, null_terminated: bool) -> Self {
        let ptr = unsafe {
            LLVMConstString(string.as_ptr() as *const i8, string.len() as u32, !null_terminated as i32)
        };

        VectorValue::new(ptr)
    }

    /// Creates a const string which may be null terminated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::values::VectorValue;
    ///
    /// let string = VectorValue::const_string("my_string", false);
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
    pub fn get_element_as_constant(&self, index: u32) -> BasicValueEnum {
        let ptr = unsafe {
            LLVMGetElementAsConstant(self.as_value_ref(), index)
        };

        BasicValueEnum::new(ptr)
    }

    // SubTypes: self can only be VectoValue<IntValue<bool>>
    pub fn const_select<BV: BasicValue>(&self, then: BV, else_: BV) -> BasicValueEnum {
        let value = unsafe {
            LLVMConstSelect(self.as_value_ref(), then.as_value_ref(), else_.as_value_ref())
        };

        BasicValueEnum::new(value)
    }

    // SubTypes: <V: VectorValue<T, Const>> self: V, right: V, mask: V -> V
    pub fn const_shuffle_vector(&self, right: VectorValue, mask: VectorValue) -> VectorValue {
        let value = unsafe {
            LLVMConstShuffleVector(self.as_value_ref(), right.as_value_ref(), mask.as_value_ref())
        };

        VectorValue::new(value)
    }
}

impl AsValueRef for VectorValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.vec_value.value
    }
}
