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
    pub fn get_context(self) -> ContextRef<'ctx> {
        self.metadata_type.get_context()
    }
}

impl AsTypeRef for MetadataType<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.metadata_type.ty
    }
}
