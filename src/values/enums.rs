use llvm_sys::core::{LLVMIsAInstruction, LLVMTypeOf, LLVMGetTypeKind};
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
            LLVMTypeKind::LLVMVoidTypeKind => {
                if unsafe { LLVMIsAInstruction(value) }.is_null() {
                    panic!("Void value isn't an instruction.");
                }
                AnyValueEnum::InstructionValue(InstructionValue::new(value))
            },
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

    pub fn into_array_value(self) -> ArrayValue<'ctx> {
        if let AnyValueEnum::ArrayValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_int_value(self) -> IntValue<'ctx> {
        if let AnyValueEnum::IntValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_float_value(self) -> FloatValue<'ctx> {
        if let AnyValueEnum::FloatValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_phi_value(self) -> PhiValue<'ctx> {
        if let AnyValueEnum::PhiValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_function_value(self) -> FunctionValue<'ctx> {
        if let AnyValueEnum::FunctionValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_pointer_value(self) -> PointerValue<'ctx> {
        if let AnyValueEnum::PointerValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_struct_value(self) -> StructValue<'ctx> {
        if let AnyValueEnum::StructValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_vector_value(self) -> VectorValue<'ctx> {
        if let AnyValueEnum::VectorValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_instruction_value(self) -> InstructionValue<'ctx> {
        if let AnyValueEnum::InstructionValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }
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

    pub fn into_array_value(self) -> ArrayValue<'ctx> {
        if let BasicValueEnum::ArrayValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_int_value(self) -> IntValue<'ctx> {
        if let BasicValueEnum::IntValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_float_value(self) -> FloatValue<'ctx> {
        if let BasicValueEnum::FloatValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_pointer_value(self) -> PointerValue<'ctx> {
        if let BasicValueEnum::PointerValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_struct_value(self) -> StructValue<'ctx> {
        if let BasicValueEnum::StructValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_vector_value(self) -> VectorValue<'ctx> {
        if let BasicValueEnum::VectorValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }
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

    pub fn is_array_value(self) -> bool {
        if let AggregateValueEnum::ArrayValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_struct_value(self) -> bool {
        if let AggregateValueEnum::StructValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn into_array_value(self) -> ArrayValue<'ctx> {
        if let AggregateValueEnum::ArrayValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_struct_value(self) -> StructValue<'ctx> {
        if let AggregateValueEnum::StructValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
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

    pub fn is_array_value(self) -> bool {
        if let BasicMetadataValueEnum::ArrayValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_int_value(self) -> bool {
        if let BasicMetadataValueEnum::IntValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_float_value(self) -> bool {
        if let BasicMetadataValueEnum::FloatValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_pointer_value(self) -> bool {
        if let BasicMetadataValueEnum::PointerValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_struct_value(self) -> bool {
        if let BasicMetadataValueEnum::StructValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_vector_value(self) -> bool {
        if let BasicMetadataValueEnum::VectorValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_metadata_value(self) -> bool {
        if let BasicMetadataValueEnum::MetadataValue(_) = self {
            true
        } else {
            false
        }
    }

    pub fn into_array_value(self) -> ArrayValue<'ctx> {
        if let BasicMetadataValueEnum::ArrayValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_int_value(self) -> IntValue<'ctx> {
        if let BasicMetadataValueEnum::IntValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_float_value(self) -> FloatValue<'ctx> {
        if let BasicMetadataValueEnum::FloatValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_pointer_value(self) -> PointerValue<'ctx> {
        if let BasicMetadataValueEnum::PointerValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_struct_value(self) -> StructValue<'ctx> {
        if let BasicMetadataValueEnum::StructValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_vector_value(self) -> VectorValue<'ctx> {
        if let BasicMetadataValueEnum::VectorValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }

    pub fn into_metadata_value(self) -> MetadataValue<'ctx> {
        if let BasicMetadataValueEnum::MetadataValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected a different variant", self)
        }
    }
}

impl<'ctx> From<BasicValueEnum<'ctx>> for AnyValueEnum<'ctx> {
    fn from(value: BasicValueEnum) -> AnyValueEnum {
        AnyValueEnum::new(value.as_value_ref())
    }
}
