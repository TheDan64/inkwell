use llvm_sys::core::LLVMGetTypeKind;
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::LLVMTypeKind;

use crate::support::LLVMString;
use crate::types::traits::AsTypeRef;
use crate::types::MetadataType;
use crate::types::{
    ArrayType, FloatType, FunctionType, IntType, PointerType, ScalableVectorType, StructType, VectorType, VoidType,
};
use crate::values::{BasicValue, BasicValueEnum, IntValue};

use std::convert::TryFrom;
use std::fmt::{self, Display};

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

        unsafe impl AsTypeRef for $enum_name<'_> {
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
        /// A contiguous homogeneous scalable "SIMD" container type.
        ScalableVectorType,
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
        /// A contiguous homogeneous scalable "SIMD" container type.
        ScalableVectorType,
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
        ScalableVectorType,
        MetadataType,
    }
}

impl<'ctx> BasicMetadataTypeEnum<'ctx> {
    /// Create [`BasicMetadataTypeEnum`] from [`LLVMTypeRef`].
    ///
    /// # Safety
    ///
    /// Undefined behavior if the referenced type cannot be represented as [`BasicMetadataTypeEnum`],
    /// or the underlying pointer is null.
    ///
    /// Before LLVM 6, [`BasicMetadataTypeEnum::MetadataType`] variants cannot be created
    /// with this function. Attempting to do results in undefined behavior.
    pub unsafe fn new(type_: LLVMTypeRef) -> Self {
        match LLVMGetTypeKind(type_) {
            LLVMTypeKind::LLVMMetadataTypeKind => Self::MetadataType(MetadataType::new(type_)),
            _ => BasicTypeEnum::new(type_).into(),
        }
    }

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

    pub fn into_scalable_vector_type(self) -> ScalableVectorType<'ctx> {
        if let BasicMetadataTypeEnum::ScalableVectorType(t) = self {
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

    pub fn is_scalable_vector_type(self) -> bool {
        matches!(self, BasicMetadataTypeEnum::ScalableVectorType(_))
    }

    /// Print the definition of a `BasicMetadataTypeEnum` to `LLVMString`.
    pub fn print_to_string(self) -> LLVMString {
        match self {
            BasicMetadataTypeEnum::ArrayType(t) => t.print_to_string(),
            BasicMetadataTypeEnum::IntType(t) => t.print_to_string(),
            BasicMetadataTypeEnum::FloatType(t) => t.print_to_string(),
            BasicMetadataTypeEnum::PointerType(t) => t.print_to_string(),
            BasicMetadataTypeEnum::StructType(t) => t.print_to_string(),
            BasicMetadataTypeEnum::VectorType(t) => t.print_to_string(),
            BasicMetadataTypeEnum::ScalableVectorType(t) => t.print_to_string(),
            BasicMetadataTypeEnum::MetadataType(t) => t.print_to_string(),
        }
    }
}

impl<'ctx> AnyTypeEnum<'ctx> {
    /// Create `AnyTypeEnum` from [`LLVMTypeRef`]
    ///
    /// # Safety
    /// Undefined behavior, if referenced type isn't part of `AnyTypeEnum`
    pub unsafe fn new(type_: LLVMTypeRef) -> Self {
        match LLVMGetTypeKind(type_) {
            LLVMTypeKind::LLVMVoidTypeKind => AnyTypeEnum::VoidType(VoidType::new(type_)),
            LLVMTypeKind::LLVMHalfTypeKind
            | LLVMTypeKind::LLVMFloatTypeKind
            | LLVMTypeKind::LLVMDoubleTypeKind
            | LLVMTypeKind::LLVMX86_FP80TypeKind
            | LLVMTypeKind::LLVMFP128TypeKind
            | LLVMTypeKind::LLVMPPC_FP128TypeKind => AnyTypeEnum::FloatType(FloatType::new(type_)),
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
            LLVMTypeKind::LLVMBFloatTypeKind => AnyTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMLabelTypeKind => panic!("FIXME: Unsupported type: Label"),
            LLVMTypeKind::LLVMIntegerTypeKind => AnyTypeEnum::IntType(IntType::new(type_)),
            LLVMTypeKind::LLVMFunctionTypeKind => AnyTypeEnum::FunctionType(FunctionType::new(type_)),
            LLVMTypeKind::LLVMStructTypeKind => AnyTypeEnum::StructType(StructType::new(type_)),
            LLVMTypeKind::LLVMArrayTypeKind => AnyTypeEnum::ArrayType(ArrayType::new(type_)),
            LLVMTypeKind::LLVMPointerTypeKind => AnyTypeEnum::PointerType(PointerType::new(type_)),
            LLVMTypeKind::LLVMVectorTypeKind => AnyTypeEnum::VectorType(VectorType::new(type_)),
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
            LLVMTypeKind::LLVMScalableVectorTypeKind => AnyTypeEnum::ScalableVectorType(ScalableVectorType::new(type_)),
            // FIXME: should inkwell support metadata as AnyType?
            LLVMTypeKind::LLVMMetadataTypeKind => panic!("Metadata type is not supported as AnyType."),
            LLVMTypeKind::LLVMX86_MMXTypeKind => panic!("FIXME: Unsupported type: MMX"),
            #[cfg(any(
                feature = "llvm12-0",
                feature = "llvm13-0",
                feature = "llvm14-0",
                feature = "llvm15-0",
                feature = "llvm16-0",
                feature = "llvm17-0",
                feature = "llvm18-1"
            ))]
            LLVMTypeKind::LLVMX86_AMXTypeKind => panic!("FIXME: Unsupported type: AMX"),
            LLVMTypeKind::LLVMTokenTypeKind => panic!("FIXME: Unsupported type: Token"),
            #[cfg(any(feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-1"))]
            LLVMTypeKind::LLVMTargetExtTypeKind => panic!("FIXME: Unsupported type: TargetExt"),
        }
    }

    /// This will panic if type is a void or function type.
    pub(crate) fn as_basic_type_enum(&self) -> BasicTypeEnum<'ctx> {
        unsafe { BasicTypeEnum::new(self.as_type_ref()) }
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

    pub fn into_scalable_vector_type(self) -> ScalableVectorType<'ctx> {
        if let AnyTypeEnum::ScalableVectorType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the ScalableVectorType variant", self);
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
            AnyTypeEnum::ScalableVectorType(t) => t.size_of(),
            AnyTypeEnum::VoidType(_) => None,
            AnyTypeEnum::FunctionType(_) => None,
        }
    }

    /// Print the definition of a `AnyTypeEnum` to `LLVMString`.
    pub fn print_to_string(self) -> LLVMString {
        match self {
            AnyTypeEnum::ArrayType(t) => t.print_to_string(),
            AnyTypeEnum::FloatType(t) => t.print_to_string(),
            AnyTypeEnum::IntType(t) => t.print_to_string(),
            AnyTypeEnum::PointerType(t) => t.print_to_string(),
            AnyTypeEnum::StructType(t) => t.print_to_string(),
            AnyTypeEnum::VectorType(t) => t.print_to_string(),
            AnyTypeEnum::ScalableVectorType(t) => t.print_to_string(),
            AnyTypeEnum::VoidType(t) => t.print_to_string(),
            AnyTypeEnum::FunctionType(t) => t.print_to_string(),
        }
    }
}

impl<'ctx> BasicTypeEnum<'ctx> {
    /// Create `BasicTypeEnum` from [`LLVMTypeRef`]
    ///
    /// # Safety
    /// Undefined behavior, if referenced type isn't part of basic type enum.
    pub unsafe fn new(type_: LLVMTypeRef) -> Self {
        match LLVMGetTypeKind(type_) {
            LLVMTypeKind::LLVMHalfTypeKind
            | LLVMTypeKind::LLVMFloatTypeKind
            | LLVMTypeKind::LLVMDoubleTypeKind
            | LLVMTypeKind::LLVMX86_FP80TypeKind
            | LLVMTypeKind::LLVMFP128TypeKind
            | LLVMTypeKind::LLVMPPC_FP128TypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
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
            LLVMTypeKind::LLVMBFloatTypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMIntegerTypeKind => BasicTypeEnum::IntType(IntType::new(type_)),
            LLVMTypeKind::LLVMStructTypeKind => BasicTypeEnum::StructType(StructType::new(type_)),
            LLVMTypeKind::LLVMPointerTypeKind => BasicTypeEnum::PointerType(PointerType::new(type_)),
            LLVMTypeKind::LLVMArrayTypeKind => BasicTypeEnum::ArrayType(ArrayType::new(type_)),
            LLVMTypeKind::LLVMVectorTypeKind => BasicTypeEnum::VectorType(VectorType::new(type_)),
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
                BasicTypeEnum::ScalableVectorType(ScalableVectorType::new(type_))
            },
            LLVMTypeKind::LLVMMetadataTypeKind => panic!("Unsupported basic type: Metadata"),
            // see https://llvm.org/docs/LangRef.html#x86-mmx-type
            LLVMTypeKind::LLVMX86_MMXTypeKind => panic!("Unsupported basic type: MMX"),
            // see https://llvm.org/docs/LangRef.html#x86-amx-type
            #[cfg(any(
                feature = "llvm12-0",
                feature = "llvm13-0",
                feature = "llvm14-0",
                feature = "llvm15-0",
                feature = "llvm16-0",
                feature = "llvm17-0",
                feature = "llvm18-1"
            ))]
            LLVMTypeKind::LLVMX86_AMXTypeKind => unreachable!("Unsupported basic type: AMX"),
            LLVMTypeKind::LLVMLabelTypeKind => unreachable!("Unsupported basic type: Label"),
            LLVMTypeKind::LLVMVoidTypeKind => unreachable!("Unsupported basic type: VoidType"),
            LLVMTypeKind::LLVMFunctionTypeKind => unreachable!("Unsupported basic type: FunctionType"),
            LLVMTypeKind::LLVMTokenTypeKind => unreachable!("Unsupported basic type: Token"),
            #[cfg(any(feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-1"))]
            LLVMTypeKind::LLVMTargetExtTypeKind => unreachable!("Unsupported basic type: TargetExt"),
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

    pub fn into_scalable_vector_type(self) -> ScalableVectorType<'ctx> {
        if let BasicTypeEnum::ScalableVectorType(t) = self {
            t
        } else {
            panic!("Found {:?} but expected the ScalableVectorType variant", self);
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

    pub fn is_scalable_vector_type(self) -> bool {
        matches!(self, BasicTypeEnum::ScalableVectorType(_))
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
            BasicTypeEnum::ScalableVectorType(ty) => ty.const_zero().as_basic_value_enum(),
        }
    }

    /// Print the definition of a `BasicTypeEnum` to `LLVMString`.
    pub fn print_to_string(self) -> LLVMString {
        match self {
            BasicTypeEnum::ArrayType(t) => t.print_to_string(),
            BasicTypeEnum::FloatType(t) => t.print_to_string(),
            BasicTypeEnum::IntType(t) => t.print_to_string(),
            BasicTypeEnum::PointerType(t) => t.print_to_string(),
            BasicTypeEnum::StructType(t) => t.print_to_string(),
            BasicTypeEnum::VectorType(t) => t.print_to_string(),
            BasicTypeEnum::ScalableVectorType(t) => t.print_to_string(),
        }
    }
}

impl<'ctx> TryFrom<AnyTypeEnum<'ctx>> for BasicTypeEnum<'ctx> {
    type Error = ();

    fn try_from(value: AnyTypeEnum<'ctx>) -> Result<Self, Self::Error> {
        use AnyTypeEnum::*;
        Ok(match value {
            ArrayType(at) => at.into(),
            FloatType(ft) => ft.into(),
            IntType(it) => it.into(),
            PointerType(pt) => pt.into(),
            StructType(st) => st.into(),
            VectorType(vt) => vt.into(),
            ScalableVectorType(vt) => vt.into(),
            VoidType(_) | FunctionType(_) => return Err(()),
        })
    }
}

impl<'ctx> TryFrom<AnyTypeEnum<'ctx>> for BasicMetadataTypeEnum<'ctx> {
    type Error = ();

    fn try_from(value: AnyTypeEnum<'ctx>) -> Result<Self, Self::Error> {
        use AnyTypeEnum::*;
        Ok(match value {
            ArrayType(at) => at.into(),
            FloatType(ft) => ft.into(),
            IntType(it) => it.into(),
            PointerType(pt) => pt.into(),
            StructType(st) => st.into(),
            VectorType(vt) => vt.into(),
            ScalableVectorType(vt) => vt.into(),
            VoidType(_) | FunctionType(_) => return Err(()),
        })
    }
}

impl<'ctx> TryFrom<BasicMetadataTypeEnum<'ctx>> for BasicTypeEnum<'ctx> {
    type Error = ();

    fn try_from(value: BasicMetadataTypeEnum<'ctx>) -> Result<Self, Self::Error> {
        use BasicMetadataTypeEnum::*;
        Ok(match value {
            ArrayType(at) => at.into(),
            FloatType(ft) => ft.into(),
            IntType(it) => it.into(),
            PointerType(pt) => pt.into(),
            StructType(st) => st.into(),
            VectorType(vt) => vt.into(),
            ScalableVectorType(vt) => vt.into(),
            MetadataType(_) => return Err(()),
        })
    }
}

impl<'ctx> From<BasicTypeEnum<'ctx>> for BasicMetadataTypeEnum<'ctx> {
    fn from(value: BasicTypeEnum<'ctx>) -> Self {
        use BasicTypeEnum::*;
        match value {
            ArrayType(at) => at.into(),
            FloatType(ft) => ft.into(),
            IntType(it) => it.into(),
            PointerType(pt) => pt.into(),
            StructType(st) => st.into(),
            VectorType(vt) => vt.into(),
            ScalableVectorType(vt) => vt.into(),
        }
    }
}

impl Display for AnyTypeEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl Display for BasicTypeEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl Display for BasicMetadataTypeEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
