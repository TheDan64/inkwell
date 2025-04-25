use either::Either;
use std::convert::TryFrom;
use std::fmt::{self, Display};

use crate::types::AsTypeRef;
use crate::values::AsValueRef;
use crate::values::{AnyValue, FunctionValue, PointerValue};

use llvm_sys::core::{LLVMGetElementType, LLVMGetTypeKind, LLVMTypeOf};
use llvm_sys::prelude::LLVMTypeRef;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMTypeKind;

/// A value that can be called with the [`build_call`] instruction.
///
/// In practice, the `F : Into<CallableValue<'ctx>>` bound of [`build_call`] means it is
/// possible to pass a [`FunctionValue`] to [`build_call`] directly. It will be implicitly converted
/// into a `CallableValue`.
///
/// [`build_call`]: crate::builder::Builder::build_call
///
/// ```no_run
/// use inkwell::context::Context;
///
/// // A simple function which calls itself:
/// let context = Context::create();
/// let module = context.create_module("ret");
/// let builder = context.create_builder();
/// let i32_type = context.i32_type();
/// let fn_type = i32_type.fn_type(&[i32_type.into()], false);
/// let fn_value = module.add_function("ret", fn_type, None);
/// let entry = context.append_basic_block(fn_value, "entry");
/// let i32_arg = fn_value.get_first_param().unwrap();
///
/// builder.position_at_end(entry);
///
/// let ret_val = builder.build_call(fn_value, &[i32_arg.into()], "call").unwrap()
///     .try_as_basic_value()
///     .left()
///     .unwrap();
///
/// builder.build_return(Some(&ret_val)).unwrap();
/// ```
///
/// A [`PointerValue`] cannot be implicitly converted to a `CallableValue` because the pointer may
/// point to a non-function value. Instead we can use [`TryFrom`] to handle this failure case explicitly.
///
/// ```no_run
/// use std::convert::TryFrom;
/// use inkwell::context::Context;
/// use inkwell::values::CallableValue;
///
/// // A simple function which calls itself:
/// let context = Context::create();
/// let module = context.create_module("ret");
/// let builder = context.create_builder();
/// let i32_type = context.i32_type();
/// let fn_type = i32_type.fn_type(&[i32_type.into()], false);
/// let fn_value = module.add_function("ret", fn_type, None);
/// let entry = context.append_basic_block(fn_value, "entry");
/// let i32_arg = fn_value.get_first_param().unwrap();
///
/// builder.position_at_end(entry);
///
/// // take a pointer to the function value
/// let fn_pointer_value = fn_value.as_global_value().as_pointer_value();
///
/// // convert that pointer value into a callable value
/// // explicitly handling the failure case (here with `unwrap`)
/// let callable_value = CallableValue::try_from(fn_pointer_value).unwrap();
///
/// let ret_val = builder.build_call(callable_value, &[i32_arg.into()], "call").unwrap()
///     .try_as_basic_value()
///     .left()
///     .unwrap();
///
/// builder.build_return(Some(&ret_val)).unwrap();
/// ```
#[derive(Debug)]
pub struct CallableValue<'ctx>(Either<FunctionValue<'ctx>, PointerValue<'ctx>>);

unsafe impl AsValueRef for CallableValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        use either::Either::*;

        match self.0 {
            Left(function) => function.as_value_ref(),
            Right(pointer) => pointer.as_value_ref(),
        }
    }
}

unsafe impl<'ctx> AnyValue<'ctx> for CallableValue<'ctx> {}

unsafe impl AsTypeRef for CallableValue<'_> {
    fn as_type_ref(&self) -> LLVMTypeRef {
        use either::Either::*;

        match self.0 {
            Left(function) => function.get_type().as_type_ref(),
            Right(pointer) => pointer.get_type().get_element_type().as_type_ref(),
        }
    }
}

impl CallableValue<'_> {
    #[llvm_versions(..=14)]
    pub(crate) fn returns_void(&self) -> bool {
        use llvm_sys::core::LLVMGetReturnType;

        let return_type =
            unsafe { LLVMGetTypeKind(LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(self.as_value_ref())))) };

        matches!(return_type, LLVMTypeKind::LLVMVoidTypeKind)
    }
}

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

impl Display for CallableValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}
