use llvm_sys::core::LLVMGetTypeKind;
use llvm_sys::LLVMTypeKind;
use llvm_sys::prelude::LLVMTypeRef;

use crate::types::{IntType, VoidType, FunctionType, PointerType, VectorType, ArrayType, StructType, FloatType};
use crate::types::MetadataType;
use crate::types::traits::AsTypeRef;
use crate::values::{BasicValue, BasicValueEnum, IntValue};

use std::convert::TryFrom;
use std::iter::FromIterator;

macro_rules! enum_type_set {
    ($(#[$enum_attrs:meta])* $enum_name:ident: { $($(#[$variant_attrs:meta])* $args:ident,)+ }) => (
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        $(#[$enum_attrs])*
        pub enum $enum_name<'ctx> {
            $(
                $(#[$variant_attrs])*
                $args($args<'ctx>),
            )*
        }

        impl AsTypeRef for $enum_name<'_> {
            fn as_type_ref(&self) -> LLVMTypeRef {
                match *self {
                    $(
                        $enum_name::$args(ref t) => t.as_type_ref(),
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

enum_type_set! {
    /// A wrapper for any `BasicType`, `VoidType`, or `FunctionType`.
    AnyTypeEnum: {
        /// A contiguous homogeneous container type.
        ArrayType,
        /// A floating point type.
        FloatType,
        /// A function return and parameter definition.
        FunctionType,
        /// An integer type.
        IntType,
        /// A pointer type.
        PointerType,
        /// A contiguous heterogeneous container type.
        StructType,
        /// A contiguous homogeneous "SIMD" container type.
        VectorType,
        /// A valueless type.
        VoidType,
    }
}
enum_type_set! {
    /// A wrapper for any `BasicType`.
    BasicTypeEnum: {
        /// A contiguous homogeneous container type.
        ArrayType,
        /// A floating point type.
        FloatType,
        /// An integer type.
        IntType,
        /// A pointer type.
        PointerType,
        /// A contiguous heterogeneous container type.
        StructType,
        /// A contiguous homogeneous "SIMD" container type.
        VectorType,
    }
}
enum_type_set! {
    BasicMetadataTypeEnum: {
        ArrayType,
        FloatType,
        IntType,
        PointerType,
        StructType,
        VectorType,
        MetadataType,
    }
}

impl<'ctx> BasicMetadataTypeEnum<'ctx> {
    pub fn into_array_type(self) -> ArrayType<'ctx> {
        if let BasicMetadataTypeEnum::ArrayType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected another variant", self);
        }
    }

    pub fn into_float_type(self) -> FloatType<'ctx> {
        if let BasicMetadataTypeEnum::FloatType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected another variant", self);
        }
    }

    pub fn into_int_type(self) -> IntType<'ctx> {
        if let BasicMetadataTypeEnum::IntType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected another variant", self);
        }
    }

    pub fn into_pointer_type(self) -> PointerType<'ctx> {
        if let BasicMetadataTypeEnum::PointerType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected another variant", self);
        }
    }

    pub fn into_struct_type(self) -> StructType<'ctx> {
        if let BasicMetadataTypeEnum::StructType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected another variant", self);
        }
    }

    pub fn into_vector_type(self) -> VectorType<'ctx> {
        if let BasicMetadataTypeEnum::VectorType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected another variant", self);
        }
    }

    pub fn into_metadata_type(self) -> MetadataType<'ctx> {
        if let BasicMetadataTypeEnum::MetadataType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected another variant", self);
        }
    }

    pub fn is_array_type(self) -> bool {
        matches!(self, BasicMetadataTypeEnum::ArrayType(_))
    }

    pub fn is_float_type(self) -> bool {
        matches!(self, BasicMetadataTypeEnum::FloatType(_))
    }

    pub fn is_int_type(self) -> bool {
        matches!(self, BasicMetadataTypeEnum::IntType(_))
    }

    pub fn is_metadata_type(self) -> bool {
        matches!(self, BasicMetadataTypeEnum::MetadataType(_))
    }

    pub fn is_pointer_type(self) -> bool {
        matches!(self, BasicMetadataTypeEnum::PointerType(_))
    }

    pub fn is_struct_type(self) -> bool {
        matches!(self, BasicMetadataTypeEnum::StructType(_))
    }

    pub fn is_vector_type(self) -> bool {
        matches!(self, BasicMetadataTypeEnum::VectorType(_))
    }
}

impl<'ctx> AnyTypeEnum<'ctx> {
    pub(crate) unsafe fn new(type_: LLVMTypeRef) -> Self {
        match LLVMGetTypeKind(type_) {
            LLVMTypeKind::LLVMVoidTypeKind => AnyTypeEnum::VoidType(VoidType::new(type_)),
            LLVMTypeKind::LLVMHalfTypeKind |
            LLVMTypeKind::LLVMFloatTypeKind |
            LLVMTypeKind::LLVMDoubleTypeKind |
            LLVMTypeKind::LLVMX86_FP80TypeKind |
            LLVMTypeKind::LLVMFP128TypeKind |
            LLVMTypeKind::LLVMPPC_FP128TypeKind => AnyTypeEnum::FloatType(FloatType::new(type_)),
            #[cfg(any(feature = "llvm11-0", feature = "llvm12-0", feature = "llvm13-0"))]
            LLVMTypeKind::LLVMBFloatTypeKind => AnyTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMLabelTypeKind => panic!("FIXME: Unsupported type: Label"),
            LLVMTypeKind::LLVMIntegerTypeKind => AnyTypeEnum::IntType(IntType::new(type_)),
            LLVMTypeKind::LLVMFunctionTypeKind => AnyTypeEnum::FunctionType(FunctionType::new(type_)),
            LLVMTypeKind::LLVMStructTypeKind => AnyTypeEnum::StructType(StructType::new(type_)),
            LLVMTypeKind::LLVMArrayTypeKind => AnyTypeEnum::ArrayType(ArrayType::new(type_)),
            LLVMTypeKind::LLVMPointerTypeKind => AnyTypeEnum::PointerType(PointerType::new(type_)),
            LLVMTypeKind::LLVMVectorTypeKind => AnyTypeEnum::VectorType(VectorType::new(type_)),
            #[cfg(any(feature = "llvm11-0", feature = "llvm12-0", feature = "llvm13-0"))]
            LLVMTypeKind::LLVMScalableVectorTypeKind => AnyTypeEnum::VectorType(VectorType::new(type_)),
            LLVMTypeKind::LLVMMetadataTypeKind => unreachable!("Metadata type is not supported as AnyType."),
            LLVMTypeKind::LLVMX86_MMXTypeKind => panic!("FIXME: Unsupported type: MMX"),
            #[cfg(any(feature = "llvm12-0", feature = "llvm13-0"))]
            LLVMTypeKind::LLVMX86_AMXTypeKind => panic!("FIXME: Unsupported type: AMX"),
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            LLVMTypeKind::LLVMTokenTypeKind => panic!("FIXME: Unsupported type: Token"),
        }
    }

    /// This will panic if type is a void or function type.
    pub(crate) fn to_basic_type_enum(&self) -> BasicTypeEnum<'ctx> {
        unsafe {
            BasicTypeEnum::new(self.as_type_ref())
        }
    }

    pub fn into_array_type(self) -> ArrayType<'ctx> {
        if let AnyTypeEnum::ArrayType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the ArrayType variant", self);
        }
    }

    pub fn into_float_type(self) -> FloatType<'ctx> {
        if let AnyTypeEnum::FloatType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the FloatType variant", self);
        }
    }

    pub fn into_function_type(self) -> FunctionType<'ctx> {
        if let AnyTypeEnum::FunctionType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the FunctionType variant", self);
        }
    }

    pub fn into_int_type(self) -> IntType<'ctx> {
        if let AnyTypeEnum::IntType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the IntType variant", self);
        }
    }

    pub fn into_pointer_type(self) -> PointerType<'ctx> {
        if let AnyTypeEnum::PointerType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the PointerType variant", self);
        }
    }

    pub fn into_struct_type(self) -> StructType<'ctx> {
        if let AnyTypeEnum::StructType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the StructType variant", self);
        }
    }

    pub fn into_vector_type(self) -> VectorType<'ctx> {
        if let AnyTypeEnum::VectorType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the VectorType variant", self);
        }
    }

    pub fn into_void_type(self) -> VoidType<'ctx> {
        if let AnyTypeEnum::VoidType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the VoidType variant", self);
        }
    }

    pub fn is_array_type(self) -> bool {
        matches!(self, AnyTypeEnum::ArrayType(_))
    }

    pub fn is_float_type(self) -> bool {
        matches!(self, AnyTypeEnum::FloatType(_))
    }

    pub fn is_function_type(self) -> bool {
        matches!(self, AnyTypeEnum::FunctionType(_))
    }

    pub fn is_int_type(self) -> bool {
        matches!(self, AnyTypeEnum::IntType(_))
    }

    pub fn is_pointer_type(self) -> bool {
        matches!(self, AnyTypeEnum::PointerType(_))
    }

    pub fn is_struct_type(self) -> bool {
        matches!(self, AnyTypeEnum::StructType(_))
    }

    pub fn is_vector_type(self) -> bool {
        matches!(self, AnyTypeEnum::VectorType(_))
    }

    pub fn is_void_type(self) -> bool {
        matches!(self, AnyTypeEnum::VoidType(_))
    }

    pub fn size_of(&self) -> Option<IntValue<'ctx>> {
        match self {
            AnyTypeEnum::ArrayType(t) => t.size_of(),
            AnyTypeEnum::FloatType(t) => Some(t.size_of()),
            AnyTypeEnum::IntType(t) => Some(t.size_of()),
            AnyTypeEnum::PointerType(t) => Some(t.size_of()),
            AnyTypeEnum::StructType(t) => t.size_of(),
            AnyTypeEnum::VectorType(t) => t.size_of(),
            AnyTypeEnum::VoidType(_) => None,
            AnyTypeEnum::FunctionType(_) => None,
        }
    }
}

impl<'ctx> BasicTypeEnum<'ctx> {
    pub(crate) unsafe fn new(type_: LLVMTypeRef) -> Self {
        match LLVMGetTypeKind(type_) {
            LLVMTypeKind::LLVMHalfTypeKind |
            LLVMTypeKind::LLVMFloatTypeKind |
            LLVMTypeKind::LLVMDoubleTypeKind |
            LLVMTypeKind::LLVMX86_FP80TypeKind |
            LLVMTypeKind::LLVMFP128TypeKind |
            LLVMTypeKind::LLVMPPC_FP128TypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
            #[cfg(any(feature = "llvm11-0", feature = "llvm12-0", feature = "llvm13-0"))]
            LLVMTypeKind::LLVMBFloatTypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMIntegerTypeKind => BasicTypeEnum::IntType(IntType::new(type_)),
            LLVMTypeKind::LLVMStructTypeKind => BasicTypeEnum::StructType(StructType::new(type_)),
            LLVMTypeKind::LLVMPointerTypeKind => BasicTypeEnum::PointerType(PointerType::new(type_)),
            LLVMTypeKind::LLVMArrayTypeKind => BasicTypeEnum::ArrayType(ArrayType::new(type_)),
            LLVMTypeKind::LLVMVectorTypeKind => BasicTypeEnum::VectorType(VectorType::new(type_)),
            #[cfg(any(feature = "llvm11-0", feature = "llvm12-0", feature = "llvm13-0"))]
            LLVMTypeKind::LLVMScalableVectorTypeKind => BasicTypeEnum::VectorType(VectorType::new(type_)),
            LLVMTypeKind::LLVMMetadataTypeKind => unreachable!("Unsupported basic type: Metadata"),
            // see https://llvm.org/docs/LangRef.html#x86-mmx-type
            LLVMTypeKind::LLVMX86_MMXTypeKind => unreachable!("Unsupported basic type: MMX"),
            // see https://llvm.org/docs/LangRef.html#x86-amx-type
            #[cfg(any(feature = "llvm12-0", feature = "llvm13-0"))]
            LLVMTypeKind::LLVMX86_AMXTypeKind => unreachable!("Unsupported basic type: AMX"),
            LLVMTypeKind::LLVMLabelTypeKind => unreachable!("Unsupported basic type: Label"),
            LLVMTypeKind::LLVMVoidTypeKind => unreachable!("Unsupported basic type: VoidType"),
            LLVMTypeKind::LLVMFunctionTypeKind => unreachable!("Unsupported basic type: FunctionType"),
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            LLVMTypeKind::LLVMTokenTypeKind => unreachable!("Unsupported basic type: Token"),
        }
    }

    pub fn into_array_type(self) -> ArrayType<'ctx> {
        if let BasicTypeEnum::ArrayType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the ArrayType variant", self);
        }
    }

    pub fn into_float_type(self) -> FloatType<'ctx> {
        if let BasicTypeEnum::FloatType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the FloatType variant", self);
        }
    }

    pub fn into_int_type(self) -> IntType<'ctx> {
        if let BasicTypeEnum::IntType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the IntType variant", self);
        }
    }

    pub fn into_pointer_type(self) -> PointerType<'ctx> {
        if let BasicTypeEnum::PointerType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the PointerType variant", self);
        }
    }

    pub fn into_struct_type(self) -> StructType<'ctx> {
        if let BasicTypeEnum::StructType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the StructType variant", self);
        }
    }

    pub fn into_vector_type(self) -> VectorType<'ctx> {
        if let BasicTypeEnum::VectorType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the VectorType variant", self);
        }
    }

    pub fn is_array_type(self) -> bool {
        matches!(self, BasicTypeEnum::ArrayType(_))
    }

    pub fn is_float_type(self) -> bool {
        matches!(self, BasicTypeEnum::FloatType(_))
    }

    pub fn is_int_type(self) -> bool {
        matches!(self, BasicTypeEnum::IntType(_))
    }

    pub fn is_pointer_type(self) -> bool {
        matches!(self, BasicTypeEnum::PointerType(_))
    }

    pub fn is_struct_type(self) -> bool {
        matches!(self, BasicTypeEnum::StructType(_))
    }

    pub fn is_vector_type(self) -> bool {
        matches!(self, BasicTypeEnum::VectorType(_))
    }

    /// Creates a constant `BasicValueZero`.
    ///
    /// # Example
    /// ```
    /// use inkwell::context::Context;
    /// use crate::inkwell::types::BasicType;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type().as_basic_type_enum();
    /// let f32_zero = f32_type.const_zero();
    /// ```
    pub fn const_zero(self) -> BasicValueEnum<'ctx> {
        match self {
            BasicTypeEnum::ArrayType(ty) => ty.const_zero().as_basic_value_enum(),
            BasicTypeEnum::FloatType(ty) => ty.const_zero().as_basic_value_enum(),
            BasicTypeEnum::IntType(ty) => ty.const_zero().as_basic_value_enum(),
            BasicTypeEnum::PointerType(ty) => ty.const_zero().as_basic_value_enum(),
            BasicTypeEnum::StructType(ty) => ty.const_zero().as_basic_value_enum(),
            BasicTypeEnum::VectorType(ty) => ty.const_zero().as_basic_value_enum(),
        }
    }
}

impl<'ctx> TryFrom<AnyTypeEnum<'ctx>> for BasicTypeEnum<'ctx> {
    type Error = ();

    fn try_from(value: AnyTypeEnum<'ctx>) -> Result<Self, Self::Error> {
        Ok(match value {
            AnyTypeEnum::ArrayType(at) => at.into(),
            AnyTypeEnum::FloatType(ft) => ft.into(),
            AnyTypeEnum::IntType(it) => it.into(),
            AnyTypeEnum::PointerType(pt) => pt.into(),
            AnyTypeEnum::StructType(st) => st.into(),
            AnyTypeEnum::VectorType(vt) => vt.into(),
            _ => return Err(()),
        })
    }
}

impl<'ctx> From<BasicTypeEnum<'ctx>> for BasicMetadataTypeEnum<'ctx> {
    fn from(value: BasicTypeEnum<'ctx>) -> Self {
        match value {
            BasicTypeEnum::ArrayType(at) => at.into(),
            BasicTypeEnum::FloatType(ft) => ft.into(),
            BasicTypeEnum::IntType(it) => it.into(),
            BasicTypeEnum::PointerType(pt) => pt.into(),
            BasicTypeEnum::StructType(st) => st.into(),
            BasicTypeEnum::VectorType(vt) => vt.into(),
        }
    }
}
