use std::{
    cell::RefCell,
    error::Error,
    ffi::CStr,
    fmt::{self, Debug, Display, Formatter},
    mem::transmute_copy,
    ops::Deref,
    ptr::null_mut,
    rc::Rc,
};

use libc::c_char;
use llvm_sys::{
    error::{
        LLVMConsumeError, LLVMCreateStringError, LLVMDisposeErrorMessage, LLVMErrorRef,
        LLVMErrorTypeId, LLVMGetErrorMessage, LLVMGetErrorTypeId,
    },
    orc2::{
        lljit::{
            LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcDisposeLLJIT,
            LLVMOrcDisposeLLJITBuilder, LLVMOrcLLJITAddLLVMIRModule,
            LLVMOrcLLJITAddLLVMIRModuleWithRT, LLVMOrcLLJITBuilderRef, LLVMOrcLLJITGetMainJITDylib,
            LLVMOrcLLJITLookup, LLVMOrcLLJITRef,
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
    support::to_c_str,
    targets::{InitializationConfig, Target},
};

/// An LLVM Error.
#[derive(Debug)]
pub struct LLVMError {
    error: LLVMErrorRef,
    handled: bool,
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
            Err(LLVMError {
                error,
                handled: false,
            })
        }
    }
    // Null type id == success
    pub fn get_type_id(&self) -> LLVMErrorTypeId {
        // FIXME: Don't expose LLVMErrorTypeId
        unsafe { LLVMGetErrorTypeId(self.error) }
    }
    /// Returns the error message of the error. This consumes the error
    /// and makes the error unusable afterwards.
    /// ```
    /// use std::ffi::{CString, CStr};
    /// use inkwell::orc2::LLVMError;
    ///
    /// let error = LLVMError::new_string_error("llvm error");
    /// assert_eq!(*error.get_message(), *CString::new("llvm error").unwrap().as_c_str());
    /// ```
    pub fn get_message(mut self) -> LLVMErrorMessage {
        self.handled = true;
        unsafe { LLVMErrorMessage::new(LLVMGetErrorMessage(self.error)) }
    }
    /// Creates a new StringError with the given message.
    /// ```
    /// use inkwell::orc2::LLVMError;
    ///
    /// let error = LLVMError::new_string_error("string error");
    /// ```
    pub fn new_string_error(message: &str) -> Self {
        let error = unsafe { LLVMCreateStringError(to_c_str(message).as_ptr()) };
        LLVMError {
            error,
            handled: false,
        }
    }
}

impl Drop for LLVMError {
    fn drop(&mut self) {
        if !self.handled {
            unsafe { LLVMConsumeError(self.error) }
        }
    }
}

/// An owned LLVM Error Message.
#[derive(Eq)]
pub struct LLVMErrorMessage {
    pub(crate) ptr: *const c_char,
}

impl LLVMErrorMessage {
    pub(crate) unsafe fn new(ptr: *const c_char) -> Self {
        LLVMErrorMessage { ptr }
    }

    /// This is a convenience method for creating a Rust `String`,
    /// however; it *will* reallocate. `LLVMErrorMessage` should be used
    /// as much as possible to save memory since it is allocated by
    /// LLVM. It's essentially a `CString` with a custom LLVM
    /// deallocator
    /// ```
    /// use inkwell::orc2::{LLVMError, LLVMErrorMessage};
    ///
    /// let error = LLVMError::new_string_error("error");
    /// let error_msg = error.get_message().to_string();
    /// assert_eq!(error_msg, "error");
    /// ```
    pub fn to_string(&self) -> String {
        (*self).to_string_lossy().into_owned()
    }
}

impl Deref for LLVMErrorMessage {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        unsafe { CStr::from_ptr(self.ptr) }
    }
}

impl Debug for LLVMErrorMessage {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.deref())
    }
}

impl Display for LLVMErrorMessage {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.deref())
    }
}

impl PartialEq for LLVMErrorMessage {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Error for LLVMErrorMessage {
    fn description(&self) -> &str {
        self.to_str()
            .expect("Could not convert LLVMErrorMessage to str (likely invalid unicode)")
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl Drop for LLVMErrorMessage {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeErrorMessage(self.ptr as *mut _);
        }
    }
}

/// The thread safe variant of [`Context`] used in this module.
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

    /// Creates a new `ThreadSafeContext`.
    /// ```
    /// use inkwell::orc2::ThreadSafeContext;
    ///
    /// let thread_safe_context = ThreadSafeContext::create();
    /// ```
    pub fn create() -> Self {
        unsafe { ThreadSafeContext::new(LLVMOrcCreateNewThreadSafeContext()) }
    }

    /// Gets the [`Context`] from this instance. The returned [`Context`]
    /// can be used for most operations concerning the context.
    /// ```
    /// use inkwell::orc2::ThreadSafeContext;
    ///
    /// let thread_safe_context = ThreadSafeContext::create();
    /// let context = thread_safe_context.context();
    /// ```
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Creates a [`ThreadSafeModule`] from a [`Module`]. The [`Module`] is now
    /// owned by the [`ThreadSafeModule`]
    /// ```
    /// use inkwell::orc2::ThreadSafeContext;
    ///
    /// let thread_safe_context = ThreadSafeContext::create();
    /// let context = thread_safe_context.context();
    /// let module = context.create_module("main");
    /// let thread_safe_module = thread_safe_context.create_module(module);
    /// ```
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
        unsafe {
            LLVMOrcDisposeThreadSafeContext(self.thread_safe_context);
        }
        self.context.context = null_mut(); // context is already disposed
    }
}

/// The thread safe variant of [`Module`] used in this module. The underlying
/// llvm module will be disposed when this object is dropped, except when
/// it is owned by an execution engine.
#[derive(Debug)]
pub struct ThreadSafeModule<'ctx> {
    thread_safe_module: LLVMOrcThreadSafeModuleRef,
    module: Module<'ctx>,
    owned_by_lljit: RefCell<Option<LLJIT>>,
}

impl<'ctx> ThreadSafeModule<'ctx> {
    fn new(thread_safe_module: LLVMOrcThreadSafeModuleRef, module: Module<'ctx>) -> Self {
        assert!(!thread_safe_module.is_null());
        ThreadSafeModule {
            thread_safe_module,
            module,
            owned_by_lljit: RefCell::new(None),
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

/// Represents a reference to an LLVM `LLJIT` execution engine.
#[derive(Debug, Clone)]
pub struct LLJIT {
    lljit: Rc<LLVMOrcLLJITRef>,
}

impl LLJIT {
    fn new(lljit: LLVMOrcLLJITRef) -> Self {
        assert!(!lljit.is_null());
        LLJIT {
            lljit: Rc::new(lljit),
        }
    }

    /// Creates a `LLJIT` instance with the provided `lljit_builder`.
    /// If `lljit_builder` is `None` the default LLJITBuilder will be
    /// used.
    /// ```
    /// use inkwell::orc2::LLJIT;
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// ```
    pub fn create() -> Result<Self, LLVMError> {
        unsafe { LLJIT::create_with_builder(null_mut()) }
    }

    unsafe fn create_with_builder(builder: LLVMOrcLLJITBuilderRef) -> Result<Self, LLVMError> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| LLVMError::new_string_error(&e))?;
        let mut lljit: LLVMOrcLLJITRef = null_mut();
        let error = LLVMOrcCreateLLJIT(&mut lljit, builder);
        LLVMError::new(error)?;
        Ok(LLJIT::new(lljit))
    }

    /// Returns the main [`JITDylib`].
    /// ```
    /// use inkwell::orc2::LLJIT;
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let main_jd = jit.get_main_jit_dylib();
    /// ```
    pub fn get_main_jit_dylib(&self) -> JITDylib {
        unsafe { JITDylib::new(LLVMOrcLLJITGetMainJITDylib(*self.lljit), Some(self.clone())) }
    }

    /// Adds `module` to `jit_dylib` in the execution engine.
    /// ```
    /// use inkwell::orc2::{LLJIT, ThreadSafeContext};
    ///
    /// let thread_safe_context = ThreadSafeContext::create();
    /// let context = thread_safe_context.context();
    /// let module = context.create_module("main");
    /// let thread_safe_module = thread_safe_context.create_module(module);
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let main_jd = jit.get_main_jit_dylib();
    /// jit.add_module(&main_jd, thread_safe_module)
    ///     .expect("LLJIT::add_module failed");
    /// ```
    pub fn add_module<'ctx>(
        &self,
        jit_dylib: &JITDylib,
        module: ThreadSafeModule<'ctx>,
    ) -> Result<(), LLVMError> {
        let error = unsafe {
            LLVMOrcLLJITAddLLVMIRModule(*self.lljit, jit_dylib.jit_dylib, module.thread_safe_module)
        };
        if module.owned_by_lljit.borrow().is_some() {
            return Err(LLVMError::new_string_error(
                "module does already belong to an lljit instance",
            ));
        }
        *module.owned_by_lljit.borrow_mut() = Some(self.clone());
        LLVMError::new(error)
    }

    /// Adds `module` to `rt`'s JITDylib in the execution engine.
    /// ```
    /// use inkwell::orc2::{LLJIT, ThreadSafeContext};
    ///
    /// let thread_safe_context = ThreadSafeContext::create();
    /// let context = thread_safe_context.context();
    /// let module = context.create_module("main");
    /// let thread_safe_module = thread_safe_context.create_module(module);
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let main_jd = jit.get_main_jit_dylib();
    /// let module_rt = main_jd.create_resource_tracker();
    /// jit.add_module_with_rt(&module_rt, thread_safe_module)
    ///     .expect("LLJIT::add_module_with_rt failed");
    /// ```
    pub fn add_module_with_rt<'ctx>(
        &self,
        rt: &ResourceTracker,
        module: ThreadSafeModule<'ctx>,
    ) -> Result<(), LLVMError> {
        let error = unsafe {
            LLVMOrcLLJITAddLLVMIRModuleWithRT(*self.lljit, rt.rt, module.thread_safe_module)
        };
        if module.owned_by_lljit.borrow().is_some() {
            return Err(LLVMError::new_string_error(
                "module does already belong to an lljit instance",
            ));
        }
        *module.owned_by_lljit.borrow_mut() = Some(self.clone());
        LLVMError::new(error)
    }

    /// Attempts to lookup a function by `name` in the execution engine.
    ///
    /// It is recommended to use [`get_function()`](LLJIT::get_function()) instead of this method when intending to call the function
    /// pointer so that you don't have to do error-prone transmutes yourself.
    pub fn get_function_address(&self, name: &str) -> Result<u64, LLVMError> {
        let mut function_address = 0;
        let error = unsafe {
            LLVMOrcLLJITLookup(*self.lljit, &mut function_address, to_c_str(name).as_ptr())
        };
        LLVMError::new(error)?;
        Ok(function_address)
    }

    /// Attempts to lookup a function by `name` in the execution engine.
    ///
    /// The [`UnsafeFunctionPointer`] trait is designed so only `unsafe extern
    /// "C"` functions can be retrieved via the `get_function()` method. If you
    /// get funny type errors then it's probably because you have specified the
    /// wrong calling convention or forgotten to specify the retrieved function
    /// as `unsafe`.
    ///
    /// # Example
    /// ```
    /// use inkwell::orc2::{ThreadSafeContext, LLJIT};
    ///
    /// let thread_safe_context = ThreadSafeContext::create();
    /// let context = thread_safe_context.context();
    /// let module = context.create_module("main");
    /// let builder = context.create_builder();
    ///
    /// // Set up the function signature
    /// let double = context.f64_type();
    /// let func_sig = double.fn_type(&[], false);
    ///
    /// // Add the function to our module
    /// let func = module.add_function("test_fn", func_sig, None);
    /// let entry_bb = context.append_basic_block(func, "entry");
    /// builder.position_at_end(entry_bb);
    ///
    /// // Insert a return statement
    /// let ret = double.const_float(64.0);
    /// builder.build_return(Some(&ret));
    ///
    /// // Create the LLJIT engine and add the module
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let thread_safe_module = thread_safe_context.create_module(module);
    /// let main_jd = jit.get_main_jit_dylib();
    /// jit.add_module(&main_jd, thread_safe_module)
    ///     .expect("LLJIT::add_module_with_rt failed");
    ///
    /// // lookup the function and execute it
    /// unsafe {
    ///     let test_fn = jit.get_function::<unsafe extern "C" fn() -> f64>("test_fn")
    ///         .expect("LLJIT::get_function failed");
    ///     let return_value = test_fn.call();
    ///     assert_eq!(return_value, 64.0);
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// It is the caller's responsibility to ensure they call the function with
    /// the correct signature and calling convention.
    ///
    /// The [`Function`] wrapper ensures a function won't accidentally outlive the
    /// execution engine it came from, but adding functions or removing resource trackers
    /// after calling this method *may* invalidate the function pointer.
    pub unsafe fn get_function<F>(&self, name: &str) -> Result<Function<F>, LLVMError>
    where
        F: UnsafeFunctionPointer,
    {
        Ok(Function {
            func: transmute_copy(&self.get_function_address(name)?),
            _owned_by_lljit: self.clone(),
        })
    }
}

impl Drop for LLJIT {
    fn drop(&mut self) {
        if Rc::strong_count(&self.lljit) == 1 {
            unsafe {
                LLVMOrcDisposeLLJIT(*self.lljit);
            }
        }
    }
}

/// An `LLJITBuilder` is used to create custom [`LLJIT`] instances.
#[derive(Debug)]
pub struct LLJITBuilder {
    builder: LLVMOrcLLJITBuilderRef,
}

impl LLJITBuilder {
    fn new(builder: LLVMOrcLLJITBuilderRef) -> Self {
        assert!(!builder.is_null());
        LLJITBuilder { builder }
    }

    /// Creates an `LLJITBuilder`.
    /// ```
    /// use inkwell::orc2::LLJITBuilder;
    ///
    /// let jit_builder = LLJITBuilder::create();
    /// ```
    pub fn create() -> Self {
        LLJITBuilder::new(unsafe { LLVMOrcCreateLLJITBuilder() })
    }

    /// Builds the [`LLJIT`] instance. Consumes the `LLJITBuilder`.
    /// ```
    /// use inkwell::orc2::LLJITBuilder;
    ///
    /// let jit_builder = LLJITBuilder::create();
    /// let jit = jit_builder.build().expect("LLJITBuilder::build failed");
    pub fn build(mut self) -> Result<LLJIT, LLVMError> {
        let lljit = unsafe { LLJIT::create_with_builder(self.builder) }?;
        self.builder = null_mut();
        Ok(lljit)
    }
}
impl Drop for LLJITBuilder {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeLLJITBuilder(self.builder);
        }
    }
}

/// Represents a dynamic library in the execution engine.
#[derive(Debug)]
pub struct JITDylib {
    jit_dylib: LLVMOrcJITDylibRef,
    owned_by_lljit: Option<LLJIT>,
}

impl JITDylib {
    fn new(jit_dylib: LLVMOrcJITDylibRef, owned_by_lljit: Option<LLJIT>) -> Self {
        assert!(!jit_dylib.is_null());
        JITDylib {
            jit_dylib,
            owned_by_lljit,
        }
    }

    /// Returns the default [`ResourceTracker`].
    /// ```
    /// use inkwell::orc2::LLJIT;
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let main_jd = jit.get_main_jit_dylib();
    /// let rt = main_jd.get_default_resource_tracker();
    /// ```
    pub fn get_default_resource_tracker(&self) -> ResourceTracker {
        ResourceTracker::new(
            unsafe { LLVMOrcJITDylibGetDefaultResourceTracker(self.jit_dylib) },
            self.owned_by_lljit.clone(),
            true,
        )
    }

    /// Creates a new [`ResourceTracker`].
    /// ```
    /// use inkwell::orc2::LLJIT;
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let main_jd = jit.get_main_jit_dylib();
    /// let rt = main_jd.create_resource_tracker();
    /// ```
    pub fn create_resource_tracker(&self) -> ResourceTracker {
        ResourceTracker::new(
            unsafe { LLVMOrcJITDylibCreateResourceTracker(self.jit_dylib) },
            self.owned_by_lljit.clone(),
            false,
        )
    }
}

/// A `ResourceTracker` is used to add/remove JIT resources.
/// Look at [`JITDylib`] for further information.
#[derive(Debug)]
pub struct ResourceTracker {
    rt: LLVMOrcResourceTrackerRef,
    _owned_by_lljit: Option<LLJIT>,
    default: bool,
}

impl ResourceTracker {
    fn new(rt: LLVMOrcResourceTrackerRef, owned_by_lljit: Option<LLJIT>, default: bool) -> Self {
        assert!(!rt.is_null());
        ResourceTracker {
            rt,
            _owned_by_lljit: owned_by_lljit,
            default,
        }
    }
}

impl Drop for ResourceTracker {
    fn drop(&mut self) {
        if !self.default {
            unsafe {
                LLVMOrcReleaseResourceTracker(self.rt);
            }
        }
    }
}

/// A wrapper around a function pointer which ensures the function being pointed
/// to doesn't accidentally outlive its execution engine.
#[derive(Debug)]
pub struct Function<F> {
    func: F,
    _owned_by_lljit: LLJIT,
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

        impl<'ctx, Output, $( $param ),*> Function<unsafe extern "C" fn($( $param ),*) -> Output> {
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
