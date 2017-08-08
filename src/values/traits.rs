use llvm_sys::prelude::LLVMValueRef;

use values::{ArrayValue, AggregateValueEnum, StructValue, BasicValueEnum, AnyValueEnum, IntValue, FloatValue, PointerValue, PhiValue, VectorValue, FunctionValue};

// This is an ugly privacy hack so that Type can stay private to this module
// and so that super traits using this trait will be not be implementable
// outside this library
pub trait AsValueRef {
    fn as_value_ref(&self) -> LLVMValueRef;
}

macro_rules! trait_value_set {
    ($trait_name:ident: $($args:ident),*) => (
        pub trait $trait_name: AsValueRef {}

        $(
            impl $trait_name for $args {}
        )*

        // REVIEW: Possible encompassing methods to implement:
        // as_instruction, is_sized
    );
}

trait_value_set! {AggregateValue: ArrayValue, AggregateValueEnum, StructValue}
trait_value_set! {AnyValue: AnyValueEnum, BasicValueEnum, AggregateValueEnum, ArrayValue, IntValue, FloatValue, PhiValue, PointerValue, FunctionValue, StructValue, VectorValue}
trait_value_set! {BasicValue: ArrayValue, BasicValueEnum, AggregateValueEnum, IntValue, FloatValue, StructValue, PointerValue, VectorValue}
