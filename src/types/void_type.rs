use llvm_sys::core::LLVMVoidType;
use llvm_sys::prelude::LLVMTypeRef;

use AddressSpace;
use context::ContextRef;
use support::LLVMString;
use types::traits::AsTypeRef;
use types::{Type, BasicTypeEnum, FunctionType, PointerType};

/// A `VoidType` is a special type with no possible direct instances. It's particularly
/// useful as a pointer element type or a function return type.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VoidType {
    void_type: Type,
}

impl VoidType {
    pub(crate) fn new(void_type: LLVMTypeRef) -> Self {
        assert!(!void_type.is_null());

        VoidType {
            void_type: Type::new(void_type),
        }
    }

    // REVIEW: Always false -> const fn?
    /// Gets whether or not this `VectorType` is sized or not. This may always
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
    pub fn is_sized(&self) -> bool {
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
    /// assert_eq!(*void_type.get_context(), context);
    /// ```
    pub fn get_context(&self) -> ContextRef {
        self.void_type.get_context()
    }

    /// Creates a `PointerType` with this `VoidType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let void_type = context.void_type();
    /// let void_ptr_type = void_type.ptr_type(AddressSpace::Generic);
    ///
    /// assert_eq!(void_ptr_type.get_element_type().into_void_type(), void_type);
    /// ```
    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.void_type.ptr_type(address_space)
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
    pub fn fn_type(&self, param_types: &[BasicTypeEnum], is_var_args: bool) -> FunctionType {
        self.void_type.fn_type(param_types, is_var_args)
    }

    /// Gets the `VoidType`. It will be assigned the global context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::VoidType;
    ///
    /// let void_type = VoidType::void_type();
    ///
    /// assert_eq!(void_type.get_context(), Context::get_global());
    /// ```
    pub fn void_type() -> Self {
        let void_type = unsafe {
            LLVMVoidType()
        };

        VoidType::new(void_type)
    }

    /// Prints the definition of a `VoidType` to a `LLVMString`.
    pub fn print_to_string(&self) -> LLVMString {
        self.void_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of a `VoidType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7 => 4.0)]
    pub fn print_to_stderr(&self) {
        self.void_type.print_to_stderr()
    }
}

impl AsTypeRef for VoidType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.void_type.type_
    }
}
