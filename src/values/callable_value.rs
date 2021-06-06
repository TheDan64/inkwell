use std::convert::TryFrom;
use either::Either;

use crate::values::{AsValueRef, BasicValueEnum, InstructionValue, Value};
use crate::values::FunctionValue;
use crate::values::PointerValue;

use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::core::{LLVMGetTypeKind, LLVMGetElementType, LLVMTypeOf};
use llvm_sys::LLVMTypeKind;

/// A value that can be called with the [`Builder::build_call`] or [`Builder::build_invoke`] instruction.
///
/// To convert a [`FunctionValue`] to a [`CallableValue`], use the [`std::convert::From`] trait:
///
/// ```rust
/// CallableValue::from(my_function_value);
/// ```
///
/// To convert a [`FunctionValue`] to a [`CallableValue`], use the [`std::convert::TryFrom`] trait:
///
/// ```rust
/// use std::convert::TryFrom;
///
/// CallableValue::try_from(my_pointer_value);
/// ```
///
/// The [`std::convert::TryFrom::try_from`] method returns a [`std::result::Result`] because to be callable,
/// the pointer must point to a function.
#[derive(Debug)]
pub struct CallableValue<'ctx>(FunctionOrPointerValue<'ctx>);

type FunctionOrPointerValue<'ctx> = Either<FunctionValue<'ctx>, PointerValue<'ctx>>;

impl<'ctx> crate::values::traits::AsValueRef for CallableValue<'ctx> {
    fn as_value_ref(&self) -> LLVMValueRef {
        use either::Either::*;

        match self.0 {
            Left(function) => function.as_value_ref(),
            Right(pointer) => pointer.as_value_ref(),
        }
    }
}

impl<'ctx> crate::values::AnyValue<'ctx> for CallableValue<'ctx> {}

impl<'ctx> From<FunctionValue<'ctx>> for CallableValue<'ctx> {
    fn from(value: FunctionValue<'ctx>) -> Self {
        Self(Either::Left(value))
    }
}

impl<'ctx> TryFrom<PointerValue<'ctx>> for CallableValue<'ctx> {
    type Error = ();

    fn try_from(value: PointerValue<'ctx>) -> Result<Self, Self::Error> {
        // If using a pointer value, we must validate it's a valid function ptr
        let value_ref = value.as_value_ref();
        let ty_kind = unsafe { LLVMGetTypeKind(LLVMGetElementType(LLVMTypeOf(value_ref))) };
        let is_a_fn_ptr = matches!(ty_kind, LLVMTypeKind::LLVMFunctionTypeKind);

        if is_a_fn_ptr {
            Ok(Self(Either::Right(value)))
        } else {
            Err(())
        }
    }
}
