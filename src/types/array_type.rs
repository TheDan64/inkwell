#[allow(deprecated)]
use llvm_sys::core::LLVMGetArrayLength;
use llvm_sys::prelude::LLVMTypeRef;

use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::types::enums::BasicMetadataTypeEnum;
use crate::types::traits::AsTypeRef;
use crate::types::{BasicTypeEnum, FunctionType, PointerType, Type};
use crate::values::{ArrayValue, IntValue};
use crate::AddressSpace;

use std::fmt::{self, Display};

/// An `ArrayType` is the type of contiguous constants or variables.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ArrayType<'ctx> {
    array_type: Type<'ctx>,
}

impl<'ctx> ArrayType<'ctx> {
    /// Create `ArrayType` from [`LLVMTypeRef`]
    ///
    /// # Safety
    /// Undefined behavior, if referenced type isn't array type
    pub unsafe fn new(array_type: LLVMTypeRef) -> Self {
        assert!(!array_type.is_null());

        ArrayType {
            array_type: Type::new(array_type),
        }
    }

    // TODO: impl only for ArrayType<!StructType<Opaque>>
    /// Gets the size of this `ArrayType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_type_size = i8_array_type.size_of();
    /// ```
    pub fn size_of(self) -> Option<IntValue<'ctx>> {
        self.array_type.size_of()
    }

    /// Gets the alignment of this `ArrayType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_type_alignment = i8_array_type.get_alignment();
    /// ```
    pub fn get_alignment(self) -> IntValue<'ctx> {
        self.array_type.get_alignment()
    }

    /// Creates a `PointerType` with this `ArrayType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_ptr_type = i8_array_type.ptr_type(AddressSpace::default());
    ///
    /// #[cfg(feature = "typed-pointers")]
    /// assert_eq!(i8_array_ptr_type.get_element_type().into_array_type(), i8_array_type);
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
        self.array_type.ptr_type(address_space)
    }

    /// Gets a reference to the `Context` this `ArrayType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    ///
    /// assert_eq!(i8_array_type.get_context(), context);
    /// ```
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.array_type.get_context()
    }

    /// Creates a `FunctionType` with this `ArrayType` for its return type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let fn_type = i8_array_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.array_type.fn_type(param_types, is_var_args)
    }

    /// Creates an `ArrayType` with this `ArrayType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_array_type = i8_array_type.array_type(3);
    ///
    /// assert_eq!(i8_array_array_type.len(), 3);
    /// assert_eq!(i8_array_array_type.get_element_type().into_array_type(), i8_array_type);
    /// ```
    pub fn array_type(self, size: u32) -> ArrayType<'ctx> {
        self.array_type.array_type(size)
    }

    /// Creates a constant `ArrayValue` of `ArrayValue`s.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_array_type = f32_type.array_type(3);
    /// let f32_array_val = f32_array_type.const_zero();
    /// let f32_array_array = f32_array_type.const_array(&[f32_array_val, f32_array_val]);
    ///
    /// assert!(f32_array_array.is_const());
    /// ```
    pub fn const_array(self, values: &[ArrayValue<'ctx>]) -> ArrayValue<'ctx> {
        unsafe { ArrayValue::new_const_array(&self, values) }
    }

    /// Creates a constant zero value of this `ArrayType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_zero = i8_array_type.const_zero();
    /// ```
    pub fn const_zero(self) -> ArrayValue<'ctx> {
        unsafe { ArrayValue::new(self.array_type.const_zero()) }
    }

    /// Gets the length of this `ArrayType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    ///
    /// assert_eq!(i8_array_type.len(), 3);
    /// ```
    pub fn len(self) -> u32 {
        #[allow(deprecated)]
        unsafe {
            LLVMGetArrayLength(self.as_type_ref())
        }
    }

    /// Returns `true` if this `ArrayType` contains no elements.
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Print the definition of an `ArrayType` to `LLVMString`
    pub fn print_to_string(self) -> LLVMString {
        self.array_type.print_to_string()
    }

    /// Creates an undefined instance of a `ArrayType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_undef = i8_array_type.get_undef();
    ///
    /// assert!(i8_array_undef.is_undef());
    /// ```
    pub fn get_undef(self) -> ArrayValue<'ctx> {
        unsafe { ArrayValue::new(self.array_type.get_undef()) }
    }

    /// Creates a poison instance of a `ArrayType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::values::AnyValue;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    /// let i8_array_poison = i8_array_type.get_poison();
    ///
    /// assert!(i8_array_poison.is_poison());
    /// ```
    #[llvm_versions(12..)]
    pub fn get_poison(self) -> ArrayValue<'ctx> {
        unsafe { ArrayValue::new(self.array_type.get_poison()) }
    }

    // SubType: ArrayType<BT> -> BT?
    /// Gets the element type of this `ArrayType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_array_type = i8_type.array_type(3);
    ///
    /// assert_eq!(i8_array_type.get_element_type().into_int_type(), i8_type);
    /// ```
    pub fn get_element_type(self) -> BasicTypeEnum<'ctx> {
        self.array_type.get_element_type().as_basic_type_enum()
    }
}

unsafe impl AsTypeRef for ArrayType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.array_type.ty
    }
}

impl Display for ArrayType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
