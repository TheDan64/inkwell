use llvm_sys::prelude::LLVMTypeRef;

use std::convert::TryFrom;
use std::fmt::Debug;

use crate::AddressSpace;
use crate::types::{IntType, FunctionType, FloatType, PointerType, StructType, ArrayType, VectorType, VoidType, Type};
use crate::types::enums::{AnyTypeEnum, BasicTypeEnum};
use crate::values::{IntMathValue, FloatMathValue, PointerMathValue, IntValue, FloatValue, PointerValue, VectorValue};

// This is an ugly privacy hack so that Type can stay private to this module
// and so that super traits using this trait will be not be implementable
// outside this library
pub trait AsTypeRef {
    fn as_type_ref(&self) -> LLVMTypeRef;
}

macro_rules! trait_type_set {
    ($trait_name:ident: $($args:ident),*) => (
        $(
            impl $trait_name for $args {}
        )*
    );
}

// REVIEW: print_to_string might be a good candidate to live here?
/// Represents any LLVM type.
pub trait AnyType: AsTypeRef + Debug {
    /// Returns an `AnyTypeEnum` that represents the current type.
    fn as_any_type_enum(&self) -> AnyTypeEnum {
        AnyTypeEnum::new(self.as_type_ref())
    }
}

/// Represents a basic LLVM type, that may be used in functions and struct definitions.
pub trait BasicType: AnyType {
    /// Returns a `BasicTypeEnum` that represents the current type.
    fn as_basic_type_enum(&self) -> BasicTypeEnum {
        BasicTypeEnum::new(self.as_type_ref())
    }

    /// Create a `FunctionType` with this `BasicType` as its return type.
    ///
    /// Example:
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::types::BasicType;
    ///
    /// let context = Context::create();
    /// let int = context.i32_type();
    /// let int_basic_type = int.as_basic_type_enum();
    /// assert_eq!(int_basic_type.fn_type(&[], false), int.fn_type(&[], false));
    /// ```
    fn fn_type(&self, param_types: &[BasicTypeEnum], is_var_args: bool) -> FunctionType {
        Type::new(self.as_type_ref()).fn_type(param_types, is_var_args)
    }

    /// Create an `ArrayType` with this `BasicType` as its elements.
    ///
    /// Example:
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::types::BasicType;
    ///
    /// let context = Context::create();
    /// let int = context.i32_type();
    /// let int_basic_type = int.as_basic_type_enum();
    /// assert_eq!(int_basic_type.array_type(32), int.array_type(32));
    /// ```
    fn array_type(&self, size: u32) -> ArrayType {
        Type::new(self.as_type_ref()).array_type(size)
    }

    /// Create a `PointerType` that points to this `BasicType`.
    ///
    /// Example:
    /// ```
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
    fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        Type::new(self.as_type_ref()).ptr_type(address_space)
    }
}

/// Represents an LLVM type that can have integer math operations applied to it.
pub trait IntMathType: BasicType {
    /// The value instance of an int or int vector type.
    type ValueType: IntMathValue;
    /// The type for int to float or int vector to float vector conversions.
    type MathConvType: FloatMathType;
    /// The type for int to pointer or int vector to pointer vector conversions.
    type PtrConvType: PointerMathType;
}

/// Represents an LLVM type that can have floating point math operations applied to it.
pub trait FloatMathType: BasicType {
    /// The value instance of a float or float vector type.
    type ValueType: FloatMathValue;
    /// The type for float to int or float vector to int vector conversions.
    type MathConvType: IntMathType;
}

/// Represents an LLVM type that can have pointer operations applied to it.
pub trait PointerMathType: BasicType {
    /// The value instance of a pointer type.
    type ValueType: PointerMathValue;
    /// The type for pointer to int or pointer vector to int conversions.
    type PtrConvType: IntMathType;
}

trait_type_set! {AnyType: AnyTypeEnum, BasicTypeEnum, IntType, FunctionType, FloatType, PointerType, StructType, ArrayType, VoidType, VectorType}
trait_type_set! {BasicType: BasicTypeEnum, IntType, FloatType, PointerType, StructType, ArrayType, VectorType}

impl IntMathType for IntType {
    type ValueType = IntValue;
    type MathConvType = FloatType;
    type PtrConvType = PointerType;
}

impl IntMathType for VectorType {
    type ValueType = VectorValue;
    type MathConvType = VectorType;
    type PtrConvType = VectorType;
}

impl FloatMathType for FloatType {
    type ValueType = FloatValue;
    type MathConvType = IntType;
}

impl FloatMathType for VectorType {
    type ValueType = VectorValue;
    type MathConvType = VectorType;
}

impl PointerMathType for PointerType {
    type ValueType = PointerValue;
    type PtrConvType = IntType;
}

impl PointerMathType for VectorType {
    type ValueType = VectorValue;
    type PtrConvType = VectorType;
}

macro_rules! impl_try_from_basic_type_enum {
    ($type_name:ident) => (
        impl TryFrom<BasicTypeEnum> for $type_name {
            type Error = &'static str;

            fn try_from(ty: BasicTypeEnum) -> Result<Self, Self::Error> {
                match ty {
                    BasicTypeEnum::$type_name(ty) => Ok(ty),
                    _ => Err("bad try from"),
                }
            }
        }
    )
}

impl_try_from_basic_type_enum!(ArrayType);
impl_try_from_basic_type_enum!(FloatType);
impl_try_from_basic_type_enum!(IntType);
impl_try_from_basic_type_enum!(PointerType);
impl_try_from_basic_type_enum!(StructType);
impl_try_from_basic_type_enum!(VectorType);
