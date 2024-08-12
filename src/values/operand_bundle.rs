use crate::context::Context;
use crate::support::to_c_str;
use crate::values::{AnyValueEnum, AsValueRef, BasicValueEnum, CallSiteValue};
use llvm_sys::core::{
    LLVMCreateOperandBundle, LLVMDisposeOperandBundle, LLVMGetNumOperandBundleArgs, LLVMGetNumOperandBundles,
    LLVMGetOperandBundleArgAtIndex, LLVMGetOperandBundleAtIndex, LLVMGetOperandBundleTag,
};
use llvm_sys::prelude::{LLVMOperandBundleRef, LLVMValueRef};
use std::cell::Cell;
use std::ffi::CStr;
use std::marker::PhantomData;

/// One of an instruction's operand bundles.
#[derive(Debug)]
pub struct OperandBundle<'ctx> {
    bundle: Cell<LLVMOperandBundleRef>,
    _marker: PhantomData<&'ctx Context>,
}

/// Iterator over an instruction's operand bundles.
#[derive(Debug)]
pub struct OperandBundleIter<'a, 'ctx> {
    instruction: &'a CallSiteValue<'ctx>,
    current: u32,
    size: u32,
}

/// Iterator over an operand bundle's arguments.
#[derive(Debug)]
pub struct OperandBundleArgsIter<'a, 'ctx> {
    bundle: &'a OperandBundle<'ctx>,
    current: u32,
    size: u32,
}

impl<'ctx> OperandBundle<'ctx> {
    /// Get an operand bundle from a [LLVMOperandBundleRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and represent an operand bundle.
    pub unsafe fn new(bundle: LLVMOperandBundleRef) -> Self {
        Self {
            bundle: Cell::new(bundle),
            _marker: PhantomData,
        }
    }

    /// Create a new operand bundle.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::values::OperandBundle;
    ///
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    ///
    /// let op_bundle = OperandBundle::create("tag", &[i32_type.const_zero().into()]);
    ///
    /// assert_eq!(op_bundle.get_tag().unwrap(), "tag");
    /// let arg = op_bundle.get_args().nth(0).unwrap().into_int_value();
    /// assert!(arg.is_const());
    /// assert_eq!(arg.get_zero_extended_constant().unwrap(), 0);
    /// ```
    pub fn create(tag: &str, args: &[AnyValueEnum<'ctx>]) -> Self {
        let c_tag = to_c_str(tag);
        let mut args: Vec<LLVMValueRef> = args.iter().map(|value| value.as_value_ref()).collect();

        unsafe {
            let bundle = LLVMCreateOperandBundle(c_tag.as_ptr(), tag.len(), args.as_mut_ptr(), args.len() as u32);
            Self::new(bundle)
        }
    }

    /// Acquire the underlying raw pointer belonging to this `OperandBundle` type.
    pub fn as_mut_ptr(&self) -> LLVMOperandBundleRef {
        self.bundle.get()
    }

    /// Get this operand bundle's tag.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::values::OperandBundle;
    ///
    /// let op_bundle = OperandBundle::create("tag", &[]);
    /// assert_eq!(op_bundle.get_tag().unwrap(), "tag");
    /// ```
    pub fn get_tag(&self) -> Result<&str, std::str::Utf8Error> {
        unsafe {
            let mut size = 0usize;
            let tag = LLVMGetOperandBundleTag(self.bundle.get(), &mut size as *mut usize);
            CStr::from_ptr(tag).to_str()
        }
    }

    /// Iterate over this operand bundle's arguments.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::values::OperandBundle;
    ///
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    /// let f32_type = context.f32_type();
    ///
    /// let op_bundle = OperandBundle::create("tag", &[i32_type.const_zero().into(), f32_type.const_float(1.23).into()]);
    /// assert_eq!(op_bundle.get_args().count(), 2);
    /// assert_eq!(op_bundle.get_args().len(), 2);
    /// ```
    pub fn get_args(&self) -> OperandBundleArgsIter<'_, 'ctx> {
        OperandBundleArgsIter::new(self)
    }
}

impl Drop for OperandBundle<'_> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeOperandBundle(self.as_mut_ptr()) }
    }
}

impl<'a, 'ctx> OperandBundleIter<'a, 'ctx> {
    pub(crate) fn new(instruction: &'a CallSiteValue<'ctx>) -> Self {
        let size = unsafe { LLVMGetNumOperandBundles(instruction.as_value_ref()) };

        Self {
            instruction,
            current: 0,
            size,
        }
    }
}

impl<'ctx> Iterator for OperandBundleIter<'_, 'ctx> {
    type Item = OperandBundle<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.size {
            let bundle = unsafe {
                OperandBundle::new(LLVMGetOperandBundleAtIndex(
                    self.instruction.as_value_ref(),
                    self.current,
                ))
            };
            self.current += 1;
            Some(bundle)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.size - self.current) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for OperandBundleIter<'_, '_> {}

impl<'a, 'ctx> OperandBundleArgsIter<'a, 'ctx> {
    fn new(bundle: &'a OperandBundle<'ctx>) -> Self {
        let size = unsafe { LLVMGetNumOperandBundleArgs(bundle.as_mut_ptr()) };
        Self {
            bundle,
            current: 0,
            size,
        }
    }
}

impl<'ctx> Iterator for OperandBundleArgsIter<'_, 'ctx> {
    type Item = BasicValueEnum<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.size {
            unsafe {
                let arg = LLVMGetOperandBundleArgAtIndex(self.bundle.as_mut_ptr(), self.current);
                self.current += 1;
                Some(BasicValueEnum::new(arg))
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.size - self.current) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for OperandBundleArgsIter<'_, '_> {}
