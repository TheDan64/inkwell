#[llvm_versions(8.0..=latest)]
use llvm_sys::core::{LLVMGetIntrinsicDeclaration, LLVMIntrinsicIsOverloaded, LLVMLookupIntrinsicID};
use llvm_sys::prelude::LLVMTypeRef;

use crate::module::Module;
use crate::types::{AsTypeRef, BasicTypeEnum, FunctionType};
use crate::values::FunctionValue;

#[derive(Debug)]
pub struct Intrinsic {
    id: u32,
}

impl Intrinsic {
    // SAFETY: the id is a valid LLVM intrinsic ID
    pub(crate) unsafe fn new(id: u32) -> Self {
        Self {
            id
        }
    }

    pub fn find(name: &str) -> Option<Self> {
        let id = unsafe {
            LLVMLookupIntrinsicID(name.as_ptr() as *const ::libc::c_char, name.len())
        };

        if id == 0 {
            return None;
        }

        return Some(unsafe {
            Intrinsic::new(id)
        });
    }

    pub fn is_overloaded(&self) -> bool {
        unsafe {
            LLVMIntrinsicIsOverloaded(self.id) != 0
        }
    }

    pub fn get_declaration<'ctx>(&self, module: &Module<'ctx>, param_types: &[BasicTypeEnum]) -> Option<FunctionValue<'ctx>> {
        let mut param_types: Vec<LLVMTypeRef> = param_types.iter()
            .map(|val| val.as_type_ref())
            .collect();

        // param_types should be empty for non-overloaded intrinsics (I think?)
        // for overloaded intrinsics they determine the overload used

        if self.is_overloaded() && param_types.is_empty() {
            // LLVM crashes otherwise
            return None;
        }

        let res = unsafe {
            FunctionValue::new(
                LLVMGetIntrinsicDeclaration(
                    module.module.get(),
                    self.id,
                    param_types.as_mut_ptr(),
                    param_types.len()
                )
            )
        };

        res
    }
}