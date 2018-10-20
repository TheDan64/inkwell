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

pub use types::array_type::ArrayType;
pub use types::enums::{AnyTypeEnum, BasicTypeEnum};
pub use types::float_type::FloatType;
pub use types::fn_type::FunctionType;
pub use types::int_type::IntType;
pub use types::ptr_type::PointerType;
pub use types::struct_type::StructType;
pub use types::traits::{AnyType, BasicType, IntMathType, FloatMathType, PointerMathType};
pub use types::vec_type::VectorType;
pub use types::void_type::VoidType;
pub(crate) use types::traits::AsTypeRef;

#[llvm_versions(3.7 => 4.0)]
use llvm_sys::core::LLVMDumpType;
use llvm_sys::core::{LLVMAlignOf, LLVMGetTypeContext, LLVMFunctionType, LLVMArrayType, LLVMGetUndef, LLVMPointerType, LLVMPrintTypeToString, LLVMTypeIsSized, LLVMSizeOf, LLVMVectorType, LLVMConstPointerNull, LLVMGetElementType, LLVMConstNull};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use std::fmt;
use std::rc::Rc;

use AddressSpace;
use context::{Context, ContextRef};
use support::LLVMString;
use values::IntValue;

// Worth noting that types seem to be singletons. At the very least, primitives are.
// Though this is likely only true per thread since LLVM claims to not be very thread-safe.
#[derive(PartialEq, Eq, Clone, Copy)]
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

    // REVIEW: LLVM 5.0+ seems to have removed this function in release mode
    // and so will fail to link when used. I've decided to remove it from 5.0+
    // for now. We should consider removing it altogether since print_to_string
    // could be used and manually written to stderr in rust...
    #[llvm_versions(3.7 => 4.0)]
    fn print_to_stderr(&self) {
        unsafe {
            LLVMDumpType(self.type_);
        }
    }

    // Even though the LLVM fuction has the word "Pointer", it doesn't seem to create
    // a pointer at all, just a null value of the current type...
    fn const_null(&self) -> LLVMValueRef {
        unsafe {
            LLVMConstPointerNull(self.type_)
        }
    }

    fn const_zero(&self) -> LLVMValueRef {
        unsafe {
            LLVMConstNull(self.type_)
        }
    }

    fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        let ptr_type = unsafe {
            LLVMPointerType(self.type_, address_space as u32)
        };

        PointerType::new(ptr_type)
    }

    fn vec_type(&self, size: u32) -> VectorType {
        let vec_type = unsafe {
            LLVMVectorType(self.type_, size)
        };

        VectorType::new(vec_type)
    }

    // REVIEW: Can you make a FunctionType from a FunctionType???
    fn fn_type(&self, param_types: &[BasicTypeEnum], is_var_args: bool) -> FunctionType {
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

    fn get_alignment(&self) -> IntValue {
        let val = unsafe {
            LLVMAlignOf(self.type_)
        };

        IntValue::new(val)
    }

    // REVIEW: get_context could potentially be unsafe and UB if Context is shared across threads.
    // Not currently possible AFAIK but might be in the future if we decide to support multithreading.
    fn get_context(&self) -> ContextRef {
        // We don't return an option because LLVM seems
        // to always assign a context, even to types
        // created without an explicit context, somehow

        let context = unsafe {
            LLVMGetTypeContext(self.type_)
        };

        // REVIEW: This probably should be somehow using the existing context Rc
        ContextRef::new(Context::new(Rc::new(context)))
    }

    // REVIEW: This should be known at compile time, maybe as a const fn?
    // On an enum or trait, this would not be known at compile time (unless
    // enum has only sized types for example)
    fn is_sized(&self) -> bool {
        unsafe {
            LLVMTypeIsSized(self.type_) == 1
        }
    }

    // REVIEW: Option<IntValue>? If we want to provide it on enums that
    // contain unsized types
    fn size_of(&self) -> IntValue {
        debug_assert!(self.is_sized());

        let int_value = unsafe {
            LLVMSizeOf(self.type_)
        };

        IntValue::new(int_value)
    }

    fn print_to_string(&self) -> LLVMString {
        let c_string_ptr = unsafe {
            LLVMPrintTypeToString(self.type_)
        };

        LLVMString::new(c_string_ptr)
    }

    pub fn get_element_type(&self) -> AnyTypeEnum {
        let ptr = unsafe {
            LLVMGetElementType(self.type_)
        };

        AnyTypeEnum::new(ptr)
    }

}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = self.print_to_string();

        f.debug_struct("Type")
            .field("address", &self.type_)
            .field("llvm_type", &llvm_type)
            .finish()
    }
}
