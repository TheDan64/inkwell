//! A value is an instance of a type.

#[deny(missing_docs)]
mod array_value;
#[deny(missing_docs)]
mod basic_value_use;
#[deny(missing_docs)]
mod call_site_value;
mod enums;
mod float_value;
mod fn_value;
mod generic_value;
mod global_value;
mod instruction_value;
mod int_value;
mod metadata_value;
mod phi_value;
mod ptr_value;
mod struct_value;
mod traits;
mod vec_value;

#[cfg(not(any(
    feature = "llvm15-0",
    feature = "llvm16-0",
    feature = "llvm17-0",
    feature = "llvm18-0"
)))]
mod callable_value;

#[cfg(not(any(
    feature = "llvm15-0",
    feature = "llvm16-0",
    feature = "llvm17-0",
    feature = "llvm18-0"
)))]
pub use crate::values::callable_value::CallableValue;

use crate::support::{to_c_str, LLVMString};
pub use crate::values::array_value::ArrayValue;
pub use crate::values::basic_value_use::BasicValueUse;
pub use crate::values::call_site_value::CallSiteValue;
pub use crate::values::enums::{AggregateValueEnum, AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum};
pub use crate::values::float_value::FloatValue;
pub use crate::values::fn_value::FunctionValue;
pub use crate::values::generic_value::GenericValue;
pub use crate::values::global_value::GlobalValue;
#[llvm_versions(7..)]
pub use crate::values::global_value::UnnamedAddress;
pub use crate::values::instruction_value::{InstructionOpcode, InstructionValue, OperandIter, OperandUseIter};
pub use crate::values::int_value::IntValue;
pub use crate::values::metadata_value::{MetadataValue, FIRST_CUSTOM_METADATA_KIND_ID};
pub use crate::values::phi_value::IncomingIter;
pub use crate::values::phi_value::PhiValue;
pub use crate::values::ptr_value::PointerValue;
pub use crate::values::struct_value::FieldValueIter;
pub use crate::values::struct_value::StructValue;
pub use crate::values::traits::AsValueRef;
pub use crate::values::traits::{AggregateValue, AnyValue, BasicValue, FloatMathValue, IntMathValue, PointerMathValue};
pub use crate::values::vec_value::VectorValue;

#[llvm_versions(18..)]
pub use llvm_sys::LLVMTailCallKind;

use llvm_sys::core::{
    LLVMDumpValue, LLVMGetFirstUse, LLVMGetSection, LLVMIsAInstruction, LLVMIsConstant, LLVMIsNull, LLVMIsUndef,
    LLVMPrintTypeToString, LLVMPrintValueToString, LLVMReplaceAllUsesWith, LLVMSetSection, LLVMTypeOf,
};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
struct Value<'ctx> {
    value: LLVMValueRef,
    _marker: PhantomData<&'ctx ()>,
}

impl<'ctx> Value<'ctx> {
    pub(crate) unsafe fn new(value: LLVMValueRef) -> Self {
        debug_assert!(
            !value.is_null(),
            "This should never happen since containing struct should check null ptrs"
        );

        Value {
            value,
            _marker: PhantomData,
        }
    }

    fn is_instruction(self) -> bool {
        unsafe { !LLVMIsAInstruction(self.value).is_null() }
    }

    fn as_instruction(self) -> Option<InstructionValue<'ctx>> {
        if !self.is_instruction() {
            return None;
        }

        unsafe { Some(InstructionValue::new(self.value)) }
    }

    fn is_null(self) -> bool {
        unsafe { LLVMIsNull(self.value) == 1 }
    }

    fn is_const(self) -> bool {
        unsafe { LLVMIsConstant(self.value) == 1 }
    }

    // TODOC: According to https://stackoverflow.com/questions/21593752/llvm-how-to-pass-a-name-to-constantint
    // you can't use set_name name on a constant(by can't, I mean it wont do anything), unless it's also a global.
    // So, you can set names on variables (ie a function parameter)
    // REVIEW: It'd be great if we could encode this into the type system somehow. For example,
    // add a ParamValue wrapper type that always have it but conditional types (IntValue<Variable>)
    // that also have it. This isn't a huge deal though, since it hasn't proven to be UB so far
    fn set_name(self, name: &str) {
        let c_string = to_c_str(name);

        #[cfg(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0"))]
        {
            use llvm_sys::core::LLVMSetValueName;

            unsafe {
                LLVMSetValueName(self.value, c_string.as_ptr());
            }
        }
        #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
        {
            use llvm_sys::core::LLVMSetValueName2;

            unsafe { LLVMSetValueName2(self.value, c_string.as_ptr(), name.len()) }
        }
    }

    // get_name should *not* return a LLVMString, because it is not an owned value AFAICT
    // TODO: Should make this take ownership of self. But what is the lifetime of the string? 'ctx?
    fn get_name(&self) -> &CStr {
        #[cfg(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0"))]
        let ptr = unsafe {
            use llvm_sys::core::LLVMGetValueName;

            LLVMGetValueName(self.value)
        };
        #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
        let ptr = unsafe {
            use llvm_sys::core::LLVMGetValueName2;
            let mut len = 0;

            LLVMGetValueName2(self.value, &mut len)
        };

        unsafe { CStr::from_ptr(ptr) }
    }

    fn is_undef(self) -> bool {
        unsafe { LLVMIsUndef(self.value) == 1 }
    }

    fn get_type(self) -> LLVMTypeRef {
        unsafe { LLVMTypeOf(self.value) }
    }

    fn print_to_string(self) -> LLVMString {
        unsafe { LLVMString::new(LLVMPrintValueToString(self.value)) }
    }

    fn print_to_stderr(self) {
        unsafe { LLVMDumpValue(self.value) }
    }

    // REVIEW: I think this is memory safe, though it may result in an IR error
    // if used incorrectly, which is OK.
    fn replace_all_uses_with(self, other: LLVMValueRef) {
        // LLVM may infinite-loop when they aren't distinct, which is UB in C++.
        if self.value != other {
            unsafe { LLVMReplaceAllUsesWith(self.value, other) }
        }
    }

    pub fn get_first_use(self) -> Option<BasicValueUse<'ctx>> {
        let use_ = unsafe { LLVMGetFirstUse(self.value) };

        if use_.is_null() {
            return None;
        }

        unsafe { Some(BasicValueUse::new(use_)) }
    }

    /// Gets the section of the global value
    pub fn get_section(&self) -> Option<&CStr> {
        let ptr = unsafe { LLVMGetSection(self.value) };

        if ptr.is_null() {
            return None;
        }

        // On MacOS we need to remove ',' before section name
        if cfg!(target_os = "macos") {
            let name = unsafe { CStr::from_ptr(ptr) };
            let name_string = name.to_string_lossy();
            let mut chars = name_string.chars();
            if Some(',') == chars.next() {
                Some(unsafe { CStr::from_ptr(ptr.add(1)) })
            } else {
                Some(name)
            }
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }

    /// Sets the section of the global value
    fn set_section(self, section: Option<&str>) {
        #[cfg(target_os = "macos")]
        let mapped_section = section.map(|s| {
            if s.contains(",") {
                format!("{}", s)
            } else {
                format!(",{}", s)
            }
        });
        #[cfg(target_os = "macos")]
        let section = mapped_section.as_deref();

        let c_string = section.map(to_c_str);

        unsafe {
            LLVMSetSection(
                self.value,
                // The as_ref call is important here so that we don't drop the cstr mid use
                c_string.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
            )
        }
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = self.print_to_string();
        let llvm_type = unsafe { CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.value))) };
        let name = self.get_name();
        let is_const = self.is_const();
        let is_null = self.is_null();
        let is_undef = self.is_undef();

        f.debug_struct("Value")
            .field("name", &name)
            .field("address", &self.value)
            .field("is_const", &is_const)
            .field("is_null", &is_null)
            .field("is_undef", &is_undef)
            .field("llvm_value", &llvm_value)
            .field("llvm_type", &llvm_type)
            .finish()
    }
}
