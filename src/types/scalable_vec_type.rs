use llvm_sys::core::LLVMGetVectorSize;
use llvm_sys::prelude::LLVMTypeRef;

use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::types::enums::BasicMetadataTypeEnum;
use crate::types::{traits::AsTypeRef, ArrayType, BasicTypeEnum, FunctionType, PointerType, Type};
use crate::values::{ArrayValue, IntValue, ScalableVectorValue};
use crate::AddressSpace;

use std::fmt::{self, Display};

/// A `ScalableVectorType` is the type of a scalable multiple value SIMD constant or variable.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ScalableVectorType<'ctx> {
    scalable_vec_type: Type<'ctx>,
}

impl<'ctx> ScalableVectorType<'ctx> {
    /// Create `ScalableVectorType` from [`LLVMTypeRef`]
    ///
    /// # Safety
    /// Undefined behavior, if referenced type isn't scalable vector type
    pub unsafe fn new(scalable_vector_type: LLVMTypeRef) -> Self {
        assert!(!scalable_vector_type.is_null());

        ScalableVectorType {
            scalable_vec_type: Type::new(scalable_vector_type),
        }
    }

    // TODO: impl only for ScalableVectorType<!StructType<Opaque>>
    // REVIEW: What about Opaque struct hiding in deeper levels
    // like ScalableVectorType<ArrayType<StructType<Opaque>>>?
    /// Gets the size of this `ScalableVectorType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vec_type = f32_type.scalable_vec_type(3);
    /// let f32_scalable_vec_type_size = f32_scalable_vec_type.size_of();
    /// ```
    pub fn size_of(self) -> Option<IntValue<'ctx>> {
        self.scalable_vec_type.size_of()
    }

    /// Gets the alignment of this `ScalableVectorType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vec_type = f32_type.scalable_vec_type(7);
    /// let f32_type_alignment = f32_scalable_vec_type.get_alignment();
    /// ```
    pub fn get_alignment(self) -> IntValue<'ctx> {
        self.scalable_vec_type.get_alignment()
    }

    /// Gets the size of this `ScalableVectorType`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vector_type = f32_type.scalable_vec_type(3);
    ///
    /// assert_eq!(f32_scalable_vector_type.get_size(), 3);
    /// assert_eq!(f32_scalable_vector_type.get_element_type().into_float_type(), f32_type);
    /// ```
    pub fn get_size(self) -> u32 {
        unsafe { LLVMGetVectorSize(self.as_type_ref()) }
    }

    /// Creates a constant zero value of this `ScalableVectorType`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vec_type = f32_type.scalable_vec_type(7);
    /// let f32_scalable_vec_zero = f32_scalable_vec_type.const_zero();
    /// ```
    pub fn const_zero(self) -> ScalableVectorValue<'ctx> {
        unsafe { ScalableVectorValue::new(self.scalable_vec_type.const_zero()) }
    }

    /// Print the definition of a `ScalableVectorType` to `LLVMString`.
    pub fn print_to_string(self) -> LLVMString {
        self.scalable_vec_type.print_to_string()
    }

    /// Creates an undefined instance of a `ScalableVectorType`.
    ///
    /// # Example
    /// ```ignore
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vec_type = f32_type.scalable_vec_type(3);
    /// let f32_scalable_vec_undef = f32_scalable_vec_type.get_undef();
    ///
    /// assert!(f32_scalable_vec_undef.is_undef());
    /// ```
    pub fn get_undef(self) -> ScalableVectorValue<'ctx> {
        unsafe { ScalableVectorValue::new(self.scalable_vec_type.get_undef()) }
    }

    /// Creates a poison instance of a `ScalableVectorType`.
    ///
    /// # Example
    /// ```ignore
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vec_type = f32_type.scalable_vec_type(3);
    /// let f32_scalable_vec_poison = f32_scalable_vec_type.get_undef();
    ///
    /// assert!(f32_scalable_vec_poison.is_undef());
    /// ```
    #[llvm_versions(12..)]
    pub fn get_poison(self) -> ScalableVectorValue<'ctx> {
        unsafe { ScalableVectorValue::new(self.scalable_vec_type.get_poison()) }
    }

    // SubType: ScalableVectorType<BT> -> BT?
    /// Gets the element type of this `ScalableVectorType`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vector_type = f32_type.scalable_vec_type(3);
    ///
    /// assert_eq!(f32_scalable_vector_type.get_size(), 3);
    /// assert_eq!(f32_scalable_vector_type.get_element_type().into_float_type(), f32_type);
    /// ```
    pub fn get_element_type(self) -> BasicTypeEnum<'ctx> {
        self.scalable_vec_type.get_element_type().as_basic_type_enum()
    }

    /// Creates a `PointerType` with this `ScalableVectorType` for its element type.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vec_type = f32_type.scalable_vec_type(3);
    /// let f32_scalable_vec_ptr_type = f32_scalable_vec_type.ptr_type(AddressSpace::default());
    ///
    /// #[cfg(feature = "typed-pointers")]
    /// assert_eq!(f32_scalable_vec_ptr_type.get_element_type().into_scalable_vector_type(), f32_scalable_vec_type);
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
        self.scalable_vec_type.ptr_type(address_space)
    }

    /// Creates a `FunctionType` with this `ScalableVectorType` for its return type.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vec_type = f32_type.scalable_vec_type(3);
    /// let fn_type = f32_scalable_vec_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.scalable_vec_type.fn_type(param_types, is_var_args)
    }

    /// Creates an `ArrayType` with this `ScalableVectorType` for its element type.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vec_type = f32_type.scalable_vec_type(3);
    /// let f32_scalable_vec_array_type = f32_scalable_vec_type.array_type(3);
    ///
    /// assert_eq!(f32_scalable_vec_array_type.len(), 3);
    /// assert_eq!(f32_scalable_vec_array_type.get_element_type().into_scalable_vector_type(), f32_scalable_vec_type);
    /// ```
    pub fn array_type(self, size: u32) -> ArrayType<'ctx> {
        self.scalable_vec_type.array_type(size)
    }

    /// Creates a constant `ArrayValue`.
    ///
    /// # Example
    /// ```ignore
    /// use inkwell::context::Context;
    /// use inkwell::types::ScalableVectorType;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_val = f32_type.const_float(0.);
    /// let f32_val2 = f32_type.const_float(2.);
    /// let f32_scalable_vec_type = f32_type.scalable_vec_type(2);
    /// let f32_scalable_vec_val = f32_scalable_vec_type.const_zero();
    /// let f32_array = f32_scalable_vec_type.const_array(&[f32_scalable_vec_val, f32_scalable_vec_val]);
    ///
    /// assert!(f32_array.is_const());
    /// ```
    pub fn const_array(self, values: &[ScalableVectorValue<'ctx>]) -> ArrayValue<'ctx> {
        unsafe { ArrayValue::new_const_array(&self, values) }
    }

    /// Gets a reference to the `Context` this `ScalableVectorType` was created in.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_scalable_vec_type = f32_type.scalable_vec_type(7);
    ///
    /// assert_eq!(f32_scalable_vec_type.get_context(), context);
    /// ```
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.scalable_vec_type.get_context()
    }
}

unsafe impl AsTypeRef for ScalableVectorType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.scalable_vec_type.ty
    }
}

impl Display for ScalableVectorType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
