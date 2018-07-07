use llvm_sys::core::LLVMVoidType;
use llvm_sys::prelude::LLVMTypeRef;

use AddressSpace;
use context::ContextRef;
use support::LLVMString;
use types::traits::AsTypeRef;
use types::{Type, BasicType, FunctionType, PointerType};
use values::PointerValue;

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
    pub fn is_sized(&self) -> bool {
        self.void_type.is_sized()
    }

    pub fn get_context(&self) -> ContextRef {
        self.void_type.get_context()
    }

    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.void_type.ptr_type(address_space)
    }

    pub fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
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

    pub fn print_to_string(&self) -> LLVMString {
        self.void_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm5-0")))]
    pub fn print_to_stderr(&self) {
        self.void_type.print_to_stderr()
    }

    pub fn const_null_ptr(&self) -> PointerValue {
        self.void_type.const_null_ptr()
    }
}

impl AsTypeRef for VoidType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.void_type.type_
    }
}
