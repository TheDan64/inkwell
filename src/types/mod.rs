//! A type is a classification which determines how data is used.

#[deny(missing_docs)]
mod array_type;
mod enums;
#[deny(missing_docs)]
mod float_type;
#[deny(missing_docs)]
mod fn_type;
#[deny(missing_docs)]
mod int_type;
#[deny(missing_docs)]
mod ptr_type;
#[deny(missing_docs)]
mod struct_type;
#[deny(missing_docs)]
mod traits;
#[deny(missing_docs)]
mod vec_type;
#[deny(missing_docs)]
mod void_type;
#[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
#[deny(missing_docs)]
mod token_type;

pub use crate::types::array_type::ArrayType;
pub use crate::types::enums::{AnyTypeEnum, BasicTypeEnum};
pub use crate::types::float_type::FloatType;
pub use crate::types::fn_type::FunctionType;
pub use crate::types::int_type::{IntType, StringRadix};
pub use crate::types::ptr_type::PointerType;
pub use crate::types::struct_type::StructType;
pub use crate::types::traits::{AnyType, BasicType, IntMathType, FloatMathType, PointerMathType};
pub use crate::types::vec_type::VectorType;
pub use crate::types::void_type::VoidType;
#[cfg(not(any(feature = "llvm3-6", feature = "llvm3-7")))]
pub use crate::types::token_type::TokenType;
pub(crate) use crate::types::traits::AsTypeRef;

#[llvm_versions(3.7..=4.0)]
use llvm_sys::core::LLVMDumpType;
use llvm_sys::core::{LLVMAlignOf, LLVMGetTypeContext, LLVMFunctionType, LLVMArrayType, LLVMGetUndef, LLVMPointerType, LLVMPrintTypeToString, LLVMTypeIsSized, LLVMSizeOf, LLVMVectorType, LLVMGetElementType, LLVMConstNull};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};
#[cfg(feature = "experimental")]
use static_alloc::Slab;

use std::fmt;
use std::marker::PhantomData;

use crate::AddressSpace;
use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::values::IntValue;

// Worth noting that types seem to be singletons. At the very least, primitives are.
// Though this is likely only true per thread since LLVM claims to not be very thread-safe.
#[derive(PartialEq, Eq, Clone, Copy)]
struct Type<'ctx> {
    ty: LLVMTypeRef,
    _marker: PhantomData<&'ctx ()>,
}

impl<'ctx> Type<'ctx> {
    fn new(ty: LLVMTypeRef) -> Self {
        assert!(!ty.is_null());

        Type {
            ty,
            _marker: PhantomData,
        }
    }

    // REVIEW: LLVM 5.0+ seems to have removed this function in release mode
    // and so will fail to link when used. I've decided to remove it from 5.0+
    // for now. We should consider removing it altogether since print_to_string
    // could be used and manually written to stderr in rust...
    #[llvm_versions(3.7..=4.0)]
    fn print_to_stderr(&self) {
        unsafe {
            LLVMDumpType(self.ty);
        }
    }

    fn const_zero(&self) -> LLVMValueRef {
        unsafe {
            LLVMConstNull(self.ty)
        }
    }

    fn ptr_type(&self, address_space: AddressSpace) -> PointerType<'ctx> {
        let ptr_type = unsafe {
            LLVMPointerType(self.ty, address_space as u32)
        };

        PointerType::new(ptr_type)
    }

    fn vec_type(&self, size: u32) -> VectorType<'ctx> {
        assert!(size != 0, "Vectors of size zero are not allowed.");
        // -- https://llvm.org/docs/LangRef.html#vector-type

        let vec_type = unsafe {
            LLVMVectorType(self.ty, size)
        };

        VectorType::new(vec_type)
    }

    #[cfg(not(feature = "experimental"))]
    fn fn_type(&self, param_types: &[BasicTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        let mut param_types: Vec<LLVMTypeRef> = param_types.iter()
                                                           .map(|val| val.as_type_ref())
                                                           .collect();
        let fn_type = unsafe {
            LLVMFunctionType(self.ty, param_types.as_mut_ptr(), param_types.len() as u32, is_var_args as i32)
        };

        FunctionType::new(fn_type)
    }

    #[cfg(feature = "experimental")]
    fn fn_type(&self, param_types: &[BasicTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        let pool: Slab<[usize; 16]> = Slab::uninit();
        let mut fixed_vec = pool.fixed_vec(param_types.len()).expect("Found more than 16 params");

        for param_type in param_types {
            fixed_vec.push(param_type.as_type_ref()).expect("Unexpected error");
        }

        let fn_type = unsafe {
            LLVMFunctionType(self.ty, fixed_vec.as_mut_ptr(), fixed_vec.len() as u32, is_var_args as i32)
        };

        FunctionType::new(fn_type)
    }

    fn array_type(&self, size: u32) -> ArrayType<'ctx> {
        let ty = unsafe {
            LLVMArrayType(self.ty, size)
        };

        ArrayType::new(ty)
    }

    fn get_undef(&self) -> LLVMValueRef {
        unsafe {
            LLVMGetUndef(self.ty)
        }
    }

    fn get_alignment(&self) -> IntValue<'ctx> {
        let val = unsafe {
            LLVMAlignOf(self.ty)
        };

        IntValue::new(val)
    }

    fn get_context(&self) -> ContextRef<'ctx> {
        let context = unsafe {
            LLVMGetTypeContext(self.ty)
        };

        ContextRef::new(context)
    }

    // REVIEW: This should be known at compile time, maybe as a const fn?
    // On an enum or trait, this would not be known at compile time (unless
    // enum has only sized types for example)
    fn is_sized(&self) -> bool {
        unsafe {
            LLVMTypeIsSized(self.ty) == 1
        }
    }

    // REVIEW: Option<IntValue>? If we want to provide it on enums that
    // contain unsized types
    fn size_of(&self) -> IntValue<'ctx> {
        debug_assert!(self.is_sized());

        let int_value = unsafe {
            LLVMSizeOf(self.ty)
        };

        IntValue::new(int_value)
    }

    fn print_to_string(&self) -> LLVMString {
        let c_string_ptr = unsafe {
            LLVMPrintTypeToString(self.ty)
        };

        LLVMString::new(c_string_ptr)
    }

    pub fn get_element_type(&self) -> AnyTypeEnum<'ctx> {
        let ptr = unsafe {
            LLVMGetElementType(self.ty)
        };

        AnyTypeEnum::new(ptr)
    }

}

impl fmt::Debug for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = self.print_to_string();

        f.debug_struct("Type")
            .field("address", &self.ty)
            .field("llvm_type", &llvm_type)
            .finish()
    }
}
