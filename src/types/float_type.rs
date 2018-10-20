use llvm_sys::core::{LLVMConstReal, LLVMHalfType, LLVMFloatType, LLVMDoubleType, LLVMFP128Type, LLVMPPCFP128Type, LLVMConstRealOfStringAndSize, LLVMX86FP80Type, LLVMConstArray};
use llvm_sys::execution_engine::LLVMCreateGenericValueOfFloat;
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use AddressSpace;
use context::ContextRef;
use support::LLVMString;
use types::traits::AsTypeRef;
use types::{Type, PointerType, FunctionType, BasicTypeEnum, ArrayType, VectorType};
use values::{AsValueRef, ArrayValue, FloatValue, GenericValue, IntValue};

/// A `FloatType` is the type of a floating point constant or variable.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FloatType {
    float_type: Type,
}

impl FloatType {
    pub(crate) fn new(float_type: LLVMTypeRef) -> Self {
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
    pub fn fn_type(&self, param_types: &[BasicTypeEnum], is_var_args: bool) -> FunctionType {
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
    pub fn array_type(&self, size: u32) -> ArrayType {
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
    pub fn vec_type(&self, size: u32) -> VectorType {
        self.float_type.vec_type(size)
    }

    /// Creates a `FloatValue` repesenting a constant value of this `FloatType`.
    /// It will be automatically assigned this `FloatType`'s `Context`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::FloatType;
    ///
    /// // Global Context
    /// let f32_type = FloatType::f32_type();
    /// let f32_value = f32_type.const_float(42.);
    ///
    /// // Custom Context
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_value = f32_type.const_float(42.);
    /// ```
    pub fn const_float(&self, value: f64) -> FloatValue {
        let value = unsafe {
            LLVMConstReal(self.float_type.type_, value)
        };

        FloatValue::new(value)
    }

    /// Create a `FloatValue` from a string. LLVM provides no error handling here,
    /// so this may produce unexpected results and should not be relied upon for validation.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
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
    pub fn const_float_from_string(&self, slice: &str) -> FloatValue {
        let value = unsafe {
            LLVMConstRealOfStringAndSize(self.as_type_ref(), slice.as_ptr() as *const i8, slice.len() as u32)
        };

        FloatValue::new(value)
    }

    /// Creates a constant null value of this `FloatType`.
    /// It will be automatically assigned this `FloatType`'s `Context`.
    ///
    /// # Example
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::types::FloatType;
    ///
    /// // Global Context
    /// let f32_type = FloatType::f32_type();
    /// let f32_value = f32_type.const_null();
    ///
    /// // Custom Context
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_value = f32_type.const_null();
    /// ```
    pub fn const_null(&self) -> FloatValue {
        FloatValue::new(self.float_type.const_null())
    }

    /// Creates a constant zero value of this `FloatType`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_zero = f32_type.const_zero();
    ///
    /// assert_eq!(f32_zero.print_to_string().to_string(), "f32 0");
    /// ```
    pub fn const_zero(&self) -> FloatValue {
        FloatValue::new(self.float_type.const_zero())
    }

    // REVIEW: Always true -> const fn?
    /// Gets whether or not this `FloatType` is sized or not. This is likely
    /// always true and may be removed in the future.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    ///
    /// assert!(f32_type.is_sized());
    /// ```
    pub fn is_sized(&self) -> bool {
        self.float_type.is_sized()
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
    pub fn size_of(&self) -> IntValue {
        self.float_type.size_of()
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
    pub fn get_alignment(&self) -> IntValue {
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
    pub fn get_context(&self) -> ContextRef {
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
    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.float_type.ptr_type(address_space)
    }

    /// Creates a new `FloatType` which represents sixteen bits (two bytes) for the global context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::FloatType;
    ///
    /// let f16_type = FloatType::f16_type();
    ///
    /// assert_eq!(f16_type.get_context(), Context::get_global());
    /// ```
    pub fn f16_type() -> Self {
        let float_type = unsafe {
            LLVMHalfType()
        };

        FloatType::new(float_type)
    }

    /// Creates a new `FloatType` which represents thirty two bits (four bytes) for the global context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::FloatType;
    ///
    /// let f32_type = FloatType::f32_type();
    ///
    /// assert_eq!(f32_type.get_context(), Context::get_global());
    /// ```
    pub fn f32_type() -> Self {
        let float_type = unsafe {
            LLVMFloatType()
        };

        FloatType::new(float_type)
    }

    /// Creates a new `FloatType` which represents sixty four bits (eight bytes) for the global context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::FloatType;
    ///
    /// let f64_type = FloatType::f64_type();
    ///
    /// assert_eq!(f64_type.get_context(), Context::get_global());
    /// ```
    pub fn f64_type() -> Self {
        let float_type = unsafe {
            LLVMDoubleType()
        };

        FloatType::new(float_type)
    }

    /// Gets the `FloatType` representing a 80 bit width. It will be assigned the global context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::FloatType;
    ///
    /// let x86_f80_type = FloatType::x86_f80_type();
    ///
    /// assert_eq!(x86_f80_type.get_context(), Context::get_global());
    /// ```
    pub fn x86_f80_type() -> FloatType {
        let f128_type = unsafe {
            LLVMX86FP80Type()
        };

        FloatType::new(f128_type)
    }

    /// Creates a new `FloatType` which represents one hundred and twenty eight bits (sixteen bytes) for the global context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::FloatType;
    ///
    /// let f128_type = FloatType::f128_type();
    ///
    /// assert_eq!(f128_type.get_context(), Context::get_global());
    /// ```
    // IEEE 754-2008’s binary128 floats according to https://internals.rust-lang.org/t/pre-rfc-introduction-of-half-and-quadruple-precision-floats-f16-and-f128/7521
    pub fn f128_type() -> Self {
        let float_type = unsafe {
            LLVMFP128Type()
        };

        FloatType::new(float_type)
    }

    /// Creates a new `FloatType` which represents one hundred and twenty eight bits (sixteen bytes) for the current context. PPC is two 64 bits side by side rather than one single 128 bit float.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::FloatType;
    ///
    /// let f128_type = FloatType::ppc_f128_type();
    ///
    /// assert_eq!(f128_type.get_context(), Context::get_global());
    /// ```
    // Two 64 bits according to https://internals.rust-lang.org/t/pre-rfc-introduction-of-half-and-quadruple-precision-floats-f16-and-f128/7521
    pub fn ppc_f128_type() -> Self {
        let float_type = unsafe {
            LLVMPPCFP128Type()
        };

        FloatType::new(float_type)
    }

    /// Prints the definition of a `FloatType` to a `LLVMString`.
    pub fn print_to_string(&self) -> LLVMString {
        self.float_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    /// Prints the definition of an `IntType` to stderr. Not available in newer LLVM versions.
    #[llvm_versions(3.7 => 4.0)]
    pub fn print_to_stderr(&self) {
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
    pub fn get_undef(&self) -> FloatValue {
        FloatValue::new(self.float_type.get_undef())
    }

    /// Creates a `GenericValue` for use with `ExecutionEngine`s.
    pub fn create_generic_value(&self, value: f64) -> GenericValue {
        let value = unsafe {
            LLVMCreateGenericValueOfFloat(self.as_type_ref(), value)
        };

        GenericValue::new(value)
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
    pub fn const_array(&self, values: &[FloatValue]) -> ArrayValue {
        let mut values: Vec<LLVMValueRef> = values.iter()
                                                  .map(|val| val.as_value_ref())
                                                  .collect();
        let value = unsafe {
            LLVMConstArray(self.as_type_ref(), values.as_mut_ptr(), values.len() as u32)
        };

        ArrayValue::new(value)
    }
}

impl AsTypeRef for FloatType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.float_type.type_
    }
}
