use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::core::LLVMConstExtractValue;

use std::fmt::Debug;

use values::{ArrayValue, AggregateValueEnum, GlobalValue, StructValue, BasicValueEnum, AnyValueEnum, IntValue, FloatValue, PointerValue, PhiValue, VectorValue, FunctionValue, InstructionValue};
use types::{IntMathType, FloatMathType, PointerMathType, IntType, FloatType, PointerType, VectorType};

// This is an ugly privacy hack so that Type can stay private to this module
// and so that super traits using this trait will be not be implementable
// outside this library
pub trait AsValueRef {
    fn as_value_ref(&self) -> LLVMValueRef;
}

macro_rules! trait_value_set {
    ($trait_name:ident: $($args:ident),*) => (
        $(
            impl $trait_name for $args {}
        )*

        // REVIEW: Possible encompassing methods to implement:
        // as_instruction, is_sized, ge/set metadata methods
    );
}

macro_rules! math_trait_value_set {
    ($trait_name:ident: $(($value_type:ident => $base_type:ident)),*) => (
        $(
            impl $trait_name for $value_type {
                type BaseType = $base_type;
                fn new(value: LLVMValueRef) -> Self {
                    $value_type::new(value)
                }
            }
        )*
    )
}

/// Represents an aggregate value, built on top of other values.
pub trait AggregateValue: BasicValue {
    /// Returns an enum containing a typed version of the `AggregateValue`.
    fn as_aggregate_value_enum(&self) -> AggregateValueEnum {
        AggregateValueEnum::new(self.as_value_ref())
    }

    // REVIEW: How does LLVM treat out of bound index? Maybe we should return an Option?
    // or is that only in bounds GEP
    // REVIEW: Should this be AggregatePointerValue?
    fn const_extract_value(&self, indexes: &mut [u32]) -> BasicValueEnum {
        let value = unsafe {
            LLVMConstExtractValue(self.as_value_ref(), indexes.as_mut_ptr(), indexes.len() as u32)
        };

        BasicValueEnum::new(value)
    }
}

/// Represents a basic value, which can be used both by itself, or in an `AggregateValue`.
pub trait BasicValue: AnyValue {
    /// Returns an enum containing a typed version of the `BasicValue`.
    fn as_basic_value_enum(&self) -> BasicValueEnum {
        BasicValueEnum::new(self.as_value_ref())
    }
}

/// Represents a value which is permitted in integer math operations
pub trait IntMathValue: BasicValue {
    type BaseType: IntMathType;
    fn new(value: LLVMValueRef) -> Self;
}

/// Represents a value which is permitted in floating point math operations
pub trait FloatMathValue: BasicValue {
    type BaseType: FloatMathType;
    fn new(value: LLVMValueRef) -> Self;
}

pub trait PointerMathValue: BasicValue {
    type BaseType: PointerMathType;
    fn new(value: LLVMValueRef) -> Self;
}

/// Defines any struct wrapping an LLVM value.
pub trait AnyValue: AsValueRef + Debug {
    /// Returns an enum containing a typed version of `AnyValue`.
    fn as_any_value_enum(&self) -> AnyValueEnum {
        AnyValueEnum::new(self.as_value_ref())
    }
}

trait_value_set! {AggregateValue: ArrayValue, AggregateValueEnum, StructValue}
trait_value_set! {AnyValue: AnyValueEnum, BasicValueEnum, AggregateValueEnum, ArrayValue, IntValue, FloatValue, GlobalValue, PhiValue, PointerValue, FunctionValue, StructValue, VectorValue, InstructionValue}
trait_value_set! {BasicValue: ArrayValue, BasicValueEnum, AggregateValueEnum, IntValue, FloatValue, GlobalValue, StructValue, PointerValue, VectorValue}
math_trait_value_set! {IntMathValue: (IntValue => IntType), (VectorValue => VectorType)}
math_trait_value_set! {FloatMathValue: (FloatValue => FloatType), (VectorValue => VectorType)}
math_trait_value_set! {PointerMathValue: (PointerValue => PointerType), (VectorValue => VectorType)}
