use llvm_sys::core::{LLVMConstReal, LLVMConstRealOfStringAndSize, LLVMConstArray};
use llvm_sys::execution_engine::LLVMCreateGenericValueOfFloat;
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use crate::AddressSpace;
use crate::context::ContextRef;
use crate::types::traits::AsTypeRef;
use crate::types::{Type, PointerType, FunctionType, BasicTypeEnum, ArrayType, VectorType};
use crate::values::{AsValueRef, ArrayValue, FloatValue, GenericValue, IntValue};
use crate::types::enums::BasicMetadataTypeEnum;

/// A `FloatType` is the type of a floating point constant or variable.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FloatType<'ctx> {
    float_type: Type<'ctx>,
}

impl<'ctx> FloatType<'ctx> {
    pub(crate) unsafe fn new(float_type: LLVMTypeRef) -> Self {
        assert!(!float_type.is_null());

        FloatType {
            float_type: Type::new(float_type),
        }
    }

    /// Creates a `FunctionType` with this `FloatType` for its return type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let fn_type = f32_type.fn_type(&[], false);
    /// ```
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.float_type.fn_type(param_types, is_var_args)
    }

    /// Creates an `ArrayType` with this `FloatType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_array_type = f32_type.array_type(3);
    ///
    /// assert_eq!(f32_array_type.len(), 3);
    /// assert_eq!(f32_array_type.get_element_type().into_float_type(), f32_type);
    /// ```
    pub fn array_type(self, size: u32) -> ArrayType<'ctx> {
        self.float_type.array_type(size)
    }

    /// Creates a `VectorType` with this `FloatType` for its element type.
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
    pub fn vec_type(self, size: u32) -> VectorType<'ctx> {
        self.float_type.vec_type(size)
    }

    /// Creates a `FloatValue` repesenting a constant value of this `FloatType`.
    /// It will be automatically assigned this `FloatType`'s `Context`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// // Local Context
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_value = f32_type.const_float(42.);
    /// ```
    pub fn const_float(self, value: f64) -> FloatValue<'ctx> {
        unsafe {
            FloatValue::new(LLVMConstReal(self.float_type.ty, value))
        }
    }

    /// Create a `FloatValue` from a string. LLVM provides no error handling here,
    /// so this may produce unexpected results and should not be relied upon for validation.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::values::AnyValue;
    ///
    /// let context = Context::create();
    /// let f64_type = context.f64_type();
    /// let f64_val = f64_type.const_float_from_string("3.6");
    ///
    /// assert_eq!(f64_val.print_to_string().to_string(), "double 3.600000e+00");
    ///
    /// let f64_val = f64_type.const_float_from_string("3.");
    ///
    /// assert_eq!(f64_val.print_to_string().to_string(), "double 3.000000e+00");
    ///
    /// let f64_val = f64_type.const_float_from_string("3");
    ///
    /// assert_eq!(f64_val.print_to_string().to_string(), "double 3.000000e+00");
    ///
    /// let f64_val = f64_type.const_float_from_string("");
    ///
    /// assert_eq!(f64_val.print_to_string().to_string(), "double 0.000000e+00");
    ///
    /// let f64_val = f64_type.const_float_from_string("3.asd");
    ///
    /// assert_eq!(f64_val.print_to_string().to_string(), "double 0x7FF0000000000000");
    /// ```
    pub fn const_float_from_string(self, slice: &str) -> FloatValue<'ctx> {
        unsafe {
            FloatValue::new(LLVMConstRealOfStringAndSize(self.as_type_ref(), slice.as_ptr() as *const ::libc::c_char, slice.len() as u32))
        }
    }

    /// Creates a constant zero value of this `FloatType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::values::AnyValue;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_zero = f32_type.const_zero();
    ///
    /// assert_eq!(f32_zero.print_to_string().to_string(), "float 0.000000e+00");
    /// ```
    pub fn const_zero(self) -> FloatValue<'ctx> {
        unsafe {
            FloatValue::new(self.float_type.const_zero())
        }
    }

    /// Gets the size of this `FloatType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_type_size = f32_type.size_of();
    /// ```
    pub fn size_of(self) -> IntValue<'ctx> {
        self.float_type.size_of().unwrap()
    }

    /// Gets the alignment of this `FloatType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_type_alignment = f32_type.get_alignment();
    /// ```
    pub fn get_alignment(self) -> IntValue<'ctx> {
        self.float_type.get_alignment()
    }

    /// Gets a reference to the `Context` this `FloatType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    ///
    /// assert_eq!(*f32_type.get_context(), context);
    /// ```
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.float_type.get_context()
    }

    /// Creates a `PointerType` with this `FloatType` for its element type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_ptr_type = f32_type.ptr_type(AddressSpace::Generic);
    ///
    /// assert_eq!(f32_ptr_type.get_element_type().into_float_type(), f32_type);
    /// ```
    pub fn ptr_type(self, address_space: AddressSpace) -> PointerType<'ctx> {
        self.float_type.ptr_type(address_space)
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of an `IntType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7..=4.0)]
    pub fn print_to_stderr(self) {
        self.float_type.print_to_stderr()
    }

    /// Creates an undefined instance of a `FloatType`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_undef = f32_type.get_undef();
    ///
    /// assert!(f32_undef.is_undef());
    /// ```
    pub fn get_undef(&self) -> FloatValue<'ctx> {
        unsafe {
            FloatValue::new(self.float_type.get_undef())
        }
    }

    /// Creates a `GenericValue` for use with `ExecutionEngine`s.
    pub fn create_generic_value(self, value: f64) -> GenericValue<'ctx> {
        unsafe {
            GenericValue::new(LLVMCreateGenericValueOfFloat(self.as_type_ref(), value))
        }
    }

    /// Creates a constant `ArrayValue`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_val = f32_type.const_float(0.);
    /// let f32_val2 = f32_type.const_float(2.);
    /// let f32_array = f32_type.const_array(&[f32_val, f32_val2]);
    ///
    /// assert!(f32_array.is_const());
    /// ```
    pub fn const_array(self, values: &[FloatValue<'ctx>]) -> ArrayValue<'ctx> {
        let mut values: Vec<LLVMValueRef> = values.iter()
                                                  .map(|val| val.as_value_ref())
                                                  .collect();
        unsafe {
            ArrayValue::new(LLVMConstArray(self.as_type_ref(), values.as_mut_ptr(), values.len() as u32))
        }
    }
}

impl AsTypeRef for FloatType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.float_type.ty
    }
}
