use crate::builder::{Builder, BuilderError};
use crate::typed::value::{TypedFloatValue, TypedIntValue};
use std::convert::TryFrom;

/// A builder that strictly enforces types using Const Generics.
#[derive(Debug)]
pub struct TypedBuilder<'a, 'ctx> {
    builder: &'a Builder<'ctx>,
}

impl<'a, 'ctx> TypedBuilder<'a, 'ctx> {
    pub fn new(builder: &'a Builder<'ctx>) -> Self {
        Self { builder }
    }

    /// Extends the underlying inner builder
    pub fn as_untyped(&self) -> &'a Builder<'ctx> {
        self.builder
    }

    pub fn build_int_add<const W: u32>(
        &self,
        lhs: TypedIntValue<'ctx, W>,
        rhs: TypedIntValue<'ctx, W>,
        name: &str,
    ) -> Result<TypedIntValue<'ctx, W>, BuilderError> {
        let result = self.builder.build_int_add(lhs.as_untyped(), rhs.as_untyped(), name)?;
        Ok(TypedIntValue::try_from(result).unwrap())
    }

    pub fn build_int_sub<const W: u32>(
        &self,
        lhs: TypedIntValue<'ctx, W>,
        rhs: TypedIntValue<'ctx, W>,
        name: &str,
    ) -> Result<TypedIntValue<'ctx, W>, BuilderError> {
        let result = self.builder.build_int_sub(lhs.as_untyped(), rhs.as_untyped(), name)?;
        Ok(TypedIntValue::try_from(result).unwrap())
    }

    pub fn build_int_mul<const W: u32>(
        &self,
        lhs: TypedIntValue<'ctx, W>,
        rhs: TypedIntValue<'ctx, W>,
        name: &str,
    ) -> Result<TypedIntValue<'ctx, W>, BuilderError> {
        let result = self.builder.build_int_mul(lhs.as_untyped(), rhs.as_untyped(), name)?;
        Ok(TypedIntValue::try_from(result).unwrap())
    }

    pub fn build_float_add<const W: u32>(
        &self,
        lhs: TypedFloatValue<'ctx, W>,
        rhs: TypedFloatValue<'ctx, W>,
        name: &str,
    ) -> Result<TypedFloatValue<'ctx, W>, BuilderError> {
        let result = self.builder.build_float_add(lhs.as_untyped(), rhs.as_untyped(), name)?;
        Ok(TypedFloatValue::try_from(result).unwrap())
    }

    pub fn build_float_mul<const W: u32>(
        &self,
        lhs: TypedFloatValue<'ctx, W>,
        rhs: TypedFloatValue<'ctx, W>,
        name: &str,
    ) -> Result<TypedFloatValue<'ctx, W>, BuilderError> {
        let result = self.builder.build_float_mul(lhs.as_untyped(), rhs.as_untyped(), name)?;
        Ok(TypedFloatValue::try_from(result).unwrap())
    }
}

impl<'ctx> Builder<'ctx> {
    /// Creates a strictly-typed version of this builder.
    pub fn as_typed<'a>(&'a self) -> TypedBuilder<'a, 'ctx> {
        TypedBuilder::new(self)
    }
}
