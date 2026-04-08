use std::convert::TryFrom;
use crate::values::IntValue;

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
