use llvm_sys::prelude::LLVMTypeRef;

use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::types::enums::BasicMetadataTypeEnum;
use crate::types::traits::AsTypeRef;
use crate::types::{FunctionType, Type};

use std::fmt::{self, Display};

/// A `VoidType` is a special type with no possible direct instances. It's only
/// useful as a function return type.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VoidType<'ctx> {
    void_type: Type<'ctx>,
}

impl<'ctx> VoidType<'ctx> {
    /// Create `VoidType` from [`LLVMTypeRef`]
    ///
    /// # Safety
    /// Undefined behavior, if referenced type isn't void type
    pub unsafe fn new(void_type: LLVMTypeRef) -> Self {
        assert!(!void_type.is_null());

        VoidType {
            void_type: Type::new(void_type),
        }
    }

    // REVIEW: Always false -> const fn?
    /// Gets whether or not this `VoidType` is sized or not. This may always
    /// be false and as such this function may be removed in the future.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let void_type = context.void_type();
    ///
    /// assert!(void_type.is_sized());
    /// ```
    pub fn is_sized(self) -> bool {
        self.void_type.is_sized()
    }

    /// Gets a reference to the `Context` this `VoidType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let void_type = context.void_type();
    ///
    /// assert_eq!(void_type.get_context(), context);
    /// ```
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.void_type.get_context()
    }

    /// Creates a `FunctionType` with this `VoidType` for its return type.
    /// This means the function does not return.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.void_type.fn_type(param_types, is_var_args)
    }

    /// Print the definition of a `VoidType` to `LLVMString`.
    pub fn print_to_string(self) -> LLVMString {
        self.void_type.print_to_string()
    }
}

unsafe impl AsTypeRef for VoidType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.void_type.ty
    }
}

impl Display for VoidType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
