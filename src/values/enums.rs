use llvm_sys::core::{LLVMGetTypeKind, LLVMGetValueKind, LLVMIsAInstruction, LLVMTypeOf};
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::{LLVMTypeKind, LLVMValueKind};

use crate::types::{AnyTypeEnum, BasicTypeEnum};
use crate::values::traits::AsValueRef;
use crate::values::{
    ArrayValue, FloatValue, FunctionValue, InstructionValue, IntValue, MetadataValue, PhiValue, PointerValue,
    ScalableVectorValue, StructValue, VectorValue,
};

use std::convert::TryFrom;
use std::ffi::CStr;
use std::fmt::{self, Display};

use super::AnyValue;

macro_rules! enum_value_set {
    ($enum_name:ident: $($args:ident),*) => (
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $enum_name<'ctx> {
            $(
                $args($args<'ctx>),
            )*
        }

        unsafe impl AsValueRef for $enum_name<'_> {
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

            impl<'ctx> TryFrom<$enum_name<'ctx>> for $args<'ctx> {
                type Error = ();

                fn try_from(value: $enum_name<'ctx>) -> Result<Self, Self::Error> {
                    match value {
                        $enum_name::$args(ty) => Ok(ty),
                        _ => Err(()),
                    }
                }
            }
        )*
    );
}

enum_value_set! {AggregateValueEnum: ArrayValue, StructValue}
enum_value_set! {AnyValueEnum: ArrayValue, IntValue, FloatValue, PhiValue, FunctionValue, PointerValue, StructValue, VectorValue, ScalableVectorValue, InstructionValue, MetadataValue}
enum_value_set! {BasicValueEnum: ArrayValue, IntValue, FloatValue, PointerValue, StructValue, VectorValue, ScalableVectorValue}
enum_value_set! {BasicMetadataValueEnum: ArrayValue, IntValue, FloatValue, PointerValue, StructValue, VectorValue, ScalableVectorValue, MetadataValue}

impl<'ctx> AnyValueEnum<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of supported enum type options ([LLVMTypeKind]).
    pub unsafe fn new(value: LLVMValueRef) -> Self {
        match LLVMGetTypeKind(LLVMTypeOf(value)) {
            LLVMTypeKind::LLVMFloatTypeKind
            | LLVMTypeKind::LLVMFP128TypeKind
            | LLVMTypeKind::LLVMDoubleTypeKind
            | LLVMTypeKind::LLVMHalfTypeKind
            | LLVMTypeKind::LLVMX86_FP80TypeKind
            | LLVMTypeKind::LLVMPPC_FP128TypeKind => AnyValueEnum::FloatValue(FloatValue::new(value)),
            LLVMTypeKind::LLVMIntegerTypeKind => AnyValueEnum::IntValue(IntValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => AnyValueEnum::StructValue(StructValue::new(value)),
            LLVMTypeKind::LLVMPointerTypeKind => match LLVMGetValueKind(value) {
                LLVMValueKind::LLVMFunctionValueKind => AnyValueEnum::FunctionValue(FunctionValue::new(value).unwrap()),
                _ => AnyValueEnum::PointerValue(PointerValue::new(value)),
            },
            LLVMTypeKind::LLVMArrayTypeKind => AnyValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMVectorTypeKind => AnyValueEnum::VectorValue(VectorValue::new(value)),
            #[cfg(any(
                feature = "llvm11-0",
                feature = "llvm12-0",
                feature = "llvm13-0",
                feature = "llvm14-0",
                feature = "llvm15-0",
                feature = "llvm16-0",
                feature = "llvm17-0",
                feature = "llvm18-1"
            ))]
            LLVMTypeKind::LLVMScalableVectorTypeKind => {
                AnyValueEnum::ScalableVectorValue(ScalableVectorValue::new(value))
            },
            LLVMTypeKind::LLVMFunctionTypeKind => AnyValueEnum::FunctionValue(FunctionValue::new(value).unwrap()),
            LLVMTypeKind::LLVMVoidTypeKind => {
                if LLVMIsAInstruction(value).is_null() {
                    panic!("Void value isn't an instruction.");
                }
                AnyValueEnum::InstructionValue(InstructionValue::new(value))
            },
            LLVMTypeKind::LLVMMetadataTypeKind => panic!("Metadata values are not supported as AnyValue's."),
            _ => panic!("The given type is not supported."),
        }
    }

    pub fn get_type(&self) -> AnyTypeEnum<'ctx> {
        unsafe { AnyTypeEnum::new(LLVMTypeOf(self.as_value_ref())) }
    }

    pub fn is_array_value(self) -> bool {
        matches!(self, AnyValueEnum::ArrayValue(_))
    }

    pub fn is_int_value(self) -> bool {
        matches!(self, AnyValueEnum::IntValue(_))
    }

    pub fn is_float_value(self) -> bool {
        matches!(self, AnyValueEnum::FloatValue(_))
    }

    pub fn is_phi_value(self) -> bool {
        matches!(self, AnyValueEnum::PhiValue(_))
    }

    pub fn is_function_value(self) -> bool {
        matches!(self, AnyValueEnum::FunctionValue(_))
    }

    pub fn is_pointer_value(self) -> bool {
        matches!(self, AnyValueEnum::PointerValue(_))
    }

    pub fn is_struct_value(self) -> bool {
        matches!(self, AnyValueEnum::StructValue(_))
    }

    pub fn is_vector_value(self) -> bool {
        matches!(self, AnyValueEnum::VectorValue(_))
    }

    pub fn is_scalable_vector_value(self) -> bool {
        matches!(self, AnyValueEnum::ScalableVectorValue(_))
    }

    pub fn is_instruction_value(self) -> bool {
        matches!(self, AnyValueEnum::InstructionValue(_))
    }

    #[track_caller]
    pub fn into_array_value(self) -> ArrayValue<'ctx> {
        if let AnyValueEnum::ArrayValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the ArrayValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_int_value(self) -> IntValue<'ctx> {
        if let AnyValueEnum::IntValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the IntValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_float_value(self) -> FloatValue<'ctx> {
        if let AnyValueEnum::FloatValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the FloatValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_phi_value(self) -> PhiValue<'ctx> {
        if let AnyValueEnum::PhiValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the PhiValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_function_value(self) -> FunctionValue<'ctx> {
        if let AnyValueEnum::FunctionValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the FunctionValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_pointer_value(self) -> PointerValue<'ctx> {
        if let AnyValueEnum::PointerValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the PointerValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_struct_value(self) -> StructValue<'ctx> {
        if let AnyValueEnum::StructValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the StructValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_vector_value(self) -> VectorValue<'ctx> {
        if let AnyValueEnum::VectorValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the VectorValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_scalable_vector_value(self) -> ScalableVectorValue<'ctx> {
        if let AnyValueEnum::ScalableVectorValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the ScalableVectorValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_instruction_value(self) -> InstructionValue<'ctx> {
        if let AnyValueEnum::InstructionValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the InstructionValue variant", self)
        }
    }
}

impl<'ctx> BasicValueEnum<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of supported enum type options ([LLVMTypeKind]).
    pub unsafe fn new(value: LLVMValueRef) -> Self {
        match LLVMGetTypeKind(LLVMTypeOf(value)) {
            LLVMTypeKind::LLVMFloatTypeKind
            | LLVMTypeKind::LLVMFP128TypeKind
            | LLVMTypeKind::LLVMDoubleTypeKind
            | LLVMTypeKind::LLVMHalfTypeKind
            | LLVMTypeKind::LLVMX86_FP80TypeKind
            | LLVMTypeKind::LLVMPPC_FP128TypeKind => BasicValueEnum::FloatValue(FloatValue::new(value)),
            LLVMTypeKind::LLVMIntegerTypeKind => BasicValueEnum::IntValue(IntValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => BasicValueEnum::StructValue(StructValue::new(value)),
            LLVMTypeKind::LLVMPointerTypeKind => BasicValueEnum::PointerValue(PointerValue::new(value)),
            LLVMTypeKind::LLVMArrayTypeKind => BasicValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMVectorTypeKind => BasicValueEnum::VectorValue(VectorValue::new(value)),
            #[cfg(any(
                feature = "llvm11-0",
                feature = "llvm12-0",
                feature = "llvm13-0",
                feature = "llvm14-0",
                feature = "llvm15-0",
                feature = "llvm16-0",
                feature = "llvm17-0",
                feature = "llvm18-1"
            ))]
            LLVMTypeKind::LLVMScalableVectorTypeKind => {
                BasicValueEnum::ScalableVectorValue(ScalableVectorValue::new(value))
            },
            _ => unreachable!("The given type is not a basic type."),
        }
    }

    /// Get the name of the `BasicValueEnum`.
    pub fn get_name(&self) -> &CStr {
        match self {
            BasicValueEnum::ArrayValue(v) => v.get_name(),
            BasicValueEnum::IntValue(v) => v.get_name(),
            BasicValueEnum::FloatValue(v) => v.get_name(),
            BasicValueEnum::PointerValue(v) => v.get_name(),
            BasicValueEnum::StructValue(v) => v.get_name(),
            BasicValueEnum::VectorValue(v) => v.get_name(),
            BasicValueEnum::ScalableVectorValue(v) => v.get_name(),
        }
    }

    /// Set name of the `BasicValueEnum`.
    pub fn set_name(&self, name: &str) {
        match self {
            BasicValueEnum::ArrayValue(v) => v.set_name(name),
            BasicValueEnum::IntValue(v) => v.set_name(name),
            BasicValueEnum::FloatValue(v) => v.set_name(name),
            BasicValueEnum::PointerValue(v) => v.set_name(name),
            BasicValueEnum::StructValue(v) => v.set_name(name),
            BasicValueEnum::VectorValue(v) => v.set_name(name),
            BasicValueEnum::ScalableVectorValue(v) => v.set_name(name),
        }
    }

    pub fn get_type(&self) -> BasicTypeEnum<'ctx> {
        unsafe { BasicTypeEnum::new(LLVMTypeOf(self.as_value_ref())) }
    }

    pub fn is_array_value(self) -> bool {
        matches!(self, BasicValueEnum::ArrayValue(_))
    }

    pub fn is_int_value(self) -> bool {
        matches!(self, BasicValueEnum::IntValue(_))
    }

    pub fn is_float_value(self) -> bool {
        matches!(self, BasicValueEnum::FloatValue(_))
    }

    pub fn is_pointer_value(self) -> bool {
        matches!(self, BasicValueEnum::PointerValue(_))
    }

    pub fn is_struct_value(self) -> bool {
        matches!(self, BasicValueEnum::StructValue(_))
    }

    pub fn is_vector_value(self) -> bool {
        matches!(self, BasicValueEnum::VectorValue(_))
    }

    pub fn is_scalable_vector_value(self) -> bool {
        matches!(self, BasicValueEnum::ScalableVectorValue(_))
    }

    #[track_caller]
    pub fn into_array_value(self) -> ArrayValue<'ctx> {
        if let BasicValueEnum::ArrayValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the ArrayValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_int_value(self) -> IntValue<'ctx> {
        if let BasicValueEnum::IntValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the IntValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_float_value(self) -> FloatValue<'ctx> {
        if let BasicValueEnum::FloatValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the FloatValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_pointer_value(self) -> PointerValue<'ctx> {
        if let BasicValueEnum::PointerValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected PointerValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_struct_value(self) -> StructValue<'ctx> {
        if let BasicValueEnum::StructValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the StructValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_vector_value(self) -> VectorValue<'ctx> {
        if let BasicValueEnum::VectorValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the VectorValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_scalable_vector_value(self) -> ScalableVectorValue<'ctx> {
        if let BasicValueEnum::ScalableVectorValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the ScalableVectorValue variant", self)
        }
    }
}

impl<'ctx> AggregateValueEnum<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of supported aggregate type enum options ([LLVMTypeKind]).
    pub unsafe fn new(value: LLVMValueRef) -> Self {
        match LLVMGetTypeKind(LLVMTypeOf(value)) {
            LLVMTypeKind::LLVMArrayTypeKind => AggregateValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => AggregateValueEnum::StructValue(StructValue::new(value)),
            _ => unreachable!("The given type is not an aggregate type."),
        }
    }

    pub fn is_array_value(self) -> bool {
        matches!(self, AggregateValueEnum::ArrayValue(_))
    }

    pub fn is_struct_value(self) -> bool {
        matches!(self, AggregateValueEnum::StructValue(_))
    }

    #[track_caller]
    pub fn into_array_value(self) -> ArrayValue<'ctx> {
        if let AggregateValueEnum::ArrayValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the ArrayValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_struct_value(self) -> StructValue<'ctx> {
        if let AggregateValueEnum::StructValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the StructValue variant", self)
        }
    }
}

impl<'ctx> BasicMetadataValueEnum<'ctx> {
    pub(crate) unsafe fn new(value: LLVMValueRef) -> Self {
        match LLVMGetTypeKind(LLVMTypeOf(value)) {
            LLVMTypeKind::LLVMFloatTypeKind
            | LLVMTypeKind::LLVMFP128TypeKind
            | LLVMTypeKind::LLVMDoubleTypeKind
            | LLVMTypeKind::LLVMHalfTypeKind
            | LLVMTypeKind::LLVMX86_FP80TypeKind
            | LLVMTypeKind::LLVMPPC_FP128TypeKind => BasicMetadataValueEnum::FloatValue(FloatValue::new(value)),
            LLVMTypeKind::LLVMIntegerTypeKind => BasicMetadataValueEnum::IntValue(IntValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => BasicMetadataValueEnum::StructValue(StructValue::new(value)),
            LLVMTypeKind::LLVMPointerTypeKind => BasicMetadataValueEnum::PointerValue(PointerValue::new(value)),
            LLVMTypeKind::LLVMArrayTypeKind => BasicMetadataValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMVectorTypeKind => BasicMetadataValueEnum::VectorValue(VectorValue::new(value)),
            #[cfg(any(
                feature = "llvm11-0",
                feature = "llvm12-0",
                feature = "llvm13-0",
                feature = "llvm14-0",
                feature = "llvm15-0",
                feature = "llvm16-0",
                feature = "llvm17-0",
                feature = "llvm18-1"
            ))]
            LLVMTypeKind::LLVMScalableVectorTypeKind => {
                BasicMetadataValueEnum::ScalableVectorValue(ScalableVectorValue::new(value))
            },
            LLVMTypeKind::LLVMMetadataTypeKind => BasicMetadataValueEnum::MetadataValue(MetadataValue::new(value)),
            _ => unreachable!("Unsupported type"),
        }
    }

    pub fn is_array_value(self) -> bool {
        matches!(self, BasicMetadataValueEnum::ArrayValue(_))
    }

    pub fn is_int_value(self) -> bool {
        matches!(self, BasicMetadataValueEnum::IntValue(_))
    }

    pub fn is_float_value(self) -> bool {
        matches!(self, BasicMetadataValueEnum::FloatValue(_))
    }

    pub fn is_pointer_value(self) -> bool {
        matches!(self, BasicMetadataValueEnum::PointerValue(_))
    }

    pub fn is_struct_value(self) -> bool {
        matches!(self, BasicMetadataValueEnum::StructValue(_))
    }

    pub fn is_vector_value(self) -> bool {
        matches!(self, BasicMetadataValueEnum::VectorValue(_))
    }

    pub fn is_scalable_vector_value(self) -> bool {
        matches!(self, BasicMetadataValueEnum::ScalableVectorValue(_))
    }

    pub fn is_metadata_value(self) -> bool {
        matches!(self, BasicMetadataValueEnum::MetadataValue(_))
    }

    #[track_caller]
    pub fn into_array_value(self) -> ArrayValue<'ctx> {
        if let BasicMetadataValueEnum::ArrayValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the ArrayValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_int_value(self) -> IntValue<'ctx> {
        if let BasicMetadataValueEnum::IntValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the IntValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_float_value(self) -> FloatValue<'ctx> {
        if let BasicMetadataValueEnum::FloatValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected FloatValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_pointer_value(self) -> PointerValue<'ctx> {
        if let BasicMetadataValueEnum::PointerValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the PointerValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_struct_value(self) -> StructValue<'ctx> {
        if let BasicMetadataValueEnum::StructValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the StructValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_vector_value(self) -> VectorValue<'ctx> {
        if let BasicMetadataValueEnum::VectorValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the VectorValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_scalable_vector_value(self) -> ScalableVectorValue<'ctx> {
        if let BasicMetadataValueEnum::ScalableVectorValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected the ScalableVectorValue variant", self)
        }
    }

    #[track_caller]
    pub fn into_metadata_value(self) -> MetadataValue<'ctx> {
        if let BasicMetadataValueEnum::MetadataValue(v) = self {
            v
        } else {
            panic!("Found {:?} but expected MetaData variant", self)
        }
    }
}

impl<'ctx> From<BasicValueEnum<'ctx>> for AnyValueEnum<'ctx> {
    fn from(value: BasicValueEnum<'ctx>) -> Self {
        unsafe { AnyValueEnum::new(value.as_value_ref()) }
    }
}

impl<'ctx> From<BasicValueEnum<'ctx>> for BasicMetadataValueEnum<'ctx> {
    fn from(value: BasicValueEnum<'ctx>) -> Self {
        unsafe { BasicMetadataValueEnum::new(value.as_value_ref()) }
    }
}

impl<'ctx> TryFrom<AnyValueEnum<'ctx>> for BasicValueEnum<'ctx> {
    type Error = ();

    fn try_from(value: AnyValueEnum<'ctx>) -> Result<Self, Self::Error> {
        use AnyValueEnum::*;
        Ok(match value {
            ArrayValue(av) => av.into(),
            IntValue(iv) => iv.into(),
            FloatValue(fv) => fv.into(),
            PointerValue(pv) => pv.into(),
            StructValue(sv) => sv.into(),
            VectorValue(vv) => vv.into(),
            ScalableVectorValue(vv) => vv.into(),
            MetadataValue(_) | PhiValue(_) | FunctionValue(_) | InstructionValue(_) => return Err(()),
        })
    }
}

impl<'ctx> TryFrom<AnyValueEnum<'ctx>> for BasicMetadataValueEnum<'ctx> {
    type Error = ();

    fn try_from(value: AnyValueEnum<'ctx>) -> Result<Self, Self::Error> {
        use AnyValueEnum::*;
        Ok(match value {
            ArrayValue(av) => av.into(),
            IntValue(iv) => iv.into(),
            FloatValue(fv) => fv.into(),
            PointerValue(pv) => pv.into(),
            StructValue(sv) => sv.into(),
            VectorValue(vv) => vv.into(),
            ScalableVectorValue(vv) => vv.into(),
            MetadataValue(mv) => mv.into(),
            PhiValue(_) | FunctionValue(_) | InstructionValue(_) => return Err(()),
        })
    }
}

impl<'ctx> TryFrom<BasicMetadataValueEnum<'ctx>> for BasicValueEnum<'ctx> {
    type Error = ();

    fn try_from(value: BasicMetadataValueEnum<'ctx>) -> Result<Self, Self::Error> {
        use BasicMetadataValueEnum::*;
        Ok(match value {
            ArrayValue(av) => av.into(),
            IntValue(iv) => iv.into(),
            FloatValue(fv) => fv.into(),
            PointerValue(pv) => pv.into(),
            StructValue(sv) => sv.into(),
            VectorValue(vv) => vv.into(),
            ScalableVectorValue(vv) => vv.into(),
            MetadataValue(_) => return Err(()),
        })
    }
}

impl Display for AggregateValueEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl Display for AnyValueEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl Display for BasicValueEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl Display for BasicMetadataValueEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
