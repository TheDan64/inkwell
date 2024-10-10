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
mod metadata_type;
#[deny(missing_docs)]
mod ptr_type;
#[deny(missing_docs)]
mod scalable_vec_type;
#[deny(missing_docs)]
mod struct_type;
#[deny(missing_docs)]
mod traits;
#[deny(missing_docs)]
mod vec_type;
#[deny(missing_docs)]
mod void_type;

pub use crate::types::array_type::ArrayType;
pub use crate::types::enums::{AnyTypeEnum, BasicMetadataTypeEnum, BasicTypeEnum};
pub use crate::types::float_type::FloatType;
pub use crate::types::fn_type::FunctionType;
pub use crate::types::int_type::{IntType, StringRadix};
pub use crate::types::metadata_type::MetadataType;
pub use crate::types::ptr_type::PointerType;
pub use crate::types::scalable_vec_type::ScalableVectorType;
pub use crate::types::struct_type::FieldTypesIter;
pub use crate::types::struct_type::StructType;
pub use crate::types::traits::{AnyType, AsTypeRef, BasicType, FloatMathType, IntMathType, PointerMathType};
pub use crate::types::vec_type::VectorType;
pub use crate::types::void_type::VoidType;

#[llvm_versions(12..)]
use llvm_sys::core::LLVMScalableVectorType;

#[llvm_versions(12..)]
use llvm_sys::core::LLVMGetPoison;

#[allow(deprecated)]
use llvm_sys::core::LLVMArrayType;
use llvm_sys::core::{
    LLVMAlignOf, LLVMConstNull, LLVMConstPointerNull, LLVMFunctionType, LLVMGetElementType, LLVMGetTypeContext,
    LLVMGetTypeKind, LLVMGetUndef, LLVMPointerType, LLVMPrintTypeToString, LLVMSizeOf, LLVMTypeIsSized, LLVMVectorType,
};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};
use llvm_sys::LLVMTypeKind;
#[cfg(feature = "experimental")]
use static_alloc::Bump;

use std::fmt;
use std::marker::PhantomData;

use crate::context::ContextRef;
use crate::support::LLVMString;
use crate::values::IntValue;
use crate::AddressSpace;

// Worth noting that types seem to be singletons. At the very least, primitives are.
// Though this is likely only true per thread since LLVM claims to not be very thread-safe.
#[derive(PartialEq, Eq, Clone, Copy)]
struct Type<'ctx> {
    ty: LLVMTypeRef,
    _marker: PhantomData<&'ctx ()>,
}

impl<'ctx> Type<'ctx> {
    unsafe fn new(ty: LLVMTypeRef) -> Self {
        assert!(!ty.is_null());

        Type {
            ty,
            _marker: PhantomData,
        }
    }

    fn const_zero(self) -> LLVMValueRef {
        unsafe {
            match LLVMGetTypeKind(self.ty) {
                LLVMTypeKind::LLVMMetadataTypeKind => LLVMConstPointerNull(self.ty),
                _ => LLVMConstNull(self.ty),
            }
        }
    }

    fn ptr_type(self, address_space: AddressSpace) -> PointerType<'ctx> {
        unsafe { PointerType::new(LLVMPointerType(self.ty, address_space.0)) }
    }

    fn vec_type(self, size: u32) -> VectorType<'ctx> {
        assert!(size != 0, "Vectors of size zero are not allowed.");
        // -- https://llvm.org/docs/LangRef.html#vector-type

        unsafe { VectorType::new(LLVMVectorType(self.ty, size)) }
    }

    #[llvm_versions(12..)]
    fn scalable_vec_type(self, size: u32) -> ScalableVectorType<'ctx> {
        assert!(size != 0, "Vectors of size zero are not allowed.");
        // -- https://llvm.org/docs/LangRef.html#vector-type

        unsafe { ScalableVectorType::new(LLVMScalableVectorType(self.ty, size)) }
    }

    #[cfg(not(feature = "experimental"))]
    fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        let mut param_types: Vec<LLVMTypeRef> = param_types.iter().map(|val| val.as_type_ref()).collect();
        unsafe {
            FunctionType::new(LLVMFunctionType(
                self.ty,
                param_types.as_mut_ptr(),
                param_types.len() as u32,
                is_var_args as i32,
            ))
        }
    }

    #[cfg(feature = "experimental")]
    fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        let pool: Bump<[usize; 16]> = Bump::uninit();
        let mut pool_start = None;

        for (i, param_type) in param_types.iter().enumerate() {
            let addr = pool.leak(param_type.as_type_ref()).expect("Found more than 16 params");

            if i == 0 {
                pool_start = Some(addr as *mut _);
            }
        }

        unsafe {
            FunctionType::new(LLVMFunctionType(
                self.ty,
                pool_start.unwrap_or(std::ptr::null_mut()),
                param_types.len() as u32,
                is_var_args as i32,
            ))
        }
    }

    #[allow(deprecated)]
    fn array_type(self, size: u32) -> ArrayType<'ctx> {
        unsafe { ArrayType::new(LLVMArrayType(self.ty, size)) }
    }

    fn get_undef(self) -> LLVMValueRef {
        unsafe { LLVMGetUndef(self.ty) }
    }

    #[llvm_versions(12..)]
    fn get_poison(&self) -> LLVMValueRef {
        unsafe { LLVMGetPoison(self.ty) }
    }

    fn get_alignment(self) -> IntValue<'ctx> {
        unsafe { IntValue::new(LLVMAlignOf(self.ty)) }
    }

    fn get_context(self) -> ContextRef<'ctx> {
        unsafe { ContextRef::new(LLVMGetTypeContext(self.ty)) }
    }

    // REVIEW: This should be known at compile time, maybe as a const fn?
    // On an enum or trait, this would not be known at compile time (unless
    // enum has only sized types for example)
    fn is_sized(self) -> bool {
        unsafe { LLVMTypeIsSized(self.ty) == 1 }
    }

    fn size_of(self) -> Option<IntValue<'ctx>> {
        if !self.is_sized() {
            return None;
        }

        unsafe { Some(IntValue::new(LLVMSizeOf(self.ty))) }
    }

    fn print_to_string(self) -> LLVMString {
        unsafe { LLVMString::new(LLVMPrintTypeToString(self.ty)) }
    }

    pub fn get_element_type(self) -> AnyTypeEnum<'ctx> {
        unsafe { AnyTypeEnum::new(LLVMGetElementType(self.ty)) }
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
