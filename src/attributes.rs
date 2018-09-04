//! `Attribute`s are optional modifiers to functions, function parameters, and return types.

use llvm_sys::prelude::LLVMAttributeRef;
use llvm_sys::core::{LLVMGetEnumAttributeKindForName, LLVMGetLastEnumAttributeKind, LLVMGetEnumAttributeKind, LLVMGetEnumAttributeValue, LLVMGetStringAttributeKind, LLVMGetStringAttributeValue, LLVMIsEnumAttribute, LLVMIsStringAttribute};

use std::ffi::CStr;

// SubTypes: Attribute<Enum>, Attribute<String>
/// Functions, function parameters, and return types can have `Attribute`s to indicate
/// how they should be treated by optimizations and code generation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attribute {
    pub(crate) attribute: LLVMAttributeRef,
}

impl Attribute {
    pub(crate) fn new(attribute: LLVMAttributeRef) -> Self {
        debug_assert!(!attribute.is_null());

        Attribute {
            attribute,
        }
    }

    /// Determines whether or not an `Attribute` is an enum. This method will
    /// likely be removed in the future in favor of `Attribute`s being generically
    /// defined.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let enum_attribute = context.create_enum_attribute(0, 10);
    ///
    /// assert!(enum_attribute.is_enum());
    /// ```
    pub fn is_enum(&self) -> bool {
        unsafe {
            LLVMIsEnumAttribute(self.attribute) == 1
        }
    }

    /// Determines whether or not an `Attribute` is a string. This method will
    /// likely be removed in the future in favor of `Attribute`s being generically
    /// defined.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let string_attribute = context.create_string_attribute("my_key_123", "my_val");
    ///
    /// assert!(string_attribute.is_string());
    /// ```
    pub fn is_string(&self) -> bool {
        unsafe {
            LLVMIsStringAttribute(self.attribute) == 1
        }
    }

    /// Gets the enum kind id associated with a builtin name.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::Attribute;
    ///
    /// // This kind id doesn't exist:
    /// assert_eq!(Attribute::get_named_enum_kind_id("foobar"), 0);
    ///
    /// // These are real kind ids:
    /// assert_eq!(Attribute::get_named_enum_kind_id("align"), 1);
    /// assert_eq!(Attribute::get_named_enum_kind_id("builtin"), 5);
    /// ```
    pub fn get_named_enum_kind_id(name: &str) -> u32 {
        unsafe {
            LLVMGetEnumAttributeKindForName(name.as_ptr() as *const i8, name.len())
        }
    }

    /// Gets the kind id associated with an enum `Attribute`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let enum_attribute = context.create_enum_attribute(0, 10);
    ///
    /// assert_eq!(enum_attribute.get_enum_kind_id(), 0);
    /// ```
    pub fn get_enum_kind_id(&self) -> u32 {
        assert!(self.is_enum()); // FIXME: SubTypes

        unsafe {
            LLVMGetEnumAttributeKind(self.attribute)
        }
    }

    /// Gets the last enum kind id associated with builtin names.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::Attribute;
    ///
    /// assert_eq!(Attribute::get_last_enum_kind_id(), 56);
    /// ```
    pub fn get_last_enum_kind_id() -> u32 {
        unsafe {
            LLVMGetLastEnumAttributeKind()
        }
    }

    /// Gets the value associated with an enum `Attribute`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let enum_attribute = context.create_enum_attribute(0, 10);
    ///
    /// assert_eq!(enum_attribute.get_enum_value(), 10);
    /// ```
    pub fn get_enum_value(&self) -> u64 {
        assert!(self.is_enum()); // FIXME: SubTypes

        unsafe {
            LLVMGetEnumAttributeValue(self.attribute)
        }
    }

    /// Gets the string kind id associated with a string attribute.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use std::ffi::CString;
    ///
    /// let context = Context::create();
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    ///
    /// assert_eq!(*string_attribute.get_string_kind_id(), *CString::new("my_key").unwrap());
    /// ```
    pub fn get_string_kind_id(&self) -> &CStr {
        assert!(self.is_string()); // FIXME: SubTypes

        let mut length = 0;
        let cstr_ptr = unsafe {
            LLVMGetStringAttributeKind(self.attribute, &mut length)
        };

        unsafe {
            CStr::from_ptr(cstr_ptr)
        }
    }

    /// Gets the string value associated with a string attribute.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use std::ffi::CString;
    ///
    /// let context = Context::create();
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    ///
    /// assert_eq!(*string_attribute.get_string_value(), *CString::new("my_val").unwrap());
    /// ```
    pub fn get_string_value(&self) -> &CStr {
        assert!(self.is_string()); // FIXME: SubTypes

        let mut length = 0;
        let cstr_ptr = unsafe {
            LLVMGetStringAttributeValue(self.attribute, &mut length)
        };

        unsafe {
            CStr::from_ptr(cstr_ptr)
        }
    }
}
