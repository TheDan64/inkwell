use llvm_sys::core::{LLVMConstGEP, LLVMConstInBoundsGEP, LLVMConstPtrToInt, LLVMConstPointerCast, LLVMConstAddrSpaceCast};
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use support::LLVMString;
use types::{AsTypeRef, IntType, PointerType};
use values::{AsValueRef, InstructionValue, IntValue, Value, MetadataValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PointerValue {
    ptr_value: Value,
}

impl PointerValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        PointerValue {
            ptr_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.ptr_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.ptr_value.set_name(name);
    }

    pub fn get_type(&self) -> PointerType {
        PointerType::new(self.ptr_value.get_type())
    }

    pub fn is_null(&self) -> bool {
        self.ptr_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.ptr_value.is_undef()
    }

    /// Determines whether or not a `PointerValue` is a constant.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let void_type = context.void_type();
    /// let void_ptr_null = void_type.ptr_type(AddressSpace::Generic).const_null();
    ///
    /// assert!(void_ptr_null.is_const());
    /// ```
    pub fn is_const(&self) -> bool {
        self.ptr_value.is_const()
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.ptr_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.ptr_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.ptr_value.as_instruction()
    }

    pub fn has_metadata(&self) -> bool {
        self.ptr_value.has_metadata()
    }

    pub fn get_metadata(&self, kind_id: u32) -> Option<MetadataValue> {
        self.ptr_value.get_metadata(kind_id)
    }

    pub fn set_metadata(&self, metadata: MetadataValue, kind_id: u32) {
        self.ptr_value.set_metadata(metadata, kind_id)
    }

    // REVIEW: Should this be on array value too?
    /// GEP is very likely to segfault if indexes are used incorrectly, and is therefore an unsafe function. Maybe we can change this in the future.
    pub unsafe fn const_gep(&self, ordered_indexes: &[IntValue]) -> PointerValue {
        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter()
                                                                 .map(|val| val.as_value_ref())
                                                                 .collect();
        let value = {
            LLVMConstGEP(self.as_value_ref(), index_values.as_mut_ptr(), index_values.len() as u32)
        };

        PointerValue::new(value)
    }

    /// GEP is very likely to segfault if indexes are used incorrectly, and is therefore an unsafe function. Maybe we can change this in the future.
    pub unsafe fn const_in_bounds_gep(&self, ordered_indexes: &[IntValue]) -> PointerValue {
        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter()
                                                                 .map(|val| val.as_value_ref())
                                                                 .collect();
        let value = {
            LLVMConstInBoundsGEP(self.as_value_ref(), index_values.as_mut_ptr(), index_values.len() as u32)
        };

        PointerValue::new(value)
    }

    pub fn const_to_int(&self, int_type: IntType) -> IntValue {
        let value = unsafe {
            LLVMConstPtrToInt(self.as_value_ref(), int_type.as_type_ref())
        };

        IntValue::new(value)
    }

    pub fn const_cast(&self, ptr_type: PointerType) -> PointerValue {
        let value = unsafe {
            LLVMConstPointerCast(self.as_value_ref(), ptr_type.as_type_ref())
        };

        PointerValue::new(value)
    }

    pub fn const_address_space_cast(&self, ptr_type: PointerType) -> PointerValue {
        let value = unsafe {
            LLVMConstAddrSpaceCast(self.as_value_ref(), ptr_type.as_type_ref())
        };

        PointerValue::new(value)
    }

    pub fn replace_all_uses_with(&self, other: PointerValue) {
        self.ptr_value.replace_all_uses_with(other.as_value_ref())
    }
}

impl AsValueRef for PointerValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.ptr_value.value
    }
}
