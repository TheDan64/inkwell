use llvm_sys::prelude::LLVMTypeRef;

use std::fmt::Debug;

use types::{IntType, FunctionType, FloatType, PointerType, StructType, ArrayType, VectorType, VoidType};
use types::enums::{AnyTypeEnum, BasicTypeEnum};

// This is an ugly privacy hack so that Type can stay private to this module
// and so that super traits using this trait will be not be implementable
// outside this library
pub trait AsTypeRef {
    fn as_type_ref(&self) -> LLVMTypeRef;
}

macro_rules! trait_type_set {
    ($trait_name:ident: $($args:ident),*) => (
        pub trait $trait_name: AsTypeRef + Debug {}

        $(
            impl $trait_name for $args {}
        )*
    );
}

trait_type_set! {AnyType: AnyTypeEnum, BasicTypeEnum, IntType, FunctionType, FloatType, PointerType, StructType, ArrayType, VoidType, VectorType}
trait_type_set! {BasicType: BasicTypeEnum, IntType, FloatType, PointerType, StructType, ArrayType, VectorType}
