use llvm_sys::prelude::LLVMValueRef;

#[llvm_versions(12..)]
use llvm_sys::core::LLVMIsPoison;

use std::fmt::Debug;

use crate::support::LLVMString;
use crate::types::{
    FloatMathType, FloatType, IntMathType, IntType, PointerMathType, PointerType, ScalableVectorType, VectorType,
};
use crate::values::{
    AggregateValueEnum, AnyValueEnum, ArrayValue, BasicValueEnum, BasicValueUse, CallSiteValue, FloatValue,
    FunctionValue, GlobalValue, InstructionValue, IntValue, PhiValue, PointerValue, ScalableVectorValue, StructValue,
    Value, VectorValue,
};

use super::{BasicMetadataValueEnum, MetadataValue};

// This is an ugly privacy hack so that Type can stay private to this module
// and so that super traits using this trait will be not be implementable
// outside this library
pub unsafe trait AsValueRef {
    fn as_value_ref(&self) -> LLVMValueRef;
}

macro_rules! trait_value_set {
    ($trait_name:ident: $($args:ident),*) => (
        $(
            unsafe impl<'ctx> $trait_name<'ctx> for $args<'ctx> {}
        )*

        // REVIEW: Possible encompassing methods to implement:
        // as_instruction, is_sized, ge/set metadata methods
    );
}

macro_rules! math_trait_value_set {
    ($trait_name:ident: $(($value_type:ident => $base_type:ident)),*) => (
        $(
            unsafe impl<'ctx> $trait_name<'ctx> for $value_type<'ctx> {
                type BaseType = $base_type<'ctx>;
                unsafe fn new(value: LLVMValueRef) -> $value_type<'ctx> {
                    unsafe {
                        $value_type::new(value)
                    }
                }
            }
        )*
    )
}

macro_rules! base_trait_value_set {
    ($trait_name:ident: $($value_type:ident),*) => (
        $(
            unsafe impl<'ctx> $trait_name<'ctx> for $value_type<'ctx> {
                unsafe fn new(value: LLVMValueRef) -> $value_type<'ctx> {
                    unsafe {
                        $value_type::new(value)
                    }
                }
            }
        )*
    )
}

/// Represents an aggregate value, built on top of other values.
pub unsafe trait AggregateValue<'ctx>: BasicValue<'ctx> {
    /// Returns an enum containing a typed version of the `AggregateValue`.
    fn as_aggregate_value_enum(&self) -> AggregateValueEnum<'ctx> {
        unsafe { AggregateValueEnum::new(self.as_value_ref()) }
    }

    // REVIEW: How does LLVM treat out of bound index? Maybe we should return an Option?
    // or is that only in bounds GEP
    // REVIEW: Should this be AggregatePointerValue?
    #[llvm_versions(..=14)]
    fn const_extract_value(&self, indexes: &mut [u32]) -> BasicValueEnum<'ctx> {
        use llvm_sys::core::LLVMConstExtractValue;

        unsafe {
            BasicValueEnum::new(LLVMConstExtractValue(
                self.as_value_ref(),
                indexes.as_mut_ptr(),
                indexes.len() as u32,
            ))
        }
    }

    // SubTypes: value should really be T in self: VectorValue<T> I think
    #[llvm_versions(..=14)]
    fn const_insert_value<BV: BasicValue<'ctx>>(&self, value: BV, indexes: &mut [u32]) -> BasicValueEnum<'ctx> {
        use llvm_sys::core::LLVMConstInsertValue;

        unsafe {
            BasicValueEnum::new(LLVMConstInsertValue(
                self.as_value_ref(),
                value.as_value_ref(),
                indexes.as_mut_ptr(),
                indexes.len() as u32,
            ))
        }
    }
}

/// Represents a basic value, which can be used both by itself, or in an `AggregateValue`.
pub unsafe trait BasicValue<'ctx>: AnyValue<'ctx> {
    /// Returns an enum containing a typed version of the `BasicValue`.
    fn as_basic_value_enum(&self) -> BasicValueEnum<'ctx> {
        unsafe { BasicValueEnum::new(self.as_value_ref()) }
    }

    /// Most `BasicValue`s are the byproduct of an instruction
    /// and so are convertible into an `InstructionValue`
    fn as_instruction_value(&self) -> Option<InstructionValue<'ctx>> {
        let value = unsafe { Value::new(self.as_value_ref()) };

        if !value.is_instruction() {
            return None;
        }

        unsafe { Some(InstructionValue::new(self.as_value_ref())) }
    }

    fn get_first_use(&self) -> Option<BasicValueUse> {
        unsafe { Value::new(self.as_value_ref()).get_first_use() }
    }

    /// Sets the name of a `BasicValue`. If the value is a constant, this is a noop.
    fn set_name(&self, name: &str) {
        unsafe { Value::new(self.as_value_ref()).set_name(name) }
    }

    // REVIEW: Possible encompassing methods to implement:
    // get/set metadata
}

/// Represents a value which is permitted in integer math operations
pub unsafe trait IntMathValue<'ctx>: BasicValue<'ctx> {
    type BaseType: IntMathType<'ctx>;
    unsafe fn new(value: LLVMValueRef) -> Self;
}

/// Represents a value which is permitted in floating point math operations
pub unsafe trait FloatMathValue<'ctx>: BasicValue<'ctx> {
    type BaseType: FloatMathType<'ctx>;
    unsafe fn new(value: LLVMValueRef) -> Self;
}

pub unsafe trait PointerMathValue<'ctx>: BasicValue<'ctx> {
    type BaseType: PointerMathType<'ctx>;
    unsafe fn new(value: LLVMValueRef) -> Self;
}

/// Represents a value which is permitted in vector operations, either fixed or scalable
pub unsafe trait VectorBaseValue<'ctx>: BasicValue<'ctx> {
    unsafe fn new(value: LLVMValueRef) -> Self;
}

// REVIEW: print_to_string might be a good candidate to live here?
/// Defines any struct wrapping an LLVM value.
pub unsafe trait AnyValue<'ctx>: AsValueRef + Debug {
    /// Returns an enum containing a typed version of `AnyValue`.
    fn as_any_value_enum(&self) -> AnyValueEnum<'ctx> {
        unsafe { AnyValueEnum::new(self.as_value_ref()) }
    }

    /// Prints a value to a `LLVMString`
    fn print_to_string(&self) -> LLVMString {
        unsafe { Value::new(self.as_value_ref()).print_to_string() }
    }

    /// Returns whether the value is `poison`
    #[llvm_versions(12..)]
    fn is_poison(&self) -> bool {
        unsafe { LLVMIsPoison(self.as_value_ref()) == 1 }
    }
}

trait_value_set! {AggregateValue: ArrayValue, AggregateValueEnum, StructValue}
trait_value_set! {AnyValue: AnyValueEnum, BasicValueEnum, BasicMetadataValueEnum, AggregateValueEnum, ArrayValue, IntValue, FloatValue, GlobalValue, PhiValue, PointerValue, FunctionValue, StructValue, VectorValue, ScalableVectorValue, InstructionValue, CallSiteValue, MetadataValue}
trait_value_set! {BasicValue: ArrayValue, BasicValueEnum, AggregateValueEnum, IntValue, FloatValue, GlobalValue, StructValue, PointerValue, VectorValue, ScalableVectorValue}
math_trait_value_set! {IntMathValue: (IntValue => IntType), (VectorValue => VectorType), (ScalableVectorValue => ScalableVectorType), (PointerValue => IntType)}
math_trait_value_set! {FloatMathValue: (FloatValue => FloatType), (VectorValue => VectorType), (ScalableVectorValue => ScalableVectorType)}
math_trait_value_set! {PointerMathValue: (PointerValue => PointerType), (VectorValue => VectorType), (ScalableVectorValue => ScalableVectorType)}
base_trait_value_set! {VectorBaseValue: VectorValue, ScalableVectorValue}
