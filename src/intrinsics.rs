#[llvm_versions(9..)]
use llvm_sys::core::{LLVMGetIntrinsicDeclaration, LLVMIntrinsicIsOverloaded, LLVMLookupIntrinsicID};
use llvm_sys::prelude::LLVMTypeRef;

use crate::module::Module;
use crate::types::{AsTypeRef, BasicTypeEnum};
use crate::values::FunctionValue;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Intrinsic {
    id: u32,
}

/// A wrapper around LLVM intrinsic id
///
/// To call it you would need to create a declaration inside a module using [`Self::get_declaration()`].
#[llvm_versions(9..)]
impl Intrinsic {
    /// Create an Intrinsic object from raw LLVM intrinsic id
    ///
    /// SAFETY: the id is a valid LLVM intrinsic ID
    pub(crate) unsafe fn new(id: u32) -> Self {
        Self { id }
    }

    /// Find llvm intrinsic id from name
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::{intrinsics::Intrinsic, context::Context};
    ///
    /// let trap_intrinsic = Intrinsic::find("llvm.trap").unwrap();
    ///
    /// let context = Context::create();
    /// let module = context.create_module("trap");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("trap", fn_type, None);
    /// let entry = context.append_basic_block(fn_value, "entry");
    ///
    /// let trap_function = trap_intrinsic.get_declaration(&module, &[]).unwrap();
    ///
    /// builder.position_at_end(entry);
    /// builder.build_call(trap_function, &[], "trap_call");
    /// ```
    pub fn find(name: &str) -> Option<Self> {
        let id = unsafe { LLVMLookupIntrinsicID(name.as_ptr() as *const ::libc::c_char, name.len()) };

        if id == 0 {
            return None;
        }

        Some(unsafe { Intrinsic::new(id) })
    }

    /// Check if specified intrinsic is overloaded
    ///
    /// Overloaded intrinsics need some argument types to be specified to declare them
    pub fn is_overloaded(&self) -> bool {
        unsafe { LLVMIntrinsicIsOverloaded(self.id) != 0 }
    }

    /// Create or insert the declaration of an intrinsic.
    ///
    /// For overloaded intrinsics, parameter types must be provided to uniquely identify an overload.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::{intrinsics::Intrinsic, context::Context};
    ///
    /// let trap_intrinsic = Intrinsic::find("llvm.trap").unwrap();
    ///
    /// let context = Context::create();
    /// let module = context.create_module("trap");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("trap", fn_type, None);
    /// let entry = context.append_basic_block(fn_value, "entry");
    ///
    /// let trap_function = trap_intrinsic.get_declaration(&module, &[]).unwrap();
    ///
    /// builder.position_at_end(entry);
    /// builder.build_call(trap_function, &[], "trap_call");
    /// ```
    pub fn get_declaration<'ctx>(
        &self,
        module: &Module<'ctx>,
        param_types: &[BasicTypeEnum],
    ) -> Option<FunctionValue<'ctx>> {
        let mut param_types: Vec<LLVMTypeRef> = param_types.iter().map(|val| val.as_type_ref()).collect();

        // param_types should be empty for non-overloaded intrinsics (I think?)
        // for overloaded intrinsics they determine the overload used

        if self.is_overloaded() && param_types.is_empty() {
            // LLVM crashes otherwise
            return None;
        }

        let res = unsafe {
            FunctionValue::new(LLVMGetIntrinsicDeclaration(
                module.module.get(),
                self.id,
                param_types.as_mut_ptr(),
                param_types.len(),
            ))
        };

        res
    }
}
