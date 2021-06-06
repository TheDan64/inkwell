use std::convert::{TryFrom, TryInto};
use either::Either;

use crate::values::{AsValueRef, BasicValueEnum, InstructionValue, Value};
use crate::values::FunctionValue;
use crate::values::PointerValue;

use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::core::{LLVMGetTypeKind, LLVMGetElementType, LLVMTypeOf, LLVMGetReturnType};
use llvm_sys::LLVMTypeKind;

use crate::builder::Builder;

/// A value that can be called with the [`Builder::build_call`] instruction.
///
/// In practice, the `F : Into<CallableValue<'ctx>>` bound of [`Builder::build_call`] means it is
/// possible to pass a [`FunctionValue`] to `build_call` directly. It will be implicitly converted
/// into a `CallableValue`.
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
/// let ret_val = builder.build_call(fn_value, &[i32_arg], "call")
///     .try_as_basic_value()
///     .left()
///     .unwrap();
///
/// builder.build_return(Some(&ret_val));
/// ```
///
/// A [`PointerValue`] cannot be implicitly converted to a `CallableValue` because the pointer may
/// point to a non-function value. Instead we can use [`TryFrom`] and [`TryInto`] to handle this
/// failure case explicitly.
///
/// ```no_run
/// use std::convert::TryInto;
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
/// let callable_value = fn_pointer_value.try_into().unwrap();
///
/// let ret_val = builder.build_call(callable_value, &[i32_arg], "call")
///     .try_as_basic_value()
///     .left()
///     .unwrap();
///
/// builder.build_return(Some(&ret_val));
/// ```
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

impl<'ctx> CallableValue<'ctx> {
    pub(crate) fn returns_void(&self) -> bool {
        let return_type = unsafe {
            LLVMGetTypeKind(LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(self.as_value_ref()))))
        };

        matches!(LLVMTypeKind::LLVMVoidTypeKind, return_type)
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
