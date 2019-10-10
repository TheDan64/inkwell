use llvm_sys::core::{LLVMTypeOf, LLVMGetTypeKind};
use llvm_sys::LLVMTypeKind;
use llvm_sys::prelude::LLVMValueRef;

use crate::types::{AnyTypeEnum, BasicTypeEnum};
use crate::values::traits::AsValueRef;
use crate::values::{IntValue, FunctionValue, PointerValue, VectorValue, ArrayValue, StructValue, FloatValue, PhiValue, InstructionValue, MetadataValue};

macro_rules! enum_value_set {
    ($enum_name:ident: $($args:ident),*) => (
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $enum_name<'ctx> {
            $(
                $args($args<'ctx>),
            )*
        }

        impl AsValueRef for $enum_name<'_> {
            fn as_value_ref(&self) -> LLVMValueRef {
                match *self {
                    $(
                        $enum_name::$args(ref t) => t.as_value_ref(),
                    )*
                }
            }
        }

        $(
            impl<'ctx> From<$args<'ctx>> for $enum_name<'ctx> {
                fn from(value: $args) -> $enum_name {
                    $enum_name::$args(value)
                }
            }

            impl<'ctx> PartialEq<$args<'ctx>> for $enum_name<'ctx> {
                fn eq(&self, other: &$args<'ctx>) -> bool {
                    self.as_value_ref() == other.as_value_ref()
                }
            }

            impl<'ctx> PartialEq<$enum_name<'ctx>> for $args<'ctx> {
                fn eq(&self, other: &$enum_name<'ctx>) -> bool {
                    self.as_value_ref() == other.as_value_ref()
                }
            }
        )*
    );
}

enum_value_set! {AggregateValueEnum: ArrayValue, StructValue}
enum_value_set! {AnyValueEnum: ArrayValue, IntValue, FloatValue, PhiValue, FunctionValue, PointerValue, StructValue, VectorValue, InstructionValue}
enum_value_set! {BasicValueEnum: ArrayValue, IntValue, FloatValue, PointerValue, StructValue, VectorValue}
enum_value_set! {BasicMetadataValueEnum: ArrayValue, IntValue, FloatValue, PointerValue, StructValue, VectorValue, MetadataValue}

impl<'ctx> AnyValueEnum<'ctx> {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        let type_kind = unsafe {
            LLVMGetTypeKind(LLVMTypeOf(value))
        };

        match type_kind {
            LLVMTypeKind::LLVMFloatTypeKind |
            LLVMTypeKind::LLVMFP128TypeKind |
            LLVMTypeKind::LLVMDoubleTypeKind |
            LLVMTypeKind::LLVMHalfTypeKind |
            LLVMTypeKind::LLVMX86_FP80TypeKind |
            LLVMTypeKind::LLVMPPC_FP128TypeKind => AnyValueEnum::FloatValue(FloatValue::new(value)),
            LLVMTypeKind::LLVMIntegerTypeKind => AnyValueEnum::IntValue(IntValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => AnyValueEnum::StructValue(StructValue::new(value)),
            LLVMTypeKind::LLVMPointerTypeKind => AnyValueEnum::PointerValue(PointerValue::new(value)),
            LLVMTypeKind::LLVMArrayTypeKind => AnyValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMVectorTypeKind => AnyValueEnum::VectorValue(VectorValue::new(value)),
            LLVMTypeKind::LLVMFunctionTypeKind => AnyValueEnum::FunctionValue(FunctionValue::new(value).unwrap()),
            LLVMTypeKind::LLVMVoidTypeKind => panic!("Void values shouldn't exist."),
            LLVMTypeKind::LLVMMetadataTypeKind => panic!("Metadata values are not supported as AnyValue's."),
            _ => panic!("The given type is not supported.")
        }
    }

    pub fn get_type(&self) -> AnyTypeEnum<'ctx> {
        let type_ = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        AnyTypeEnum::new(type_)
    }

    pub fn is_array_value(self) -> bool {
        if let AnyValueEnum::ArrayValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_int_value(self) -> bool {
        if let AnyValueEnum::IntValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_float_value(self) -> bool {
        if let AnyValueEnum::FloatValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_phi_value(self) -> bool {
        if let AnyValueEnum::PhiValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_function_value(self) -> bool {
        if let AnyValueEnum::FunctionValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_pointer_value(self) -> bool {
        if let AnyValueEnum::PointerValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_struct_value(self) -> bool {
        if let AnyValueEnum::StructValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_vector_value(self) -> bool {
        if let AnyValueEnum::VectorValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_instruction_value(self) -> bool {
        if let AnyValueEnum::InstructionValue(_) = self {
            true
        } else {
            false
        }
    }

    // TODO: into_x_value methods
}

impl<'ctx> BasicValueEnum<'ctx> {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
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
            _ => unreachable!("The given type is not a basic type."),
        }
    }

    pub fn get_type(&self) -> BasicTypeEnum<'ctx> {
        let type_ = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        BasicTypeEnum::new(type_)
    }

    pub fn is_array_value(self) -> bool {
        if let BasicValueEnum::ArrayValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_int_value(self) -> bool {
        if let BasicValueEnum::IntValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_float_value(self) -> bool {
        if let BasicValueEnum::FloatValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_pointer_value(self) -> bool {
        if let BasicValueEnum::PointerValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_struct_value(self) -> bool {
        if let BasicValueEnum::StructValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_vector_value(self) -> bool {
        if let BasicValueEnum::VectorValue(_) = self {
            true
        } else {
            false
        }
    }

    // TODO: into_x_value methods
}

impl<'ctx> AggregateValueEnum<'ctx> {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        let type_kind = unsafe {
            LLVMGetTypeKind(LLVMTypeOf(value))
        };

        match type_kind {
            LLVMTypeKind::LLVMArrayTypeKind => AggregateValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => AggregateValueEnum::StructValue(StructValue::new(value)),
            _ => unreachable!("The given type is not an aggregate type."),
        }
    }
}

impl<'ctx> BasicMetadataValueEnum<'ctx> {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        let type_kind = unsafe {
            LLVMGetTypeKind(LLVMTypeOf(value))
        };

        match type_kind {
            LLVMTypeKind::LLVMFloatTypeKind |
            LLVMTypeKind::LLVMFP128TypeKind |
            LLVMTypeKind::LLVMDoubleTypeKind |
            LLVMTypeKind::LLVMHalfTypeKind |
            LLVMTypeKind::LLVMX86_FP80TypeKind |
            LLVMTypeKind::LLVMPPC_FP128TypeKind => BasicMetadataValueEnum::FloatValue(FloatValue::new(value)),
            LLVMTypeKind::LLVMIntegerTypeKind => BasicMetadataValueEnum::IntValue(IntValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => BasicMetadataValueEnum::StructValue(StructValue::new(value)),
            LLVMTypeKind::LLVMPointerTypeKind => BasicMetadataValueEnum::PointerValue(PointerValue::new(value)),
            LLVMTypeKind::LLVMArrayTypeKind => BasicMetadataValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMVectorTypeKind => BasicMetadataValueEnum::VectorValue(VectorValue::new(value)),
            LLVMTypeKind::LLVMMetadataTypeKind => BasicMetadataValueEnum::MetadataValue(MetadataValue::new(value)),
            _ => unreachable!("Unsupported type"),
        }
    }
}

impl<'ctx> From<BasicValueEnum<'ctx>> for AnyValueEnum<'ctx> {
    fn from(value: BasicValueEnum) -> AnyValueEnum {
        AnyValueEnum::new(value.as_value_ref())
    }
}
