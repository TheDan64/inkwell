mod array_type;
mod float_type;
mod fn_type;
mod int_type;
mod ptr_type;
mod private {
    // REVIEW: Move to private.rs?
    // This is an ugly privacy hack so that Type can stay private to this module
    // and so that super traits using this trait will be not be implementable
    // outside this library
    use llvm_sys::prelude::LLVMTypeRef;

    pub trait AsTypeRef {
        fn as_type_ref(&self) -> LLVMTypeRef;
    }
}
mod struct_type;
mod vec_type;
mod void_type;

pub use types::array_type::ArrayType;
pub use types::float_type::FloatType;
pub use types::fn_type::FunctionType;
pub use types::int_type::IntType;
pub use types::ptr_type::PointerType;
pub use types::struct_type::StructType;
pub use types::vec_type::VectorType;
pub use types::void_type::VoidType;
pub(crate) use self::private::AsTypeRef;

use llvm_sys::core::{LLVMAlignOf, LLVMGetTypeContext, LLVMFunctionType, LLVMArrayType, LLVMDumpType, LLVMGetTypeKind, LLVMGetUndef, LLVMPointerType, LLVMPrintTypeToString, LLVMTypeIsSized, LLVMSizeOf, LLVMVectorType, LLVMConstPointerNull};
use llvm_sys::LLVMTypeKind;
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use std::ffi::CStr;
use std::fmt;

use context::{Context, ContextRef};
use values::{IntValue, PointerValue};

// REVIEW: Maybe move this into its own module?
// Worth noting that types seem to be singletons. At the very least, primitives are.
// Though this is likely only true per thread since LLVM claims to not be very thread-safe.
#[derive(PartialEq, Eq)]
struct Type {
    type_: LLVMTypeRef,
}

impl Type {
    fn new(type_: LLVMTypeRef) -> Self {
        assert!(!type_.is_null());

        Type {
            type_: type_,
        }
    }

    fn print_to_stderr(&self) {
        unsafe {
            LLVMDumpType(self.type_);
        }
    }

    fn const_null_ptr(&self) -> PointerValue {
        let ptr_type = unsafe {
            LLVMConstPointerNull(self.type_)
        };

        PointerValue::new(ptr_type)
    }

    fn ptr_type(&self, address_space: u32) -> PointerType {
        let ptr_type = unsafe {
            LLVMPointerType(self.type_, address_space)
        };

        PointerType::new(ptr_type)
    }

    fn vec_type(&self, size: u32) -> VectorType {
        let vec_type = unsafe {
            LLVMVectorType(self.type_, size)
        };

        VectorType::new(vec_type)
    }

    // REVIEW: Is this actually AnyType except FunctionType? VoidType? Can you make a FunctionType from a FunctionType???
    // Probably should just be BasicType
    fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
        let mut param_types: Vec<LLVMTypeRef> = param_types.iter()
                                                           .map(|val| val.as_type_ref())
                                                           .collect();
        let fn_type = unsafe {
            LLVMFunctionType(self.type_, param_types.as_mut_ptr(), param_types.len() as u32, is_var_args as i32)
        };

        FunctionType::new(fn_type)
    }

    fn array_type(&self, size: u32) -> ArrayType {
        let type_ = unsafe {
            LLVMArrayType(self.type_, size)
        };

        ArrayType::new(type_)
    }

    fn get_undef(&self) -> LLVMValueRef {
        unsafe {
            LLVMGetUndef(self.type_)
        }
    }

    // NOTE: AnyType
    pub(crate) fn get_kind(&self) -> LLVMTypeKind {
        unsafe {
            LLVMGetTypeKind(self.type_)
        }
    }

    // REVIEW: Return IntValue?
    fn get_alignment(&self) -> IntValue {
        let val = unsafe {
            LLVMAlignOf(self.type_)
        };

        IntValue::new(val)
    }

    fn get_context(&self) -> ContextRef {
        // We don't return an option because LLVM seems
        // to always assign a context, even to types
        // created without an explicit context, somehow

        let context = unsafe {
            LLVMGetTypeContext(self.type_)
        };

        ContextRef::new(Context::new(context))
    }

    fn is_sized(&self) -> bool {
        unsafe {
            LLVMTypeIsSized(self.type_) == 1
        }
    }

    // REVIEW: Option<IntValue>? What happens when type is unsized? We could return 0?
    // Also, is this even useful? Sized or not should be known at compile time?
    // For example, void is not sized. This may only be useful on Type Traits/Enums
    // where the actual type is unknown (trait) or yet undetermined (enum)
    fn size(&self) -> IntValue {
        let int_value = unsafe {
            LLVMSizeOf(self.type_)
        };

        IntValue::new(int_value)
    }

    fn print_to_string(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(self.type_))
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = self.print_to_string();

        write!(f, "Type {{\n    address: {:?}\n    llvm_type: {:?}\n}}", self.type_, llvm_type)
    }
}

macro_rules! trait_type_set {
    ($trait_name:ident: $($args:ident),*) => (
        pub trait $trait_name: AsTypeRef {}

        $(
            impl $trait_name for $args {}
        )*
    );
}

macro_rules! enum_type_set {
    ($enum_name:ident: $($args:ident),*) => (
        #[derive(Debug, EnumAsGetters, EnumIntoGetters, EnumIsA)]
        pub enum $enum_name {
            $(
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

enum_type_set! {AnyTypeEnum: IntType, FunctionType, FloatType, PointerType, StructType, ArrayType, VoidType, VectorType}
enum_type_set! {BasicTypeEnum: IntType, FloatType, PointerType, StructType, ArrayType, VectorType}

// TODO: Possibly rename to AnyTypeTrait, BasicTypeTrait
trait_type_set! {AnyType: AnyTypeEnum, BasicTypeEnum, IntType, FunctionType, FloatType, PointerType, StructType, ArrayType, VoidType, VectorType}
trait_type_set! {BasicType: BasicTypeEnum, IntType, FloatType, PointerType, StructType, ArrayType, VectorType}

impl AnyTypeEnum {
    pub(crate) fn new(type_: LLVMTypeRef) -> AnyTypeEnum {
        let type_kind = unsafe {
            LLVMGetTypeKind(type_)
        };

        match type_kind {
            LLVMTypeKind::LLVMVoidTypeKind => AnyTypeEnum::VoidType(VoidType::new(type_)),
            LLVMTypeKind::LLVMHalfTypeKind => AnyTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMFloatTypeKind => AnyTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMDoubleTypeKind => AnyTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMX86_FP80TypeKind => AnyTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMFP128TypeKind => AnyTypeEnum::FloatType(FloatType::new(type_)),
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
            // LLVMTypeKind::LLVMTokenTypeKind => panic!("FIXME: Unsupported type: Token"), // Different version?
        }
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
            _ => unreachable!("Unsupported type"),
        }
    }
}
