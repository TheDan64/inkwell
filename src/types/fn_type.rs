use llvm_sys::core::{
    LLVMCountParamTypes, LLVMGetParamTypes, LLVMGetReturnType, LLVMGetTypeKind, LLVMIsFunctionVarArg,
};
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::LLVMTypeKind;

use std::fmt::{self, Display};
use std::mem::forget;

use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::types::traits::AsTypeRef;
use crate::types::{AnyType, BasicMetadataTypeEnum, BasicTypeEnum, PointerType, Type};
use crate::AddressSpace;

/// A `FunctionType` is the type of a function variable.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FunctionType<'ctx> {
    fn_type: Type<'ctx>,
}

impl<'ctx> FunctionType<'ctx> {
    /// Create `FunctionType` from [`LLVMTypeRef`]
    ///
    /// # Safety
    /// Undefined behavior, if referenced type isn't function type
    pub unsafe fn new(fn_type: LLVMTypeRef) -> Self {
        assert!(!fn_type.is_null());

        FunctionType {
            fn_type: Type::new(fn_type),
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
    /// let fn_ptr_type = fn_type.ptr_type(AddressSpace::default());
    ///
    /// #[cfg(feature = "typed-pointers")]
    /// assert_eq!(fn_ptr_type.get_element_type().into_function_type(), fn_type);
    /// ```
    #[cfg_attr(
        any(
            all(feature = "llvm15-0", not(feature = "typed-pointers")),
            all(feature = "llvm16-0", not(feature = "typed-pointers")),
            feature = "llvm17-0",
            feature = "llvm18-1"
        ),
        deprecated(
            note = "Starting from version 15.0, LLVM doesn't differentiate between pointer types. Use Context::ptr_type instead."
        )
    )]
    pub fn ptr_type(self, address_space: AddressSpace) -> PointerType<'ctx> {
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
    pub fn is_var_arg(self) -> bool {
        unsafe { LLVMIsFunctionVarArg(self.as_type_ref()) != 0 }
    }

    /// Gets param types this `FunctionType` has.
    ///
    /// # Example
    ///
    /// ```
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
    pub fn get_param_types(self) -> Vec<BasicMetadataTypeEnum<'ctx>> {
        let count = self.count_param_types();
        let mut raw_vec: Vec<LLVMTypeRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetParamTypes(self.as_type_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec
            .iter()
            .map(|val| unsafe { BasicMetadataTypeEnum::new(*val) })
            .collect()
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
    pub fn count_param_types(self) -> u32 {
        unsafe { LLVMCountParamTypes(self.as_type_ref()) }
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
    pub fn is_sized(self) -> bool {
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
    /// assert_eq!(fn_type.get_context(), context);
    /// ```
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.fn_type.get_context()
    }

    /// Print the definition of a `FunctionType` to `LLVMString`.
    pub fn print_to_string(self) -> LLVMString {
        self.fn_type.print_to_string()
    }

    /// Gets the return type of this `FunctionType`.
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
    /// assert_eq!(fn_type.get_return_type().unwrap().into_float_type(), f32_type);
    /// ```
    pub fn get_return_type(self) -> Option<BasicTypeEnum<'ctx>> {
        let ty = unsafe { LLVMGetReturnType(self.as_type_ref()) };

        let kind = unsafe { LLVMGetTypeKind(ty) };

        if let LLVMTypeKind::LLVMVoidTypeKind = kind {
            return None;
        }

        unsafe { Some(BasicTypeEnum::new(ty)) }
    }

    // REVIEW: Can you do undef for functions?
    // Seems to "work" - no UB or SF so far but fails
    // LLVMIsAFunction() check. Commenting out for further research
    // pub fn get_undef(&self) -> FunctionValue {
    //     FunctionValue::new(self.fn_type.get_undef()).expect("Should always get an undef value")
    // }
}

impl fmt::Debug for FunctionType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = self.print_to_string();

        f.debug_struct("FunctionType")
            .field("address", &self.as_type_ref())
            .field("is_var_args", &self.is_var_arg())
            .field("llvm_type", &llvm_type)
            .finish()
    }
}

unsafe impl AsTypeRef for FunctionType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.fn_type.ty
    }
}

impl Display for FunctionType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
