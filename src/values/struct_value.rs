use llvm_sys::core::{LLVMGetNumOperands, LLVMGetOperand, LLVMSetOperand};

use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;
use std::fmt::{self, Display};

use crate::types::StructType;
use crate::values::traits::AsValueRef;
use crate::values::{BasicValue, InstructionValue, Value};

use super::{AnyValue, BasicValueEnum};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct StructValue<'ctx> {
    struct_value: Value<'ctx>,
}

impl<'ctx> StructValue<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of type struct.
    pub unsafe fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        StructValue {
            struct_value: Value::new(value),
        }
    }

    /// Gets the value of a field belonging to this `StructValue`.
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    /// let i8_type = context.i8_type();
    /// let i8_val = i8_type.const_all_ones();
    /// let i32_val = i32_type.const_all_ones();
    /// let struct_type = context.struct_type(&[i8_type.into(), i32_type.into()], false);
    /// let struct_val = struct_type.const_named_struct(&[i8_val.into(), i32_val.into()]);
    ///
    /// assert!(struct_val.get_field_at_index(0).is_some());
    /// assert!(struct_val.get_field_at_index(1).is_some());
    /// assert!(struct_val.get_field_at_index(3).is_none());
    /// assert!(struct_val.get_field_at_index(0).unwrap().is_int_value());
    /// ```
    pub fn get_field_at_index(self, index: u32) -> Option<BasicValueEnum<'ctx>> {
        // OoB indexing seems to be unchecked and therefore is UB
        if index >= self.count_fields() {
            return None;
        }

        Some(unsafe { self.get_field_at_index_unchecked(index) })
    }

    /// Gets the value of a field belonging to this `StructValue`.
    ///
    /// # Safety
    ///
    /// The index must be smaller than [StructValue::count_fields].
    pub unsafe fn get_field_at_index_unchecked(self, index: u32) -> BasicValueEnum<'ctx> {
        unsafe { BasicValueEnum::new(LLVMGetOperand(self.as_value_ref(), index)) }
    }

    /// Get a field value iterator.
    pub fn get_fields(self) -> FieldValueIter<'ctx> {
        FieldValueIter {
            sv: self,
            i: 0,
            count: self.count_fields(),
        }
    }

    /// Sets the value of a field belonging to this `StructValue`.
    pub fn set_field_at_index<BV: BasicValue<'ctx>>(self, index: u32, val: BV) -> bool {
        if self
            .get_type()
            .get_field_type_at_index(index)
            .map(|t| t == val.as_basic_value_enum().get_type())
            != Some(true)
        {
            return false;
        }

        unsafe { LLVMSetOperand(self.as_value_ref(), index, val.as_value_ref()) }

        true
    }

    /// Counts the number of fields.
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    /// let i8_type = context.i8_type();
    /// let i8_val = i8_type.const_all_ones();
    /// let i32_val = i32_type.const_all_ones();
    /// let struct_type = context.struct_type(&[i8_type.into(), i32_type.into()], false);
    /// let struct_val = struct_type.const_named_struct(&[i8_val.into(), i32_val.into()]);
    ///
    /// assert_eq!(struct_val.count_fields(), 2);
    /// assert_eq!(struct_val.count_fields(), struct_type.count_fields());
    /// ```
    pub fn count_fields(self) -> u32 {
        unsafe { LLVMGetNumOperands(self.as_value_ref()) as u32 }
    }

    /// Gets the name of a `StructValue`. If the value is a constant, this will
    /// return an empty string.
    pub fn get_name(&self) -> &CStr {
        self.struct_value.get_name()
    }

    /// Get name of the `StructValue`.
    pub fn set_name(&self, name: &str) {
        self.struct_value.set_name(name)
    }

    pub fn get_type(self) -> StructType<'ctx> {
        unsafe { StructType::new(self.struct_value.get_type()) }
    }

    pub fn is_null(self) -> bool {
        self.struct_value.is_null()
    }

    pub fn is_undef(self) -> bool {
        self.struct_value.is_undef()
    }

    pub fn print_to_stderr(self) {
        self.struct_value.print_to_stderr()
    }

    pub fn as_instruction(self) -> Option<InstructionValue<'ctx>> {
        self.struct_value.as_instruction()
    }

    pub fn replace_all_uses_with(self, other: StructValue<'ctx>) {
        self.struct_value.replace_all_uses_with(other.as_value_ref())
    }

    /// Determines whether or not a `StructValue` is a constant.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::{context::Context, values::BasicValue};
    ///
    /// let context = Context::create();
    /// let i64_type = context.i64_type();
    /// let i64_val = i64_type.const_int(23, false).as_basic_value_enum();
    /// let struct_val = context.const_struct(&[i64_val, i64_val], false);
    ///
    /// assert!(struct_val.is_const());
    /// ```
    pub fn is_const(self) -> bool {
        self.struct_value.is_const()
    }
}

unsafe impl AsValueRef for StructValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.struct_value.value
    }
}

impl Display for StructValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

/// Iterate over all the field values of this struct.
#[derive(Debug)]
pub struct FieldValueIter<'ctx> {
    sv: StructValue<'ctx>,
    i: u32,
    count: u32,
}

impl<'ctx> Iterator for FieldValueIter<'ctx> {
    type Item = BasicValueEnum<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.count {
            let result = unsafe { self.sv.get_field_at_index_unchecked(self.i) };
            self.i += 1;
            Some(result)
        } else {
            None
        }
    }
}
