use std::{
    cell::RefCell, ffi::CStr, marker::PhantomData, mem::transmute_copy, ops::Deref, ptr::null_mut,
    rc::Rc,
};

use llvm_sys::{
    error::{
        LLVMConsumeError, LLVMCreateStringError, LLVMErrorRef, LLVMErrorTypeId,
        LLVMGetErrorMessage, LLVMGetErrorTypeId,
    },
    orc2::{
        lljit::{
            LLVMOrcCreateLLJIT, LLVMOrcDisposeLLJIT, LLVMOrcDisposeLLJITBuilder,
            LLVMOrcLLJITAddLLVMIRModule, LLVMOrcLLJITAddLLVMIRModuleWithRT, LLVMOrcLLJITBuilderRef,
            LLVMOrcLLJITGetMainJITDylib, LLVMOrcLLJITLookup, LLVMOrcLLJITRef,
        },
        LLVMOrcCreateNewThreadSafeContext, LLVMOrcCreateNewThreadSafeModule,
        LLVMOrcDisposeThreadSafeContext, LLVMOrcDisposeThreadSafeModule,
        LLVMOrcJITDylibCreateResourceTracker, LLVMOrcJITDylibGetDefaultResourceTracker,
        LLVMOrcJITDylibRef, LLVMOrcReleaseResourceTracker, LLVMOrcResourceTrackerRef,
        LLVMOrcThreadSafeContextGetContext, LLVMOrcThreadSafeContextRef,
        LLVMOrcThreadSafeModuleRef,
    },
};

use crate::{
    context::Context,
    module::Module,
    support::{to_c_str, LLVMString},
    targets::{InitializationConfig, Target},
};

#[derive(Debug)]
pub struct LLVMError {
    error: LLVMErrorRef,
}

impl LLVMError {
    fn new(error: LLVMErrorRef) -> Result<(), Self> {
        if error.is_null() {
            return Ok(());
        }
        let error_type = unsafe { LLVMGetErrorTypeId(error) };
        if error_type.is_null() {
            Ok(())
        } else {
            Err(LLVMError { error })
        }
    }
    // Null type id == success
    pub fn get_type_id(&self) -> LLVMErrorTypeId {
        // FIXME: Don't expose LLVMErrorTypeId
        unsafe { LLVMGetErrorTypeId(self.error) }
    }
    pub fn get_message(self) -> LLVMString {
        unsafe { LLVMString::new(LLVMGetErrorMessage(self.error)) }
    }
    pub fn new_string_error(message: &str) -> Self {
        let error = unsafe { LLVMCreateStringError(to_c_str(message).as_ptr()) };
        LLVMError { error }
    }
}

impl Deref for LLVMError {
    type Target = CStr;

    fn deref(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetErrorMessage(self.error)) // FIXME: LLVMGetErrorMessage consumes the error, needs LLVMDisposeErrorMessage after
        }
    }
}

impl Drop for LLVMError {
    fn drop(&mut self) {
        unsafe { LLVMConsumeError(self.error) }
    }
}

#[derive(Debug)]
pub struct ThreadSafeContext {
    thread_safe_context: LLVMOrcThreadSafeContextRef,
    context: Context,
}

impl ThreadSafeContext {
    fn new(thread_safe_context: LLVMOrcThreadSafeContextRef) -> Self {
        assert!(!thread_safe_context.is_null());
        let context;
        unsafe {
            context = Context::new(LLVMOrcThreadSafeContextGetContext(thread_safe_context));
        }
        ThreadSafeContext {
            thread_safe_context,
            context,
        }
    }
    pub fn create() -> Self {
        unsafe { ThreadSafeContext::new(LLVMOrcCreateNewThreadSafeContext()) }
    }
    pub fn context(&self) -> &Context {
        &self.context
    }
    pub fn create_module<'ctx>(&'ctx self, module: Module<'ctx>) -> ThreadSafeModule<'ctx> {
        unsafe {
            ThreadSafeModule::new(
                LLVMOrcCreateNewThreadSafeModule(module.module.get(), self.thread_safe_context),
                module,
            )
        }
    }
}
impl Drop for ThreadSafeContext {
    fn drop(&mut self) {
        println!("Dispose context");
        unsafe {
            LLVMOrcDisposeThreadSafeContext(self.thread_safe_context);
        }
        self.context.context = null_mut(); // context is already disposed
    }
}

#[derive(Debug)]
pub struct ThreadSafeModule<'ctx> {
    thread_safe_module: LLVMOrcThreadSafeModuleRef,
    module: Module<'ctx>,
    owned_by_lljit: RefCell<Option<LLJIT<'ctx>>>,
    _marker: PhantomData<&'ctx ThreadSafeContext>,
}

impl<'ctx> ThreadSafeModule<'ctx> {
    fn new(thread_safe_module: LLVMOrcThreadSafeModuleRef, module: Module<'ctx>) -> Self {
        assert!(!thread_safe_module.is_null());
        ThreadSafeModule {
            thread_safe_module,
            module,
            owned_by_lljit: RefCell::new(None),
            _marker: PhantomData,
        }
    }
}
impl<'ctx> Drop for ThreadSafeModule<'ctx> {
    fn drop(&mut self) {
        if self.owned_by_lljit.borrow().is_none() {
            unsafe {
                LLVMOrcDisposeThreadSafeModule(self.thread_safe_module);
            }
        }
        self.module.module.set(null_mut()); // module is already disposed
    }
}

#[derive(Debug, Clone)]
pub struct LLJIT<'ctx> {
    lljit: Rc<LLVMOrcLLJITRef>,
    _marker: PhantomData<&'ctx ThreadSafeContext>,
}

impl<'ctx> LLJIT<'ctx> {
    fn new(lljit: LLVMOrcLLJITRef) -> Self {
        assert!(!lljit.is_null());
        LLJIT {
            lljit: Rc::new(lljit),
            _marker: PhantomData,
        }
    }
    pub fn create(lljit_builder: Option<LLJITBuilder<'ctx>>) -> Result<Self, LLVMError> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| LLVMError::new_string_error(&e))?;
        let mut lljit: LLVMOrcLLJITRef = null_mut();
        let builder = match lljit_builder {
            Some(ref builder) => builder.builder,
            None => null_mut(),
        };
        let error;
        unsafe {
            error = LLVMOrcCreateLLJIT(&mut lljit, builder);
        }
        LLVMError::new(error)?;
        if let Some(mut builder) = lljit_builder {
            builder.builder = null_mut();
            drop(builder);
        }
        Ok(LLJIT::new(lljit))
    }
    pub fn get_main_jit_dylib(&self) -> JITDylib<'ctx> {
        unsafe { JITDylib::new(LLVMOrcLLJITGetMainJITDylib(*self.lljit), Some(self.clone())) }
    }

    pub fn add_module(
        &self,
        jit_dylib: &JITDylib<'ctx>,
        module: &ThreadSafeModule<'ctx>,
    ) -> Result<(), LLVMError> {
        let error;
        unsafe {
            error = LLVMOrcLLJITAddLLVMIRModule(
                *self.lljit,
                jit_dylib.jit_dylib,
                module.thread_safe_module,
            );
        }
        LLVMError::new(error)
    }

    pub fn add_module_with_rt(
        &self,
        rt: &ResourceTracker<'ctx>,
        module: &ThreadSafeModule<'ctx>,
    ) -> Result<(), LLVMError> {
        let error;
        unsafe {
            error =
                LLVMOrcLLJITAddLLVMIRModuleWithRT(*self.lljit, rt.rt, module.thread_safe_module);
        }
        if module.owned_by_lljit.borrow().is_some() {
            return Err(LLVMError::new_string_error(
                "module does already belong to an lljit instance",
            ));
        }
        *module.owned_by_lljit.borrow_mut() = Some(self.clone());
        LLVMError::new(error)
    }

    pub unsafe fn get_function_pointer<FnPtr>(&self, name: &str) -> Result<FnPtr, LLVMError> {
        let mut function_address = 0;
        let error = LLVMOrcLLJITLookup(*self.lljit, &mut function_address, to_c_str(name).as_ptr());
        LLVMError::new(error)?;
        Ok(transmute_copy(&function_address))
    }

    pub fn get_function<F>(&self, name: &str) -> Result<Function<'ctx, F>, LLVMError>
    where
        F: UnsafeFunctionPointer,
    {
        let func = unsafe { self.get_function_pointer(name) }?;
        Ok(Function {
            func,
            _owned_by_lljit: self.clone(),
        })
    }
}

impl<'ctx> Drop for LLJIT<'ctx> {
    fn drop(&mut self) {
        if Rc::strong_count(&self.lljit) == 1 {
            unsafe {
                LLVMOrcDisposeLLJIT(*self.lljit);
            }
        }
    }
}

#[derive(Debug)]
pub struct LLJITBuilder<'ctx> {
    builder: LLVMOrcLLJITBuilderRef,
    _marker: PhantomData<&'ctx ThreadSafeContext>,
}

impl<'ctx> Drop for LLJITBuilder<'ctx> {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeLLJITBuilder(self.builder);
        }
    }
}

#[derive(Debug)]
pub struct JITDylib<'ctx> {
    jit_dylib: LLVMOrcJITDylibRef,
    owned_by_lljit: Option<LLJIT<'ctx>>,
}

impl<'ctx> JITDylib<'ctx> {
    fn new(jit_dylib: LLVMOrcJITDylibRef, owned_by_lljit: Option<LLJIT<'ctx>>) -> Self {
        assert!(!jit_dylib.is_null());
        JITDylib {
            jit_dylib,
            owned_by_lljit,
        }
    }
    pub fn get_default_resource_tracker(&self) -> ResourceTracker<'ctx> {
        ResourceTracker::new(unsafe { LLVMOrcJITDylibGetDefaultResourceTracker(self.jit_dylib) })
    }
    pub fn create_resource_tracker(&self) -> ResourceTracker<'ctx> {
        ResourceTracker::new(unsafe { LLVMOrcJITDylibCreateResourceTracker(self.jit_dylib) })
    }
}

impl<'ctx> Drop for JITDylib<'ctx> {
    fn drop(&mut self) {
        if self.owned_by_lljit.is_none() {
            drop(self.get_default_resource_tracker());
        }
    }
}

#[derive(Debug)]
pub struct ResourceTracker<'ctx> {
    rt: LLVMOrcResourceTrackerRef,
    _marker: PhantomData<&'ctx ThreadSafeContext>,
}

impl<'ctx> ResourceTracker<'ctx> {
    fn new(rt: LLVMOrcResourceTrackerRef) -> Self {
        assert!(!rt.is_null());
        ResourceTracker {
            rt,
            _marker: PhantomData,
        }
    }
}

impl<'ctx> Drop for ResourceTracker<'ctx> {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcReleaseResourceTracker(self.rt);
        }
    }
}

#[derive(Debug)]
pub struct Function<'ctx, F> {
    func: F,
    _owned_by_lljit: LLJIT<'ctx>,
}

/// Marker trait representing an unsafe function pointer (`unsafe extern "C" fn(A, B, ...) -> Output`).
pub trait UnsafeFunctionPointer: private::SealedUnsafeFunctionPointer {}

mod private {
    /// A sealed trait which ensures nobody outside this crate can implement
    /// `UnsafeFunctionPointer`.
    ///
    /// See https://rust-lang-nursery.github.io/api-guidelines/future-proofing.html
    pub trait SealedUnsafeFunctionPointer: Copy {}
}

impl<F: private::SealedUnsafeFunctionPointer> UnsafeFunctionPointer for F {}

macro_rules! impl_unsafe_fn {
    (@recurse $first:ident $( , $rest:ident )*) => {
        impl_unsafe_fn!($( $rest ),*);
    };

    (@recurse) => {};

    ($( $param:ident ),*) => {
        impl<Output, $( $param ),*> private::SealedUnsafeFunctionPointer for unsafe extern "C" fn($( $param ),*) -> Output {}

        impl<'ctx, Output, $( $param ),*> Function<'ctx, unsafe extern "C" fn($( $param ),*) -> Output> {
            /// This method allows you to call the underlying function while making
            /// sure that the backing storage is not dropped too early and
            /// preserves the `unsafe` marker for any calls.
            #[allow(non_snake_case)]
            #[inline(always)]
            pub unsafe fn call(&self, $( $param: $param ),*) -> Output {
                (self.func)($( $param ),*)
            }
        }

        impl_unsafe_fn!(@recurse $( $param ),*);
    };
}

impl_unsafe_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
