use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::core::{LLVMConstExtractValue, LLVMConstInsertValue};

use std::fmt::Debug;

use crate::values::{ArrayValue, AggregateValueEnum, BasicValueUse, CallSiteValue, GlobalValue, StructValue, BasicValueEnum, AnyValueEnum, IntValue, FloatValue, PointerValue, PhiValue, VectorValue, FunctionValue, InstructionValue, Value};
use crate::types::{IntMathType, FloatMathType, PointerMathType, IntType, FloatType, PointerType, VectorType};
use crate::support::LLVMString;

// This is an ugly privacy hack so that Type can stay private to this module
// and so that super traits using this trait will be not be implementable
// outside this library
pub trait AsValueRef {
    fn as_value_ref(&self) -> LLVMValueRef;
}

macro_rules! trait_value_set {
    ($trait_name:ident: $($args:ident),*) => (
        $(
            impl<'ctx> $trait_name<'ctx> for $args<'ctx> {}
        )*

        // REVIEW: Possible encompassing methods to implement:
        // as_instruction, is_sized, ge/set metadata methods
    );
}

macro_rules! math_trait_value_set {
    ($trait_name:ident: $(($value_type:ident => $base_type:ident)),*) => (
        $(
            impl<'ctx> $trait_name<'ctx> for $value_type<'ctx> {
                type BaseType = $base_type<'ctx>;
                fn new(value: LLVMValueRef) -> Self {
                    $value_type::new(value)
                }
            }
        )*
    )
}

/// Represents an aggregate value, built on top of other values.
pub trait AggregateValue<'ctx>: BasicValue<'ctx> {
    /// Returns an enum containing a typed version of the `AggregateValue`.
    fn as_aggregate_value_enum(&self) -> AggregateValueEnum<'ctx> {
        AggregateValueEnum::new(self.as_value_ref())
    }

    // REVIEW: How does LLVM treat out of bound index? Maybe we should return an Option?
    // or is that only in bounds GEP
    // REVIEW: Should this be AggregatePointerValue?
    fn const_extract_value(&self, indexes: &mut [u32]) -> BasicValueEnum<'ctx> {
        let value = unsafe {
            LLVMConstExtractValue(self.as_value_ref(), indexes.as_mut_ptr(), indexes.len() as u32)
        };

        BasicValueEnum::new(value)
    }

    // SubTypes: value should really be T in self: VectorValue<T> I think
    fn const_insert_value<BV: BasicValue<'ctx>>(&self, value: BV, indexes: &mut [u32]) -> BasicValueEnum<'ctx> {
        let value = unsafe {
            LLVMConstInsertValue(self.as_value_ref(), value.as_value_ref(), indexes.as_mut_ptr(), indexes.len() as u32)
        };

        BasicValueEnum::new(value)
    }
}

/// Represents a basic value, which can be used both by itself, or in an `AggregateValue`.
pub trait BasicValue<'ctx>: AnyValue<'ctx> {
    /// Returns an enum containing a typed version of the `BasicValue`.
    fn as_basic_value_enum(&self) -> BasicValueEnum<'ctx> {
        BasicValueEnum::new(self.as_value_ref())
    }

    /// Most `BasicValue`s are the byproduct of an instruction
    /// and so are convertable into an `InstructionValue`
    fn as_instruction_value(&self) -> Option<InstructionValue<'ctx>> {
        let value = Value::new(self.as_value_ref());

        if !value.is_instruction() {
            return None;
        }

        Some(InstructionValue::new(self.as_value_ref()))
    }

    fn get_first_use(&self) -> Option<BasicValueUse> {
        Value::new(self.as_value_ref()).get_first_use()
    }

    /// Sets the name of a `BasicValue`. If the value is a constant, this is a noop.
    fn set_name(&self, name: &str) {
        Value::new(self.as_value_ref()).set_name(name)
    }

    // REVIEW: Possible encompassing methods to implement:
    // get/set metadata
}

/// Represents a value which is permitted in integer math operations
pub trait IntMathValue<'ctx>: BasicValue<'ctx> {
    type BaseType: IntMathType<'ctx>;
    fn new(value: LLVMValueRef) -> Self;
}

/// Represents a value which is permitted in floating point math operations
pub trait FloatMathValue<'ctx>: BasicValue<'ctx> {
    type BaseType: FloatMathType<'ctx>;
    fn new(value: LLVMValueRef) -> Self;
}

pub trait PointerMathValue<'ctx>: BasicValue<'ctx> {
    type BaseType: PointerMathType<'ctx>;
    fn new(value: LLVMValueRef) -> Self;
}

// REVIEW: print_to_string might be a good candidate to live here?
/// Defines any struct wrapping an LLVM value.
pub trait AnyValue<'ctx>: AsValueRef + Debug {
    /// Returns an enum containing a typed version of `AnyValue`.
    fn as_any_value_enum(&self) -> AnyValueEnum<'ctx> {
        AnyValueEnum::new(self.as_value_ref())
    }

    /// Prints a value to a `LLVMString`
    fn print_to_string(&self) -> LLVMString {
        Value::new(self.as_value_ref()).print_to_string()
    }
}

trait_value_set! {AggregateValue: ArrayValue, AggregateValueEnum, StructValue}
trait_value_set! {AnyValue: AnyValueEnum, BasicValueEnum, AggregateValueEnum, ArrayValue, IntValue, FloatValue, GlobalValue, PhiValue, PointerValue, FunctionValue, StructValue, VectorValue, InstructionValue, CallSiteValue}
trait_value_set! {BasicValue: ArrayValue, BasicValueEnum, AggregateValueEnum, IntValue, FloatValue, GlobalValue, StructValue, PointerValue, VectorValue}
math_trait_value_set! {IntMathValue: (IntValue => IntType), (VectorValue => VectorType)}
math_trait_value_set! {FloatMathValue: (FloatValue => FloatType), (VectorValue => VectorType)}
math_trait_value_set! {PointerMathValue: (PointerValue => PointerType), (VectorValue => VectorType)}
