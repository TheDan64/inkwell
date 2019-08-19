use llvm_sys::prelude::LLVMTypeRef;

use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::types::traits::AsTypeRef;
use crate::types::{Type, BasicTypeEnum, FunctionType};

/// The `TokenType` is used when a value is associated with an instruction but all
/// uses of the value must not attempt to introspect or obscure it. As such, it is
/// not appropriate to have a phi or select of type token.
///
/// It is only possible to construct this type through a `Context`
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TokenType<'ctx> {
    token_type: Type<'ctx>,
}

impl<'ctx> TokenType<'ctx> {
    pub(crate) fn new(token_type: LLVMTypeRef) -> Self {
        assert!(!token_type.is_null());

        TokenType {
            token_type: Type::new(token_type),
        }
    }

    /// Gets a reference to the `Context` this `TokenType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let token_type = context.token_type();
    ///
    /// assert_eq!(*token_type.get_context(), context);
    /// ```
    pub fn get_context(&self) -> ContextRef<'ctx> {
        self.token_type.get_context()
    }

    /// Creates a `FunctionType` with this `TokenType` for its return type.
    ///
    /// This is typically used with LLVM intrinsics
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let token_type = context.token_type();
    /// let fn_type = token_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(&self, param_types: &[BasicTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.token_type.fn_type(param_types, is_var_args)
    }

    /// Prints the definition of a `TokenType` to a `LLVMString`.
    pub fn print_to_string(&self) -> LLVMString {
        self.token_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of a `TokenType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7..=4.0)]
    pub fn print_to_stderr(&self) {
        self.token_type.print_to_stderr()
    }
}

impl AsTypeRef for TokenType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.token_type.ty
    }
}
