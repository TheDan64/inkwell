use llvm_sys::core::{LLVMConstVector, LLVMGetVectorSize};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::types::enums::BasicMetadataTypeEnum;
use crate::types::{traits::AsTypeRef, ArrayType, BasicTypeEnum, FunctionType, PointerType, Type};
use crate::values::{ArrayValue, BasicValue, IntValue, VectorValue};
use crate::AddressSpace;

use std::fmt::{self, Display};

/// A `VectorType` is the type of a multiple value SIMD constant or variable.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VectorType<'ctx> {
    vec_type: Type<'ctx>,
}

impl<'ctx> VectorType<'ctx> {
    /// Create `VectorType` from [`LLVMTypeRef`]
    ///
    /// # Safety
    /// Undefined behavior, if referenced type isn't vector type
    pub unsafe fn new(vector_type: LLVMTypeRef) -> Self {
        assert!(!vector_type.is_null());

        VectorType {
            vec_type: Type::new(vector_type),
        }
    }

    // TODO: impl only for VectorType<!StructType<Opaque>>
    // REVIEW: What about Opaque struct hiding in deeper levels
    // like VectorType<ArrayType<StructType<Opaque>>>?
    /// Gets the size of this `VectorType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vec_type = f32_type.vec_type(3);
    /// let f32_vec_type_size = f32_vec_type.size_of();
    /// ```
    pub fn size_of(self) -> Option<IntValue<'ctx>> {
        self.vec_type.size_of()
    }

    /// Gets the alignment of this `VectorType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vec_type = f32_type.vec_type(7);
    /// let f32_type_alignment = f32_vec_type.get_alignment();
    /// ```
    pub fn get_alignment(self) -> IntValue<'ctx> {
        self.vec_type.get_alignment()
    }

    /// Gets the size of this `VectorType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vector_type = f32_type.vec_type(3);
    ///
    /// assert_eq!(f32_vector_type.get_size(), 3);
    /// assert_eq!(f32_vector_type.get_element_type().into_float_type(), f32_type);
    /// ```
    pub fn get_size(self) -> u32 {
        unsafe { LLVMGetVectorSize(self.as_type_ref()) }
    }

    // REVIEW:
    // TypeSafety v2 (GH Issue #8) could help here by constraining
    // sub-types to be the same across the board. For now, we could
    // have V just be the set of Int & Float and any others that
    // are valid for Vectors
    // REVIEW: Maybe we could make this use &self if the vector size
    // is stored as a const and the input values took a const size?
    // Something like: values: &[&V; self.size]. Doesn't sound possible though
    /// Creates a constant `VectorValue`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::VectorType;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_val = f32_type.const_float(0.);
    /// let f32_val2 = f32_type.const_float(2.);
    /// let f32_vec_val = VectorType::const_vector(&[f32_val, f32_val2]);
    ///
    /// assert!(f32_vec_val.is_constant_vector());
    /// ```
    pub fn const_vector<V: BasicValue<'ctx>>(values: &[V]) -> VectorValue<'ctx> {
        let mut values: Vec<LLVMValueRef> = values.iter().map(|val| val.as_value_ref()).collect();
        unsafe { VectorValue::new(LLVMConstVector(values.as_mut_ptr(), values.len() as u32)) }
    }

    /// Creates a constant zero value of this `VectorType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vec_type = f32_type.vec_type(7);
    /// let f32_vec_zero = f32_vec_type.const_zero();
    /// ```
    pub fn const_zero(self) -> VectorValue<'ctx> {
        unsafe { VectorValue::new(self.vec_type.const_zero()) }
    }

    /// Print the definition of a `VectorType` to `LLVMString`.
    pub fn print_to_string(self) -> LLVMString {
        self.vec_type.print_to_string()
    }

    /// Creates an undefined instance of a `VectorType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vec_type = f32_type.vec_type(3);
    /// let f32_vec_undef = f32_vec_type.get_undef();
    ///
    /// assert!(f32_vec_undef.is_undef());
    /// ```
    pub fn get_undef(self) -> VectorValue<'ctx> {
        unsafe { VectorValue::new(self.vec_type.get_undef()) }
    }

    /// Creates a poison instance of a `VectorType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vec_type = f32_type.vec_type(3);
    /// let f32_vec_poison = f32_vec_type.get_undef();
    ///
    /// assert!(f32_vec_poison.is_undef());
    /// ```
    #[llvm_versions(12..)]
    pub fn get_poison(self) -> VectorValue<'ctx> {
        unsafe { VectorValue::new(self.vec_type.get_poison()) }
    }

    // SubType: VectorType<BT> -> BT?
    /// Gets the element type of this `VectorType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vector_type = f32_type.vec_type(3);
    ///
    /// assert_eq!(f32_vector_type.get_size(), 3);
    /// assert_eq!(f32_vector_type.get_element_type().into_float_type(), f32_type);
    /// ```
    pub fn get_element_type(self) -> BasicTypeEnum<'ctx> {
        self.vec_type.get_element_type().as_basic_type_enum()
    }

    /// Creates a `PointerType` with this `VectorType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vec_type = f32_type.vec_type(3);
    /// let f32_vec_ptr_type = f32_vec_type.ptr_type(AddressSpace::default());
    ///
    /// #[cfg(feature = "typed-pointers")]
    /// assert_eq!(f32_vec_ptr_type.get_element_type().into_vector_type(), f32_vec_type);
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
        self.vec_type.ptr_type(address_space)
    }

    /// Creates a `FunctionType` with this `VectorType` for its return type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vec_type = f32_type.vec_type(3);
    /// let fn_type = f32_vec_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.vec_type.fn_type(param_types, is_var_args)
    }

    /// Creates an `ArrayType` with this `VectorType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vec_type = f32_type.vec_type(3);
    /// let f32_vec_array_type = f32_vec_type.array_type(3);
    ///
    /// assert_eq!(f32_vec_array_type.len(), 3);
    /// assert_eq!(f32_vec_array_type.get_element_type().into_vector_type(), f32_vec_type);
    /// ```
    pub fn array_type(self, size: u32) -> ArrayType<'ctx> {
        self.vec_type.array_type(size)
    }

    /// Creates a constant `ArrayValue`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::VectorType;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_val = f32_type.const_float(0.);
    /// let f32_val2 = f32_type.const_float(2.);
    /// let f32_vec_type = f32_type.vec_type(2);
    /// let f32_vec_val = VectorType::const_vector(&[f32_val, f32_val2]);
    /// let f32_array = f32_vec_type.const_array(&[f32_vec_val, f32_vec_val]);
    ///
    /// assert!(f32_array.is_const());
    /// ```
    pub fn const_array(self, values: &[VectorValue<'ctx>]) -> ArrayValue<'ctx> {
        unsafe { ArrayValue::new_const_array(&self, values) }
    }

    /// Gets a reference to the `Context` this `VectorType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vec_type = f32_type.vec_type(7);
    ///
    /// assert_eq!(f32_vec_type.get_context(), context);
    /// ```
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.vec_type.get_context()
    }
}

unsafe impl AsTypeRef for VectorType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.vec_type.ty
    }
}

impl Display for VectorType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
