use llvm_sys::core::{LLVMTypeOf, LLVMGetTypeKind};
use llvm_sys::LLVMTypeKind;
use llvm_sys::prelude::LLVMValueRef;

use types::BasicTypeEnum;
use values::traits::AsValueRef;
use values::{IntValue, FunctionValue, PointerValue, VectorValue, ArrayValue, StructValue, FloatValue, PhiValue, InstructionValue};

macro_rules! enum_value_set {
    ($enum_name:ident: $($args:ident),*) => (
        #[derive(Debug, EnumAsGetters, EnumIntoGetters, EnumIsA)]
        pub enum $enum_name {
            $(
                $args($args),
            )*
        }

        impl AsValueRef for $enum_name {
            fn as_value_ref(&self) -> LLVMValueRef {
                match *self {
                    $(
                        $enum_name::$args(ref t) => t.as_value_ref(),
                    )*
                }
            }
        }

        $(
            impl From<$args> for $enum_name {
                fn from(value: $args) -> $enum_name {
                    $enum_name::$args(value)
                }
            }
        )*

        // REVIEW: Possible encompassing methods to implement:
        // as_instruction, is_sized
    );
}

enum_value_set! {AggregateValueEnum: ArrayValue, StructValue}
enum_value_set! {AnyValueEnum: ArrayValue, IntValue, FloatValue, PhiValue, FunctionValue, PointerValue, StructValue, VectorValue}
enum_value_set! {BasicValueEnum: ArrayValue, IntValue, FloatValue, PointerValue, StructValue, VectorValue}

impl BasicValueEnum {
    pub(crate) fn new(value: LLVMValueRef) -> BasicValueEnum {
        let type_kind = unsafe {
            LLVMGetTypeKind(LLVMTypeOf(value))
        };

        match type_kind {
            LLVMTypeKind::LLVMFloatTypeKind |
            LLVMTypeKind::LLVMFP128TypeKind |
            LLVMTypeKind::LLVMDoubleTypeKind |
            LLVMTypeKind::LLVMHalfTypeKind |
            LLVMTypeKind::LLVMX86_FP80TypeKind |
            LLVMTypeKind::LLVMPPC_FP128TypeKind => BasicValueEnum::FloatValue(FloatValue::new(value)),
            LLVMTypeKind::LLVMIntegerTypeKind => BasicValueEnum::IntValue(IntValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => BasicValueEnum::StructValue(StructValue::new(value)),
            LLVMTypeKind::LLVMPointerTypeKind => BasicValueEnum::PointerValue(PointerValue::new(value)),
            LLVMTypeKind::LLVMArrayTypeKind => BasicValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMVectorTypeKind => BasicValueEnum::VectorValue(VectorValue::new(value)),
            _ => unreachable!("Unsupported type"),
        }
    }

    pub fn get_type(&self) -> BasicTypeEnum {
        let type_ = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        BasicTypeEnum::new(type_)
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        match *self {
            BasicValueEnum::ArrayValue(ref val) => val.as_instruction(),
            BasicValueEnum::IntValue(ref val) => val.as_instruction(),
            BasicValueEnum::FloatValue(ref val) => val.as_instruction(),
            BasicValueEnum::StructValue(ref val) => val.as_instruction(),
            BasicValueEnum::PointerValue(ref val) => val.as_instruction(),
            BasicValueEnum::VectorValue(ref val) => val.as_instruction(),
        }
    }
}
