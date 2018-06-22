use llvm_sys::prelude::LLVMTypeRef;

use std::fmt::Debug;

use types::{IntType, FunctionType, FloatType, PointerType, StructType, ArrayType, VectorType, VoidType, Type};
use types::enums::{AnyTypeEnum, BasicTypeEnum};
use values::{IntMathValue, FloatMathValue, IntValue, FloatValue, VectorValue};

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

macro_rules! math_trait_type_set {
    ($trait_name: ident: $(($base_type:ident => $value_type:ident, $conv_type:ident)),*) => (
        $(
            impl $trait_name for $base_type {
                type ValueType = $value_type;
                type ConvType = $conv_type;
            }
        )*
    );
}

/// Represents any LLVM type.
pub trait AnyType: AsTypeRef + Debug {
    /// Returns an `AnyTypeEnum` that represents the current type.
    fn as_any_type_enum(&self) -> AnyTypeEnum {
        AnyTypeEnum::new(self.as_type_ref())
    }
}

/// Represents a basic LLVM type, that may be used in functions and struct declarations.
pub trait BasicType: AnyType {
    /// Returns a `BasicTypeEnum` that represents the current type.
    fn as_basic_type_enum(&self) -> BasicTypeEnum {
        BasicTypeEnum::new(self.as_type_ref())
    }

    fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
        Type::new(self.as_type_ref()).fn_type(param_types, is_var_args)
    }
}

/// Represents an LLVM type that can have integer math operations applied to it.
pub trait IntMathType: BasicType {
    type ValueType: IntMathValue;
    type ConvType: FloatMathType;
}

/// Represents an LLVM type that can have floating point math operations applied to it.
pub trait FloatMathType: BasicType {
    type ValueType: FloatMathValue;
    type ConvType: IntMathType;
}

trait_type_set! {AnyType: AnyTypeEnum, BasicTypeEnum, IntType, FunctionType, FloatType, PointerType, StructType, ArrayType, VoidType, VectorType}
trait_type_set! {BasicType: BasicTypeEnum, IntType, FloatType, PointerType, StructType, ArrayType, VectorType}
math_trait_type_set! {IntMathType: (IntType => IntValue, FloatType), (VectorType => VectorValue, VectorType)}
math_trait_type_set! {FloatMathType: (FloatType => FloatValue, IntType), (VectorType => VectorValue, VectorType)}
