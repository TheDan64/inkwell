use llvm_sys::core::LLVMGetPointerAddressSpace;
#[llvm_versions(15..)]
use llvm_sys::core::LLVMPointerTypeIsOpaque;
use llvm_sys::prelude::LLVMTypeRef;

use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::types::traits::AsTypeRef;
#[llvm_versions(..=14)]
use crate::types::AnyTypeEnum;
use crate::types::{ArrayType, FunctionType, Type, VectorType};
use crate::values::{ArrayValue, IntValue, PointerValue};
use crate::AddressSpace;

use crate::types::enums::BasicMetadataTypeEnum;
use std::fmt::{self, Display};

/// A `PointerType` is the type of a pointer constant or variable.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PointerType<'ctx> {
    ptr_type: Type<'ctx>,
}

impl<'ctx> PointerType<'ctx> {
    /// Create `PointerType` from [`LLVMTypeRef`]
    ///
    /// # Safety
    /// Undefined behavior, if referenced type isn't pointer type
    pub unsafe fn new(ptr_type: LLVMTypeRef) -> Self {
        assert!(!ptr_type.is_null());

        PointerType {
            ptr_type: Type::new(ptr_type),
        }
    }

    /// Gets the size of this `PointerType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let f32_ptr_type_size = f32_ptr_type.size_of();
    /// ```
    pub fn size_of(self) -> IntValue<'ctx> {
        self.ptr_type.size_of().unwrap()
    }

    /// Gets the alignment of this `PointerType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let f32_ptr_type_alignment = f32_ptr_type.get_alignment();
    /// ```
    pub fn get_alignment(self) -> IntValue<'ctx> {
        self.ptr_type.get_alignment()
    }

    /// Creates a `PointerType` with this `PointerType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let f32_ptr_ptr_type = f32_ptr_type.ptr_type(AddressSpace::default());
    ///
    /// #[cfg(any(
    ///     feature = "llvm4-0",
    ///     feature = "llvm5-0",
    ///     feature = "llvm6-0",
    ///     feature = "llvm7-0",
    ///     feature = "llvm8-0",
    ///     feature = "llvm9-0",
    ///     feature = "llvm10-0",
    ///     feature = "llvm11-0",
    ///     feature = "llvm12-0",
    ///     feature = "llvm13-0",
    ///     feature = "llvm14-0"
    /// ))]
    /// assert_eq!(f32_ptr_ptr_type.get_element_type().into_pointer_type(), f32_ptr_type);
    /// ```
    #[cfg_attr(
        any(
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ),
        deprecated(
            note = "Starting from version 15.0, LLVM doesn't differentiate between pointer types. Use Context::ptr_type instead."
        )
    )]
    pub fn ptr_type(self, address_space: AddressSpace) -> PointerType<'ctx> {
        self.ptr_type.ptr_type(address_space)
    }

    /// Gets a reference to the `Context` this `PointerType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    ///
    /// assert_eq!(f32_ptr_type.get_context(), context);
    /// ```
    // TODO: Move to AnyType trait
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.ptr_type.get_context()
    }

    /// Creates a `FunctionType` with this `PointerType` for its return type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let fn_type = f32_ptr_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.ptr_type.fn_type(param_types, is_var_args)
    }

    /// Creates an `ArrayType` with this `PointerType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let f32_ptr_array_type = f32_ptr_type.array_type(3);
    ///
    /// assert_eq!(f32_ptr_array_type.len(), 3);
    /// assert_eq!(f32_ptr_array_type.get_element_type().into_pointer_type(), f32_ptr_type);
    /// ```
    pub fn array_type(self, size: u32) -> ArrayType<'ctx> {
        self.ptr_type.array_type(size)
    }

    /// Gets the `AddressSpace` a `PointerType` was created with.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    ///
    /// assert_eq!(f32_ptr_type.get_address_space(), AddressSpace::default());
    /// ```
    pub fn get_address_space(self) -> AddressSpace {
        let addr_space = unsafe { LLVMGetPointerAddressSpace(self.as_type_ref()) };

        AddressSpace(addr_space)
    }

    /// Print the definition of a `PointerType` to `LLVMString`.
    pub fn print_to_string(self) -> LLVMString {
        self.ptr_type.print_to_string()
    }

    /// Creates a null `PointerValue` of this `PointerType`.
    /// It will be automatically assigned this `PointerType`'s `Context`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// // Local Context
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let f32_ptr_null = f32_ptr_type.const_null();
    ///
    /// assert!(f32_ptr_null.is_null());
    /// ```
    pub fn const_null(self) -> PointerValue<'ctx> {
        unsafe { PointerValue::new(self.ptr_type.const_zero()) }
    }

    // REVIEW: Unlike the other const_zero functions, this one becomes null instead of a 0 value. Maybe remove?
    /// Creates a constant null value of this `PointerType`.
    /// This is practically the same as calling `const_null` for this particular type and
    /// so this function may be removed in the future.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let f32_ptr_zero = f32_ptr_type.const_zero();
    /// ```
    pub fn const_zero(self) -> PointerValue<'ctx> {
        unsafe { PointerValue::new(self.ptr_type.const_zero()) }
    }

    /// Creates an undefined instance of a `PointerType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let f32_ptr_undef = f32_ptr_type.get_undef();
    ///
    /// assert!(f32_ptr_undef.is_undef());
    /// ```
    pub fn get_undef(self) -> PointerValue<'ctx> {
        unsafe { PointerValue::new(self.ptr_type.get_undef()) }
    }

    /// Creates a poison instance of a `PointerType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    /// use inkwell::values::AnyValue;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let f32_ptr_undef = f32_ptr_type.get_poison();
    ///
    /// assert!(f32_ptr_undef.is_poison());
    /// ```
    #[llvm_versions(12..)]
    pub fn get_poison(self) -> PointerValue<'ctx> {
        unsafe { PointerValue::new(self.ptr_type.get_poison()) }
    }

    /// Creates a `VectorType` with this `PointerType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let f32_ptr_vec_type = f32_ptr_type.vec_type(3);
    ///
    /// assert_eq!(f32_ptr_vec_type.get_size(), 3);
    /// assert_eq!(f32_ptr_vec_type.get_element_type().into_pointer_type(), f32_ptr_type);
    /// ```
    pub fn vec_type(self, size: u32) -> VectorType<'ctx> {
        self.ptr_type.vec_type(size)
    }

    // SubType: PointerrType<BT> -> BT?
    /// Gets the element type of this `PointerType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    ///
    /// assert_eq!(f32_ptr_type.get_element_type().into_float_type(), f32_type);
    /// ```
    #[llvm_versions(..=14)]
    pub fn get_element_type(self) -> AnyTypeEnum<'ctx> {
        self.ptr_type.get_element_type()
    }

    /// Creates a constant `ArrayValue`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// #[cfg(not(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0")))]
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-0"))]
    /// let f32_ptr_type = context.ptr_type(AddressSpace::default());
    /// let f32_ptr_val = f32_ptr_type.const_null();
    /// let f32_ptr_array = f32_ptr_type.const_array(&[f32_ptr_val, f32_ptr_val]);
    ///
    /// assert!(f32_ptr_array.is_const());
    /// ```
    pub fn const_array(self, values: &[PointerValue<'ctx>]) -> ArrayValue<'ctx> {
        unsafe { ArrayValue::new_const_array(&self, values) }
    }

    /// Determine whether this pointer is opaque.
    #[llvm_versions(15..)]
    pub fn is_opaque(self) -> bool {
        unsafe { LLVMPointerTypeIsOpaque(self.ptr_type.ty) != 0 }
    }
}

unsafe impl AsTypeRef for PointerType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.ptr_type.ty
    }
}

impl Display for PointerType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
