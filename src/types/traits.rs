use llvm_sys::prelude::LLVMTypeRef;

use std::fmt::Debug;

use crate::AddressSpace;
use crate::types::{IntType, FunctionType, FloatType, PointerType, StructType, ArrayType, VectorType, VoidType, Type};
use crate::types::enums::{AnyTypeEnum, BasicTypeEnum};
use crate::values::{IntMathValue, FloatMathValue, PointerMathValue, IntValue, FloatValue, PointerValue, VectorValue};
use crate::support::LLVMString;

// This is an ugly privacy hack so that Type can stay private to this module
// and so that super traits using this trait will be not be implementable
// outside this library
pub trait AsTypeRef {
    fn as_type_ref(&self) -> LLVMTypeRef;
}

macro_rules! trait_type_set {
    ($trait_name:ident: $($args:ident),*) => (
        $(
            impl<'ctx> $trait_name<'ctx> for $args<'ctx> {}
        )*
    );
}

/// Represents any LLVM type.
pub trait AnyType<'ctx>: AsTypeRef + Debug {
    /// Returns an `AnyTypeEnum` that represents the current type.
    fn as_any_type_enum(&self) -> AnyTypeEnum<'ctx> {
        AnyTypeEnum::new(self.as_type_ref())
    }

    /// Prints the definition of a Type to a `LLVMString`.
    fn print_to_string(&self) -> LLVMString {
        Type::new(self.as_type_ref()).print_to_string()
    }
}

/// Represents a basic LLVM type, that may be used in functions and struct definitions.
pub trait BasicType<'ctx>: AnyType<'ctx> {
    /// Returns a `BasicTypeEnum` that represents the current type.
    fn as_basic_type_enum(&self) -> BasicTypeEnum<'ctx> {
        BasicTypeEnum::new(self.as_type_ref())
    }

    /// Create a `FunctionType` with this `BasicType` as its return type.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::BasicType;
    ///
    /// let context = Context::create();
    /// let int = context.i32_type();
    /// let int_basic_type = int.as_basic_type_enum();
    /// assert_eq!(int_basic_type.fn_type(&[], false), int.fn_type(&[], false));
    /// ```
    fn fn_type(&self, param_types: &[BasicTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        Type::new(self.as_type_ref()).fn_type(param_types, is_var_args)
    }

    /// Determines whether or not this `BasicType` is sized or not.
    /// For example, opaque structs are unsized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::BasicType;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_vec_type = f32_type.vec_type(40);
    ///
    /// assert!(f32_vec_type.is_sized());
    /// ```
    fn is_sized(&self) -> bool {
        Type::new(self.as_type_ref()).is_sized()
    }

    /// Gets the size of this `BasicType`. Value may vary depending on the target architecture.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::BasicType;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let f32_basic_type = f32_type.as_basic_type_enum();
    /// let f32_type_size = f32_basic_type.size_of();
    /// ```
    fn size_of(&self) -> Option<IntValue<'ctx>> {
        Type::new(self.as_type_ref()).size_of()
    }

    /// Create an `ArrayType` with this `BasicType` as its elements.
    ///
    /// Example:
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::BasicType;
    ///
    /// let context = Context::create();
    /// let int = context.i32_type();
    /// let int_basic_type = int.as_basic_type_enum();
    /// assert_eq!(int_basic_type.array_type(32), int.array_type(32));
    /// ```
    // FIXME: This likely doesn't belong on the trait, since not all basic types can be turned into arrays?
    fn array_type(&self, size: u32) -> ArrayType<'ctx> {
        Type::new(self.as_type_ref()).array_type(size)
    }

    /// Create a `PointerType` that points to this `BasicType`.
    ///
    /// Example:
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::types::BasicType;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let int = context.i32_type();
    /// let int_basic_type = int.as_basic_type_enum();
    /// let addr_space = AddressSpace::Generic;
    /// assert_eq!(int_basic_type.ptr_type(addr_space), int.ptr_type(addr_space));
    /// ```
    fn ptr_type(&self, address_space: AddressSpace) -> PointerType<'ctx> {
        Type::new(self.as_type_ref()).ptr_type(address_space)
    }
}

/// Represents an LLVM type that can have integer math operations applied to it.
pub trait IntMathType<'ctx>: BasicType<'ctx> {
    /// The value instance of an int or int vector type.
    type ValueType: IntMathValue<'ctx>;
    /// The type for int to float or int vector to float vector conversions.
    type MathConvType: FloatMathType<'ctx>;
    /// The type for int to pointer or int vector to pointer vector conversions.
    type PtrConvType: PointerMathType<'ctx>;
}

/// Represents an LLVM type that can have floating point math operations applied to it.
pub trait FloatMathType<'ctx>: BasicType<'ctx> {
    /// The value instance of a float or float vector type.
    type ValueType: FloatMathValue<'ctx>;
    /// The type for float to int or float vector to int vector conversions.
    type MathConvType: IntMathType<'ctx>;
}

/// Represents an LLVM type that can have pointer operations applied to it.
pub trait PointerMathType<'ctx>: BasicType<'ctx> {
    /// The value instance of a pointer type.
    type ValueType: PointerMathValue<'ctx>;
    /// The type for pointer to int or pointer vector to int conversions.
    type PtrConvType: IntMathType<'ctx>;
}

trait_type_set! {AnyType: AnyTypeEnum, BasicTypeEnum, IntType, FunctionType, FloatType, PointerType, StructType, ArrayType, VoidType, VectorType}
trait_type_set! {BasicType: BasicTypeEnum, IntType, FloatType, PointerType, StructType, ArrayType, VectorType}

impl<'ctx> IntMathType<'ctx> for IntType<'ctx> {
    type ValueType = IntValue<'ctx>;
    type MathConvType = FloatType<'ctx>;
    type PtrConvType = PointerType<'ctx>;
}

impl<'ctx> IntMathType<'ctx> for VectorType<'ctx> {
    type ValueType = VectorValue<'ctx>;
    type MathConvType = VectorType<'ctx>;
    type PtrConvType = VectorType<'ctx>;
}

impl<'ctx> FloatMathType<'ctx> for FloatType<'ctx> {
    type ValueType = FloatValue<'ctx>;
    type MathConvType = IntType<'ctx>;
}

impl<'ctx> FloatMathType<'ctx> for VectorType<'ctx> {
    type ValueType = VectorValue<'ctx>;
    type MathConvType = VectorType<'ctx>;
}

impl<'ctx> PointerMathType<'ctx> for PointerType<'ctx> {
    type ValueType = PointerValue<'ctx>;
    type PtrConvType = IntType<'ctx>;
}

impl<'ctx> PointerMathType<'ctx> for VectorType<'ctx> {
    type ValueType = VectorValue<'ctx>;
    type PtrConvType = VectorType<'ctx>;
}
