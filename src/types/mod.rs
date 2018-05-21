mod array_type;
mod enums;
mod float_type;
mod fn_type;
mod int_type;
mod ptr_type;
mod struct_type;
mod traits;
mod vec_type;
mod void_type;

pub use types::array_type::ArrayType;
pub use types::enums::{AnyTypeEnum, BasicTypeEnum};
pub use types::float_type::FloatType;
pub use types::fn_type::FunctionType;
pub use types::int_type::IntType;
pub use types::ptr_type::PointerType;
pub use types::struct_type::StructType;
pub use types::traits::{AnyType, BasicType};
pub use types::vec_type::VectorType;
pub use types::void_type::VoidType;
pub(crate) use types::traits::AsTypeRef;

#[cfg(not(feature = "llvm3-6"))]
use llvm_sys::core::LLVMDumpType;
use llvm_sys::core::{LLVMAlignOf, LLVMGetTypeContext, LLVMFunctionType, LLVMArrayType, LLVMGetTypeKind, LLVMGetUndef, LLVMPointerType, LLVMPrintTypeToString, LLVMTypeIsSized, LLVMSizeOf, LLVMVectorType, LLVMConstPointerNull};
use llvm_sys::LLVMTypeKind;
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use std::ffi::CStr;
use std::fmt;
use std::rc::Rc;

use AddressSpace;
use context::{Context, ContextRef};
use values::{IntValue, PointerValue};

// Worth noting that types seem to be singletons. At the very least, primitives are.
// Though this is likely only true per thread since LLVM claims to not be very thread-safe.
// REVIEW: Maybe move this into its own module?
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
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm5-0")))]
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

    // FIXME: LLVMPrintTypeToString actually returns an owned string according to
    // https://github.com/llvm-mirror/llvm/blob/master/include/llvm-c/Core.h#L994
    // We should create a LLVMString wrapper type which derefs to &CStr (or &str
    // if safely possible?) and calls LLVMDisposeMessage on drop
    fn print_to_string(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(self.type_))
        }
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
