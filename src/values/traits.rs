use llvm_sys::prelude::LLVMValueRef;

use std::fmt::Debug;

use values::{ArrayValue, AggregateValueEnum, GlobalValue, StructValue, BasicValueEnum, AnyValueEnum, IntValue, FloatValue, PointerValue, PhiValue, VectorValue, FunctionValue, InstructionValue};

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

/// Represents an aggregate value, built on top of other values.
pub trait AggregateValue: BasicValue {
    /// Returns an enum containing a typed version of the `AggregateValue`.
    fn as_aggregate_value_enum(&self) -> AggregateValueEnum {
        AggregateValueEnum::new(self.as_value_ref())
    }
}

/// Represents a basic value, which can be used both by itself, or in an `AggregateValue`.
pub trait BasicValue: AnyValue {
    /// Returns an enum containing a typed version of the `BasicValue`.
    fn as_basic_value_enum(&self) -> BasicValueEnum {
        BasicValueEnum::new(self.as_value_ref())
    }
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
