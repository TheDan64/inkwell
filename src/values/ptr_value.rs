#[llvm_versions(..=14)]
use llvm_sys::core::{LLVMConstGEP, LLVMConstInBoundsGEP};
#[llvm_versions(15..)]
use llvm_sys::core::{LLVMConstGEP2, LLVMConstInBoundsGEP2};

use llvm_sys::core::{LLVMConstAddrSpaceCast, LLVMConstPointerCast, LLVMConstPtrToInt};
use llvm_sys::prelude::LLVMValueRef;

use std::convert::TryFrom;
use std::ffi::CStr;
use std::fmt::{self, Display};

use crate::types::{AsTypeRef, BasicType, IntType, PointerType};
use crate::values::{AsValueRef, InstructionValue, IntValue, Value};

use super::AnyValue;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PointerValue<'ctx> {
    ptr_value: Value<'ctx>,
}

impl<'ctx> PointerValue<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of type pointer.
    pub unsafe fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        PointerValue {
            ptr_value: Value::new(value),
        }
    }

    /// Get name of the `PointerValue`. If the value is a constant, this
    /// will return an empty string.
    pub fn get_name(&self) -> &CStr {
        self.ptr_value.get_name()
    }

    /// Set name of the `PointerValue`.
    pub fn set_name(&self, name: &str) {
        self.ptr_value.set_name(name)
    }

    pub fn get_type(self) -> PointerType<'ctx> {
        unsafe { PointerType::new(self.ptr_value.get_type()) }
    }

    pub fn is_null(self) -> bool {
        self.ptr_value.is_null()
    }

    pub fn is_undef(self) -> bool {
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
    /// ```
    pub fn is_const(self) -> bool {
        self.ptr_value.is_const()
    }

    pub fn print_to_stderr(self) {
        self.ptr_value.print_to_stderr()
    }

    pub fn as_instruction(self) -> Option<InstructionValue<'ctx>> {
        self.ptr_value.as_instruction()
    }

    // REVIEW: Should this be on array value too?
    /// GEP is very likely to segfault if indexes are used incorrectly, and is therefore an unsafe function. Maybe we can change this in the future.
    #[llvm_versions(..=14)]
    pub unsafe fn const_gep(self, ordered_indexes: &[IntValue<'ctx>]) -> PointerValue<'ctx> {
        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter().map(|val| val.as_value_ref()).collect();

        let value = {
            LLVMConstGEP(
                self.as_value_ref(),
                index_values.as_mut_ptr(),
                index_values.len() as u32,
            )
        };

        PointerValue::new(value)
    }

    // REVIEW: Should this be on array value too?
    /// GEP is very likely to segfault if indexes are used incorrectly, and is therefore an unsafe function. Maybe we can change this in the future.
    #[llvm_versions(15..)]
    pub unsafe fn const_gep<T: BasicType<'ctx>>(self, ty: T, ordered_indexes: &[IntValue<'ctx>]) -> PointerValue<'ctx> {
        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter().map(|val| val.as_value_ref()).collect();

        let value = {
            LLVMConstGEP2(
                ty.as_type_ref(),
                self.as_value_ref(),
                index_values.as_mut_ptr(),
                index_values.len() as u32,
            )
        };

        PointerValue::new(value)
    }

    /// GEP is very likely to segfault if indexes are used incorrectly, and is therefore an unsafe function. Maybe we can change this in the future.
    #[llvm_versions(..=14)]
    pub unsafe fn const_in_bounds_gep(self, ordered_indexes: &[IntValue<'ctx>]) -> PointerValue<'ctx> {
        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter().map(|val| val.as_value_ref()).collect();

        let value = {
            LLVMConstInBoundsGEP(
                self.as_value_ref(),
                index_values.as_mut_ptr(),
                index_values.len() as u32,
            )
        };

        PointerValue::new(value)
    }

    /// GEP is very likely to segfault if indexes are used incorrectly, and is therefore an unsafe function. Maybe we can change this in the future.
    #[llvm_versions(15..)]
    pub unsafe fn const_in_bounds_gep<T: BasicType<'ctx>>(
        self,
        ty: T,
        ordered_indexes: &[IntValue<'ctx>],
    ) -> PointerValue<'ctx> {
        let mut index_values: Vec<LLVMValueRef> = ordered_indexes.iter().map(|val| val.as_value_ref()).collect();

        let value = {
            LLVMConstInBoundsGEP2(
                ty.as_type_ref(),
                self.as_value_ref(),
                index_values.as_mut_ptr(),
                index_values.len() as u32,
            )
        };

        PointerValue::new(value)
    }
}

impl<'ctx> PointerValue<'ctx> {
    pub fn const_to_int(self, int_type: IntType<'ctx>) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMConstPtrToInt(self.as_value_ref(), int_type.as_type_ref())) }
    }

    pub fn const_cast(self, ptr_type: PointerType<'ctx>) -> PointerValue<'ctx> {
        unsafe { PointerValue::new(LLVMConstPointerCast(self.as_value_ref(), ptr_type.as_type_ref())) }
    }

    pub fn const_address_space_cast(self, ptr_type: PointerType<'ctx>) -> PointerValue<'ctx> {
        unsafe { PointerValue::new(LLVMConstAddrSpaceCast(self.as_value_ref(), ptr_type.as_type_ref())) }
    }

    pub fn replace_all_uses_with(self, other: PointerValue<'ctx>) {
        self.ptr_value.replace_all_uses_with(other.as_value_ref())
    }
}

unsafe impl AsValueRef for PointerValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.ptr_value.value
    }
}

impl Display for PointerValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl<'ctx> TryFrom<InstructionValue<'ctx>> for PointerValue<'ctx> {
    type Error = ();

    fn try_from(value: InstructionValue) -> Result<Self, Self::Error> {
        if value.get_type().is_pointer_type() {
            unsafe { Ok(PointerValue::new(value.as_value_ref())) }
        } else {
            Err(())
        }
    }
}
