use llvm_sys::core::{LLVMGetParamTypes, LLVMIsFunctionVarArg, LLVMCountParamTypes};
use llvm_sys::prelude::LLVMTypeRef;

use std::fmt;
use std::mem::forget;

use AddressSpace;
use context::ContextRef;
use support::LLVMString;
use types::traits::AsTypeRef;
use types::{PointerType, Type, BasicTypeEnum};

// REVIEW: Add a get_return_type() -> Option<BasicTypeEnum>?
/// A `FunctionType` is the type of a function variable.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FunctionType {
    fn_type: Type,
}

impl FunctionType {
    pub(crate) fn new(fn_type: LLVMTypeRef) -> FunctionType {
        assert!(!fn_type.is_null());

        FunctionType {
            fn_type: Type::new(fn_type)
        }
    }

    /// Creates a `PointerType` with this `FunctionType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let fn_type = f32_type.fn_type(&[], false);
    /// let fn_ptr_type = fn_type.ptr_type(AddressSpace::Global);
    ///
    /// assert_eq!(fn_ptr_type.get_element_type().into_function_type(), fn_type);
    /// ```
    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.fn_type.ptr_type(address_space)
    }

    /// Determines whether or not a `FunctionType` is a variadic function.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let fn_type = f32_type.fn_type(&[], true);
    ///
    /// assert!(fn_type.is_var_arg());
    /// ```
    pub fn is_var_arg(&self) -> bool {
        unsafe {
            LLVMIsFunctionVarArg(self.as_type_ref()) != 0
        }
    }

    /// Gets param types this `FunctionType` has.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let fn_type = f32_type.fn_type(&[f32_type.into()], true);
    /// let param_types = fn_type.get_param_types();
    ///
    /// assert_eq!(param_types.len(), 1);
    /// assert_eq!(param_types[0].into_float_type(), f32_type);
    /// ```
    pub fn get_param_types(&self) -> Vec<BasicTypeEnum> {
        let count = self.count_param_types();
        let mut raw_vec: Vec<LLVMTypeRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetParamTypes(self.as_type_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| BasicTypeEnum::new(*val)).collect()
    }

    /// Counts the number of param types this `FunctionType` has.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let fn_type = f32_type.fn_type(&[f32_type.into()], true);
    ///
    /// assert_eq!(fn_type.count_param_types(), 1);
    /// ```
    pub fn count_param_types(&self) -> u32 {
        unsafe {
            LLVMCountParamTypes(self.as_type_ref())
        }
    }

    // REVIEW: Always false -> const fn?
    /// Gets whether or not this `FunctionType` is sized or not. This is likely
    /// always false and may be removed in the future.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let fn_type = f32_type.fn_type(&[], true);
    ///
    /// assert!(!fn_type.is_sized());
    /// ```
    pub fn is_sized(&self) -> bool {
        self.fn_type.is_sized()
    }

    // REVIEW: Does this work on functions?
    // fn get_alignment(&self) -> IntValue {
    //     self.fn_type.get_alignment()
    // }

    /// Gets a reference to the `Context` this `FunctionType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let fn_type = f32_type.fn_type(&[], true);
    ///
    /// assert_eq!(*fn_type.get_context(), context);
    /// ```
    pub fn get_context(&self) -> ContextRef {
        self.fn_type.get_context()
    }

    /// Prints the definition of a `FunctionType` to a `LLVMString`.
    pub fn print_to_string(&self) -> LLVMString {
        self.fn_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of an `IntType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7 => 4.0)]
    pub fn print_to_stderr(&self) {
        self.fn_type.print_to_stderr()
    }

    // REVIEW: Can you do undef for functions?
    // Seems to "work" - no UB or SF so far but fails
    // LLVMIsAFunction() check. Commenting out for further research
    // pub fn get_undef(&self) -> FunctionValue {
    //     FunctionValue::new(self.fn_type.get_undef()).expect("Should always get an undef value")
    // }
}

impl fmt::Debug for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = self.print_to_string();

        f.debug_struct("FunctionType")
            .field("address", &self.as_type_ref())
            .field("is_var_args", &self.is_var_arg())
            .field("llvm_type", &llvm_type)
            .finish()
    }
}

impl AsTypeRef for FunctionType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.fn_type.type_
    }
}
