use std::{
    cell::RefCell,
    error::Error,
    ffi::CStr,
    fmt::{self, Debug, Display, Formatter},
    mem::{transmute, transmute_copy},
    ops::Deref,
    ptr::null_mut,
    rc::Rc,
};

use libc::{c_char, c_void};
use llvm_sys::{
    error::{
        LLVMConsumeError, LLVMCreateStringError, LLVMDisposeErrorMessage, LLVMErrorRef,
        LLVMErrorTypeId, LLVMGetErrorMessage, LLVMGetErrorTypeId,
    },
    orc2::{
        lljit::{
            LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcDisposeLLJIT,
            LLVMOrcDisposeLLJITBuilder, LLVMOrcLLJITAddLLVMIRModule,
            LLVMOrcLLJITAddLLVMIRModuleWithRT, LLVMOrcLLJITAddObjectFile,
            LLVMOrcLLJITAddObjectFileWithRT, LLVMOrcLLJITBuilderRef,
            LLVMOrcLLJITBuilderSetJITTargetMachineBuilder,
            LLVMOrcLLJITBuilderSetObjectLinkingLayerCreator, LLVMOrcLLJITGetDataLayoutStr,
            LLVMOrcLLJITGetExecutionSession, LLVMOrcLLJITGetGlobalPrefix,
            LLVMOrcLLJITGetIRTransformLayer, LLVMOrcLLJITGetMainJITDylib,
            LLVMOrcLLJITGetObjLinkingLayer, LLVMOrcLLJITGetObjTransformLayer,
            LLVMOrcLLJITGetTripleString, LLVMOrcLLJITLookup, LLVMOrcLLJITMangleAndIntern,
            LLVMOrcLLJITRef,
        },
        LLVMOrcCreateNewThreadSafeContext, LLVMOrcCreateNewThreadSafeModule,
        LLVMOrcDisposeJITTargetMachineBuilder, LLVMOrcDisposeObjectLayer,
        LLVMOrcDisposeThreadSafeContext, LLVMOrcDisposeThreadSafeModule,
        LLVMOrcExecutionSessionCreateBareJITDylib, LLVMOrcExecutionSessionCreateJITDylib,
        LLVMOrcExecutionSessionGetJITDylibByName, LLVMOrcExecutionSessionIntern,
        LLVMOrcExecutionSessionRef, LLVMOrcIRTransformLayerRef, LLVMOrcJITDylibClear,
        LLVMOrcJITDylibCreateResourceTracker, LLVMOrcJITDylibGetDefaultResourceTracker,
        LLVMOrcJITDylibRef, LLVMOrcJITTargetMachineBuilderDetectHost,
        LLVMOrcJITTargetMachineBuilderGetTargetTriple, LLVMOrcJITTargetMachineBuilderRef,
        LLVMOrcJITTargetMachineBuilderSetTargetTriple, LLVMOrcObjectLayerAddObjectFile,
        LLVMOrcObjectLayerAddObjectFileWithRT, LLVMOrcObjectLayerRef,
        LLVMOrcObjectTransformLayerRef, LLVMOrcReleaseResourceTracker,
        LLVMOrcReleaseSymbolStringPoolEntry, LLVMOrcResourceTrackerRef,
        LLVMOrcResourceTrackerRemove, LLVMOrcResourceTrackerTransferTo,
        LLVMOrcRetainSymbolStringPoolEntry, LLVMOrcSymbolStringPoolEntryRef,
        LLVMOrcSymbolStringPoolEntryStr, LLVMOrcThreadSafeContextGetContext,
        LLVMOrcThreadSafeContextRef, LLVMOrcThreadSafeModuleRef,
    },
};

use crate::{
    context::Context,
    memory_buffer::MemoryBuffer,
    module::Module,
    support::{to_c_str, LLVMString},
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
        Err(LLVMError {
            error,
            handled: false,
        })
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
    unsafe fn new(thread_safe_context: LLVMOrcThreadSafeContextRef) -> Self {
        assert!(!thread_safe_context.is_null());
        let context = Context::new(LLVMOrcThreadSafeContextGetContext(thread_safe_context));
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
    unsafe fn new(thread_safe_module: LLVMOrcThreadSafeModuleRef, module: Module<'ctx>) -> Self {
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
    unsafe fn new(lljit: LLVMOrcLLJITRef) -> Self {
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
        unsafe {
            JITDylib::new(
                LLVMOrcLLJITGetMainJITDylib(*self.lljit),
                self.get_execution_session(),
            )
        }
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

    pub fn add_object_file(
        &self,
        jit_dylib: &JITDylib,
        object_buffer: MemoryBuffer,
    ) -> Result<(), LLVMError> {
        unsafe {
            LLVMError::new(LLVMOrcLLJITAddObjectFile(
                *self.lljit,
                jit_dylib.jit_dylib,
                object_buffer.memory_buffer,
            ))?;
            object_buffer.transfer_ownership_to_llvm();
        }
        Ok(())
    }

    pub fn add_object_file_with_rt(
        &self,
        rt: &ResourceTracker,
        object_buffer: MemoryBuffer,
    ) -> Result<(), LLVMError> {
        unsafe {
            LLVMError::new(LLVMOrcLLJITAddObjectFileWithRT(
                *self.lljit,
                rt.rt,
                object_buffer.memory_buffer,
            ))?;
            object_buffer.transfer_ownership_to_llvm();
        }
        Ok(())
    }

    /// Attempts to lookup a function by `name` in the execution engine.
    ///
    /// It is recommended to use [`get_function()`](LLJIT::get_function())
    /// instead of this method when intending to call the function
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

    pub fn get_execution_session(&self) -> ExecutionSession {
        unsafe {
            ExecutionSession::new_non_owning(
                LLVMOrcLLJITGetExecutionSession(*self.lljit),
                Some(self.clone()),
            )
        }
    }

    pub fn get_triple_string(&self) -> &CStr {
        unsafe { CStr::from_ptr(LLVMOrcLLJITGetTripleString(*self.lljit)) }
    }

    pub fn get_global_prefix(&self) -> i8 {
        unsafe { LLVMOrcLLJITGetGlobalPrefix(*self.lljit) }
    }

    pub fn mangle_and_intern(&self, unmangled_name: &str) -> SymbolStringPoolEntry {
        unsafe {
            SymbolStringPoolEntry::new(LLVMOrcLLJITMangleAndIntern(
                *self.lljit,
                to_c_str(unmangled_name).as_ptr(),
            ))
        }
    }

    pub fn get_object_linking_layer(&self) -> ObjectLayer {
        unsafe { ObjectLayer::new_non_owning(LLVMOrcLLJITGetObjLinkingLayer(*self.lljit)) }
    }

    pub fn get_object_transform_layer(&self) -> ObjectTransformLayer {
        unsafe {
            ObjectTransformLayer::new_non_owning(LLVMOrcLLJITGetObjTransformLayer(*self.lljit))
        }
    }

    pub fn get_ir_transform_layer(&self) -> IRTransformLayer {
        unsafe { IRTransformLayer::new_non_owning(LLVMOrcLLJITGetIRTransformLayer(*self.lljit)) }
    }

    pub fn get_data_layout_string(&self) -> &CStr {
        unsafe { CStr::from_ptr(LLVMOrcLLJITGetDataLayoutStr(*self.lljit)) }
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
pub struct LLJITBuilder {
    builder: LLVMOrcLLJITBuilderRef,
    object_linking_layer_creator: Option<Box<dyn ObjectLinkingLayerCreator>>,
}

impl LLJITBuilder {
    unsafe fn new(builder: LLVMOrcLLJITBuilderRef) -> Self {
        assert!(!builder.is_null());
        LLJITBuilder {
            builder,
            object_linking_layer_creator: None,
        }
    }

    /// Creates an `LLJITBuilder`.
    /// ```
    /// use inkwell::orc2::LLJITBuilder;
    ///
    /// let jit_builder = LLJITBuilder::create();
    /// ```
    pub fn create() -> Self {
        unsafe { LLJITBuilder::new(LLVMOrcCreateLLJITBuilder()) }
    }

    /// Builds the [`LLJIT`] instance. Consumes the `LLJITBuilder`.
    /// ```
    /// use inkwell::orc2::LLJITBuilder;
    ///
    /// let jit_builder = LLJITBuilder::create();
    /// let jit = jit_builder.build().expect("LLJITBuilder::build failed");
    pub fn build(self) -> Result<LLJIT, LLVMError> {
        unsafe {
            let lljit = LLJIT::create_with_builder(self.builder)?;
            self.transfer_ownership_to_llvm();
            Ok(lljit)
        }
    }

    unsafe fn transfer_ownership_to_llvm(mut self) {
        self.builder = null_mut();
    }

    pub fn set_jit_target_machine_builder(
        &self,
        jit_target_machine_builder: JITTargetMachineBuilder,
    ) {
        unsafe {
            LLVMOrcLLJITBuilderSetJITTargetMachineBuilder(
                self.builder,
                jit_target_machine_builder.builder,
            );
            jit_target_machine_builder.transfer_ownership_to_llvm();
        }
    }

    pub fn set_object_linking_layer_creator<'ctx>(
        &'ctx self,
        object_linking_layer_creator: &'ctx mut Box<dyn ObjectLinkingLayerCreator>,
    ) {
        unsafe {
            LLVMOrcLLJITBuilderSetObjectLinkingLayerCreator(
                self.builder,
                object_linking_layer_creator_function,
                transmute(object_linking_layer_creator),
            );
        }
    }

    pub fn set_object_linking_layer_creator_raw<'ctx, Ctx>(
        &'ctx self,
        object_linking_layer_creator_function: extern "C" fn(
            &Ctx,
            LLVMOrcExecutionSessionRef,
            *const c_char,
        ) -> LLVMOrcObjectLayerRef,
        ctx: &'ctx mut Ctx,
    ) {
        unsafe {
            LLVMOrcLLJITBuilderSetObjectLinkingLayerCreator(
                self.builder,
                transmute(object_linking_layer_creator_function),
                transmute(ctx as *mut Ctx),
            );
        }
    }
}

impl Debug for LLJITBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("LLJITBuilder")
            .field("builder", &self.builder)
            .field(
                "object_linking_layer_creator",
                &self.object_linking_layer_creator.as_ref().map(|_| ()),
            )
            .finish()
    }
}

impl Drop for LLJITBuilder {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeLLJITBuilder(self.builder);
        }
    }
}

#[no_mangle]
extern "C" fn object_linking_layer_creator_function(
    ctx: *mut c_void,
    execution_session: LLVMOrcExecutionSessionRef,
    triple: *const c_char,
) -> LLVMOrcObjectLayerRef {
    unsafe {
        let object_linking_layer_creator: &mut Box<dyn ObjectLinkingLayerCreator> = transmute(ctx);
        let object_layer = object_linking_layer_creator.create_object_linking_layer(
            ExecutionSession::new_non_owning(execution_session, None),
            CStr::from_ptr(triple),
        );
        let object_layer_ref = object_layer.object_layer;
        object_layer.transfer_ownership_to_llvm();
        object_layer_ref
    }
}

pub trait ObjectLinkingLayerCreator {
    fn create_object_linking_layer(
        &self,
        execution_session: ExecutionSession,
        triple: &CStr,
    ) -> ObjectLayer;
}
#[derive(Debug)]
pub struct JITTargetMachineBuilder {
    builder: LLVMOrcJITTargetMachineBuilderRef,
}

impl JITTargetMachineBuilder {
    unsafe fn new(builder: LLVMOrcJITTargetMachineBuilderRef) -> Self {
        assert!(!builder.is_null());
        JITTargetMachineBuilder { builder }
    }

    unsafe fn transfer_ownership_to_llvm(mut self) {
        self.builder = null_mut();
    }

    pub fn detect_host() -> Result<Self, LLVMError> {
        let mut builder = null_mut();
        unsafe {
            LLVMError::new(LLVMOrcJITTargetMachineBuilderDetectHost(&mut builder))?;
            Ok(JITTargetMachineBuilder::new(builder))
        }
    }

    pub fn create_from_target_machine() {
        todo!();
    }

    pub fn get_target_triple(&self) -> LLVMString {
        unsafe { LLVMString::new(LLVMOrcJITTargetMachineBuilderGetTargetTriple(self.builder)) }
    }

    pub fn set_target_triple(&self, target_triple: &str) {
        unsafe {
            LLVMOrcJITTargetMachineBuilderSetTargetTriple(
                self.builder,
                to_c_str(target_triple).as_ptr(),
            )
        }
    }
}

impl Drop for JITTargetMachineBuilder {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeJITTargetMachineBuilder(self.builder);
        }
    }
}

/// Represents a dynamic library in the execution engine.
#[derive(Debug)]
pub struct JITDylib {
    jit_dylib: LLVMOrcJITDylibRef,
    owned_by_es: ExecutionSession,
}

impl JITDylib {
    unsafe fn new(jit_dylib: LLVMOrcJITDylibRef, owned_by_es: ExecutionSession) -> Self {
        assert!(!jit_dylib.is_null());
        JITDylib {
            jit_dylib,
            owned_by_es,
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
        unsafe {
            ResourceTracker::new(
                LLVMOrcJITDylibGetDefaultResourceTracker(self.jit_dylib),
                self.owned_by_es.clone(),
                true,
            )
        }
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
        unsafe {
            ResourceTracker::new(
                LLVMOrcJITDylibCreateResourceTracker(self.jit_dylib),
                self.owned_by_es.clone(),
                false,
            )
        }
    }

    pub fn define() {
        todo!();
    }

    pub fn clear(&self) -> Result<(), LLVMError> {
        LLVMError::new(unsafe { LLVMOrcJITDylibClear(self.jit_dylib) })
    }

    pub fn add_generator() {
        todo!();
    }
}

/// A `ResourceTracker` is used to add/remove JIT resources.
/// Look at [`JITDylib`] for further information.
#[derive(Debug)]
pub struct ResourceTracker {
    rt: LLVMOrcResourceTrackerRef,
    _owned_by_es: ExecutionSession,
    default: bool,
}

impl ResourceTracker {
    unsafe fn new(
        rt: LLVMOrcResourceTrackerRef,
        owned_by_es: ExecutionSession,
        default: bool,
    ) -> Self {
        assert!(!rt.is_null());
        ResourceTracker {
            rt,
            _owned_by_es: owned_by_es,
            default,
        }
    }

    pub fn transfer_to(&self, destination: &ResourceTracker) {
        unsafe {
            LLVMOrcResourceTrackerTransferTo(self.rt, destination.rt);
        }
    }

    pub fn remove(&self) -> Result<(), LLVMError> {
        LLVMError::new(unsafe { LLVMOrcResourceTrackerRemove(self.rt) })
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

#[derive(Debug, Clone)]
pub struct ExecutionSession {
    execution_session: LLVMOrcExecutionSessionRef,
    _owned_by_lljit: Option<LLJIT>,
}

impl ExecutionSession {
    unsafe fn new_non_owning(
        execution_session: LLVMOrcExecutionSessionRef,
        owned_by_lljit: Option<LLJIT>,
    ) -> Self {
        assert!(!execution_session.is_null());
        ExecutionSession {
            execution_session,
            _owned_by_lljit: owned_by_lljit,
        }
    }

    pub fn set_error_reporter() {
        todo!();
    }

    pub fn get_symbol_string_pool() {
        todo!();
    }

    pub fn intern(&self, name: &str) -> SymbolStringPoolEntry {
        unsafe {
            SymbolStringPoolEntry::new(LLVMOrcExecutionSessionIntern(
                self.execution_session,
                to_c_str(name).as_ptr(),
            ))
        }
    }

    pub fn create_bare_jit_dylib(&self, name: &str) -> JITDylib {
        unsafe {
            JITDylib::new(
                LLVMOrcExecutionSessionCreateBareJITDylib(
                    self.execution_session,
                    to_c_str(name).as_ptr(),
                ),
                self.clone(),
            )
        }
    }

    pub fn create_jit_dylib(&self, name: &str) -> Result<JITDylib, LLVMError> {
        let mut jit_dylib = null_mut();
        unsafe {
            LLVMError::new(LLVMOrcExecutionSessionCreateJITDylib(
                self.execution_session,
                &mut jit_dylib,
                to_c_str(name).as_ptr(),
            ))?;
            Ok(JITDylib::new(jit_dylib, self.clone()))
        }
    }

    pub fn get_jit_dylib_by_name(&self, name: &str) -> Option<JITDylib> {
        let jit_dylib = unsafe {
            LLVMOrcExecutionSessionGetJITDylibByName(
                self.execution_session,
                to_c_str(name).as_ptr(),
            )
        };
        if jit_dylib.is_null() {
            None
        } else {
            Some(unsafe { JITDylib::new(jit_dylib, self.clone()) })
        }
    }
}

#[derive(Debug)]
pub struct ObjectLayer {
    object_layer: LLVMOrcObjectLayerRef,
}

impl ObjectLayer {
    unsafe fn new_non_owning(object_layer: LLVMOrcObjectLayerRef) -> Self {
        assert!(!object_layer.is_null());
        ObjectLayer { object_layer }
    }

    unsafe fn transfer_ownership_to_llvm(mut self) {
        self.object_layer = null_mut();
    }

    pub fn add_object_file(
        &self,
        jit_dylib: &JITDylib,
        object_buffer: MemoryBuffer,
    ) -> Result<(), LLVMError> {
        unsafe {
            LLVMError::new(LLVMOrcObjectLayerAddObjectFile(
                self.object_layer,
                jit_dylib.jit_dylib,
                object_buffer.memory_buffer,
            ))?;
            object_buffer.transfer_ownership_to_llvm()
        }
        Ok(())
    }
    pub fn add_object_file_with_rt(
        &self,
        rt: &ResourceTracker,
        object_buffer: MemoryBuffer,
    ) -> Result<(), LLVMError> {
        unsafe {
            LLVMError::new(LLVMOrcObjectLayerAddObjectFileWithRT(
                self.object_layer,
                rt.rt,
                object_buffer.memory_buffer,
            ))?;
            object_buffer.transfer_ownership_to_llvm()
        }
        Ok(())
    }

    pub fn emit() {
        todo!();
    }
}

impl Drop for ObjectLayer {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeObjectLayer(self.object_layer);
        }
    }
}

#[derive(Debug)]
pub struct ObjectTransformLayer {
    object_transform_layer: LLVMOrcObjectTransformLayerRef,
}

impl ObjectTransformLayer {
    unsafe fn new_non_owning(object_transform_layer: LLVMOrcObjectTransformLayerRef) -> Self {
        assert!(!object_transform_layer.is_null());
        ObjectTransformLayer {
            object_transform_layer,
        }
    }
    pub fn set_transform() {
        todo!();
    }
}

#[derive(Debug)]
pub struct IRTransformLayer {
    ir_transform_layer: LLVMOrcIRTransformLayerRef,
}

impl IRTransformLayer {
    unsafe fn new_non_owning(ir_transform_layer: LLVMOrcIRTransformLayerRef) -> Self {
        assert!(!ir_transform_layer.is_null());
        IRTransformLayer { ir_transform_layer }
    }

    pub fn emit() {
        todo!();
    }

    pub fn set_transform() {
        todo!();
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SymbolStringPoolEntry {
    entry: LLVMOrcSymbolStringPoolEntryRef,
}

impl SymbolStringPoolEntry {
    unsafe fn new(entry: LLVMOrcSymbolStringPoolEntryRef) -> Self {
        assert!(!entry.is_null());
        SymbolStringPoolEntry { entry }
    }
    pub fn get_string(&self) -> &CStr {
        unsafe { CStr::from_ptr(LLVMOrcSymbolStringPoolEntryStr(self.entry)) }
    }
}

impl Clone for SymbolStringPoolEntry {
    fn clone(&self) -> Self {
        unsafe {
            LLVMOrcRetainSymbolStringPoolEntry(self.entry);
        }
        Self { entry: self.entry }
    }
}

impl Drop for SymbolStringPoolEntry {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcReleaseSymbolStringPoolEntry(self.entry);
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
