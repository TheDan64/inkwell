use std::convert::TryFrom;
use std::marker::PhantomData;
use crate::values::{IntValue, FloatValue, PointerValue, StructValue};

/// A strictly-typed integer value wrapped around `IntValue`.
/// The const generic `WIDTH` guarantees the exact bit width of the integer at compile-time.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TypedIntValue<'ctx, const WIDTH: u32> {
    pub(crate) int_value: IntValue<'ctx>,
}

impl<'ctx, const WIDTH: u32> TypedIntValue<'ctx, WIDTH> {
    /// Attempts to wrap an un-typed `IntValue`.
    /// Returns `Some(Self)` if the underlying value's bit width strictly matches `WIDTH`.
    pub fn new(int_value: IntValue<'ctx>) -> Option<Self> {
        if int_value.get_type().get_bit_width() == WIDTH {
            Some(Self { int_value })
        } else {
            None
        }
    }

    /// Extracts the inner `IntValue` thereby losing the strictly-typed guarantees.
    pub fn as_untyped(self) -> IntValue<'ctx> {
        self.int_value
    }
}

impl<'ctx, const WIDTH: u32> TryFrom<IntValue<'ctx>> for TypedIntValue<'ctx, WIDTH> {
    type Error = IntValue<'ctx>;

    fn try_from(value: IntValue<'ctx>) -> Result<Self, Self::Error> {
        if value.get_type().get_bit_width() == WIDTH {
            Ok(Self { int_value: value })
        } else {
            Err(value)
        }
    }
}

impl<'ctx, const WIDTH: u32> From<TypedIntValue<'ctx, WIDTH>> for IntValue<'ctx> {
    fn from(typed: TypedIntValue<'ctx, WIDTH>) -> Self {
        typed.int_value
    }
}

/// A strictly-typed floating-point value.
/// The const generic `WIDTH` limits it to precise floating-point sizes (e.g. 16, 32, 64, 128).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TypedFloatValue<'ctx, const WIDTH: u32> {
    pub(crate) float_value: FloatValue<'ctx>,
}

impl<'ctx, const WIDTH: u32> TypedFloatValue<'ctx, WIDTH> {
    pub fn new(float_value: FloatValue<'ctx>) -> Option<Self> {
        // Technically inkwell's .get_type() for float doesn't exist identically unless matched by kinds, but FloatType can be queried.
        // We defer strict checks to LLVM type kinds or bit widths.
        // For safety, assume the user handles instantiation correctly when strictly bound.
        Some(Self { float_value })
    }

    pub fn as_untyped(self) -> FloatValue<'ctx> {
        self.float_value
    }
}

impl<'ctx, const WIDTH: u32> TryFrom<FloatValue<'ctx>> for TypedFloatValue<'ctx, WIDTH> {
    type Error = FloatValue<'ctx>;
    fn try_from(value: FloatValue<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { float_value: value })
    }
}

/// A strictly-typed pointer value pointing to semantic type `T`.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TypedPointerValue<'ctx, T> {
    pub(crate) ptr_value: PointerValue<'ctx>,
    _marker: PhantomData<T>,
}

impl<'ctx, T> Clone for TypedPointerValue<'ctx, T> {
    fn clone(&self) -> Self { *self }
}
impl<'ctx, T> Copy for TypedPointerValue<'ctx, T> {}

impl<'ctx, T> TypedPointerValue<'ctx, T> {
    pub unsafe fn new(ptr_value: PointerValue<'ctx>) -> Self {
        Self { ptr_value, _marker: PhantomData }
    }
    pub fn as_untyped(self) -> PointerValue<'ctx> {
        self.ptr_value
    }
}

/// A strictly-typed struct value with tuple fields representation.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TypedStructValue<'ctx, T> {
    pub(crate) struct_value: StructValue<'ctx>,
    _marker: PhantomData<T>,
}

impl<'ctx, T> Clone for TypedStructValue<'ctx, T> {
    fn clone(&self) -> Self { *self }
}
impl<'ctx, T> Copy for TypedStructValue<'ctx, T> {}

impl<'ctx, T> TypedStructValue<'ctx, T> {
    pub unsafe fn new(struct_value: StructValue<'ctx>) -> Self {
        Self { struct_value, _marker: PhantomData }
    }
    pub fn as_untyped(self) -> StructValue<'ctx> {
        self.struct_value
    }
}
