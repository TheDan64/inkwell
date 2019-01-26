use llvm_sys::core::LLVMGetTypeKind;
use llvm_sys::LLVMTypeKind;
use llvm_sys::prelude::LLVMTypeRef;

use types::{IntType, VoidType, FunctionType, PointerType, VectorType, ArrayType, StructType, FloatType};
use types::traits::AsTypeRef;

macro_rules! enum_type_set {
    ($(#[$enum_attrs:meta])* $enum_name:ident: { $($(#[$variant_attrs:meta])* $args:ident,)+ }) => (
        #[derive(Debug, EnumAsGetters, EnumIntoGetters, EnumIsA, PartialEq, Eq, Clone, Copy)]
        $(#[$enum_attrs])*
        pub enum $enum_name {
            $(
                $(#[$variant_attrs])*
                $args($args),
            )*
        }

        impl AsTypeRef for $enum_name {
            fn as_type_ref(&self) -> LLVMTypeRef {
                match *self {
                    $(
                        $enum_name::$args(ref t) => t.as_type_ref(),
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
        // An integer type.
        IntType,
        /// A pointer type.
        PointerType,
        /// A contiguous heterogeneous container type.
        StructType,
        /// A contiguous homogeneous "SIMD" container type.
        VectorType,
    }
}

impl AnyTypeEnum {
    pub(crate) fn new(type_: LLVMTypeRef) -> AnyTypeEnum {
        let type_kind = unsafe {
            LLVMGetTypeKind(type_)
        };

        match type_kind {
            LLVMTypeKind::LLVMVoidTypeKind => AnyTypeEnum::VoidType(VoidType::new(type_)),
            LLVMTypeKind::LLVMHalfTypeKind |
            LLVMTypeKind::LLVMFloatTypeKind |
            LLVMTypeKind::LLVMDoubleTypeKind |
            LLVMTypeKind::LLVMX86_FP80TypeKind |
            LLVMTypeKind::LLVMFP128TypeKind |
            LLVMTypeKind::LLVMPPC_FP128TypeKind => AnyTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMLabelTypeKind => panic!("FIXME: Unsupported type: Label"),
            LLVMTypeKind::LLVMIntegerTypeKind => AnyTypeEnum::IntType(IntType::new(type_)),
            LLVMTypeKind::LLVMFunctionTypeKind => AnyTypeEnum::FunctionType(FunctionType::new(type_)),
            LLVMTypeKind::LLVMStructTypeKind => AnyTypeEnum::StructType(StructType::new(type_)),
            LLVMTypeKind::LLVMArrayTypeKind => AnyTypeEnum::ArrayType(ArrayType::new(type_)),
            LLVMTypeKind::LLVMPointerTypeKind => AnyTypeEnum::PointerType(PointerType::new(type_)),
            LLVMTypeKind::LLVMVectorTypeKind => AnyTypeEnum::VectorType(VectorType::new(type_)),
            LLVMTypeKind::LLVMMetadataTypeKind => panic!("FIXME: Unsupported type: Metadata"),
            LLVMTypeKind::LLVMX86_MMXTypeKind => panic!("FIXME: Unsupported type: MMX"),
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            LLVMTypeKind::LLVMTokenTypeKind => panic!("FIXME: Unsupported type: Token"),
        }
    }

    /// This will panic if type is a void or function type.
    pub(crate) fn to_basic_type_enum(&self) -> BasicTypeEnum {
        BasicTypeEnum::new(self.as_type_ref())
    }
}

impl BasicTypeEnum {
    pub(crate) fn new(type_: LLVMTypeRef) -> BasicTypeEnum {
        let type_kind = unsafe {
            LLVMGetTypeKind(type_)
        };

        match type_kind {
            LLVMTypeKind::LLVMHalfTypeKind |
            LLVMTypeKind::LLVMFloatTypeKind |
            LLVMTypeKind::LLVMDoubleTypeKind |
            LLVMTypeKind::LLVMX86_FP80TypeKind |
            LLVMTypeKind::LLVMFP128TypeKind |
            LLVMTypeKind::LLVMPPC_FP128TypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMIntegerTypeKind => BasicTypeEnum::IntType(IntType::new(type_)),
            LLVMTypeKind::LLVMStructTypeKind => BasicTypeEnum::StructType(StructType::new(type_)),
            LLVMTypeKind::LLVMPointerTypeKind => BasicTypeEnum::PointerType(PointerType::new(type_)),
            LLVMTypeKind::LLVMArrayTypeKind => BasicTypeEnum::ArrayType(ArrayType::new(type_)),
            LLVMTypeKind::LLVMVectorTypeKind => BasicTypeEnum::VectorType(VectorType::new(type_)),
            LLVMTypeKind::LLVMMetadataTypeKind => unreachable!("Unsupported basic type: Metadata"),
            LLVMTypeKind::LLVMX86_MMXTypeKind => unreachable!("Unsupported basic type: MMX"),
            LLVMTypeKind::LLVMLabelTypeKind => unreachable!("Unsupported basic type: Label"),
            LLVMTypeKind::LLVMVoidTypeKind => unreachable!("Unsupported basic type: VoidType"),
            LLVMTypeKind::LLVMFunctionTypeKind => unreachable!("Unsupported basic type: FunctionType"),
            #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
            LLVMTypeKind::LLVMTokenTypeKind => unreachable!("Unsupported basic type: Token"),
        }
    }
}
