use llvm_sys::prelude::LLVMTypeRef;

use crate::context::ContextRef;
use crate::types::traits::AsTypeRef;
use crate::types::{Type, FunctionType, BasicTypeEnum, ArrayType, VectorType};
use crate::values::{IntValue, MetadataValue};
use crate::types::enums::BasicMetadataTypeEnum;

/// A `MetadataType` is the type of a metadata.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MetadataType<'ctx> {
    metadata_type: Type<'ctx>,
}

impl<'ctx> MetadataType<'ctx> {
    #[llvm_versions(6.0..=latest)]
    pub(crate) unsafe fn new(metadata_type: LLVMTypeRef) -> Self {
        assert!(!metadata_type.is_null());

        MetadataType {
            metadata_type: Type::new(metadata_type),
        }
    }

    /// Creates a `FunctionType` with this `MetadataType` for its return type.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let md_type = context.metadata_type();
    /// let fn_type = md_type.fn_type(&[], false);
    /// ```
    #[llvm_versions(6.0..=latest)]
    pub fn fn_type(self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
        self.metadata_type.fn_type(param_types, is_var_args)
    }

    /// Gets a reference to the `Context` this `MetadataType` was created in.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let md_type = context.metadata_type();
    ///
    /// assert_eq!(*md_type.get_context(), context);
    /// ```
    #[llvm_versions(6.0..=latest)]
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.metadata_type.get_context()
    }
}

impl AsTypeRef for MetadataType<'_> {
    #[llvm_versions(6.0..=latest)]
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.metadata_type.ty
    }

    #[llvm_versions(3.6..6.0)]
    fn as_type_ref(&self) -> LLVMTypeRef {
        unimplemented!("MetadataType is only available in LLVM > 6.0")
    }
}
