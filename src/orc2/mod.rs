pub mod lljit;

use std::{
    cell::RefCell,
    ffi::CStr,
    fmt::{self, Debug, Formatter},
    mem::{transmute, transmute_copy},
    ptr::null_mut,
    rc::Rc,
};

use libc::{c_char, c_void};
use llvm_sys::orc2::{
    LLVMOrcCreateNewThreadSafeContext, LLVMOrcCreateNewThreadSafeModule,
    LLVMOrcDisposeJITTargetMachineBuilder, LLVMOrcDisposeThreadSafeContext,
    LLVMOrcDisposeThreadSafeModule, LLVMOrcExecutionSessionIntern, LLVMOrcExecutionSessionRef,
    LLVMOrcJITDylibRef, LLVMOrcJITTargetMachineBuilderDetectHost,
    LLVMOrcJITTargetMachineBuilderRef, LLVMOrcReleaseSymbolStringPoolEntry,
    LLVMOrcSymbolStringPoolEntryRef, LLVMOrcThreadSafeContextGetContext,
    LLVMOrcThreadSafeContextRef, LLVMOrcThreadSafeModuleRef,
};
#[llvm_versions(12.0..=latest)]
use llvm_sys::orc2::{
    LLVMOrcDisposeObjectLayer, LLVMOrcExecutionSessionCreateBareJITDylib,
    LLVMOrcExecutionSessionCreateJITDylib, LLVMOrcExecutionSessionGetJITDylibByName,
    LLVMOrcJITDylibClear, LLVMOrcJITDylibCreateResourceTracker,
    LLVMOrcJITDylibGetDefaultResourceTracker, LLVMOrcObjectLayerRef, LLVMOrcReleaseResourceTracker,
    LLVMOrcResourceTrackerRef, LLVMOrcResourceTrackerRemove, LLVMOrcResourceTrackerTransferTo,
    LLVMOrcRetainSymbolStringPoolEntry, LLVMOrcSymbolStringPoolEntryStr,
};
#[llvm_versions(13.0..=latest)]
use llvm_sys::orc2::{
    LLVMOrcIRTransformLayerRef, LLVMOrcJITTargetMachineBuilderGetTargetTriple,
    LLVMOrcJITTargetMachineBuilderSetTargetTriple, LLVMOrcObjectLayerAddObjectFile,
    LLVMOrcObjectLayerAddObjectFileWithRT, LLVMOrcObjectTransformLayerRef,
};

use crate::{
    context::Context,
    error::LLVMError,
    memory_buffer::MemoryBuffer,
    module::Module,
    support::{to_c_str, LLVMString},
    targets::{InitializationConfig, Target},
};

use self::lljit::LLJIT;

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

    #[llvm_versions(13.0..=latest)]
    pub fn get_target_triple(&self) -> LLVMString {
        unsafe { LLVMString::new(LLVMOrcJITTargetMachineBuilderGetTargetTriple(self.builder)) }
    }

    #[llvm_versions(13.0..=latest)]
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
    /// use inkwell::orc2::lljit::LLJIT;
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let main_jd = jit.get_main_jit_dylib();
    /// let rt = main_jd.get_default_resource_tracker();
    /// ```
    #[llvm_versions(12.0..=latest)]
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
    /// use inkwell::orc2::lljit::LLJIT;
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let main_jd = jit.get_main_jit_dylib();
    /// let rt = main_jd.create_resource_tracker();
    /// ```
    #[llvm_versions(12.0..=latest)]
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

    #[llvm_versions(12.0..=latest)]
    pub fn clear(&self) -> Result<(), LLVMError> {
        LLVMError::new(unsafe { LLVMOrcJITDylibClear(self.jit_dylib) })
    }

    pub fn add_generator() {
        todo!();
    }
}

/// A `ResourceTracker` is used to add/remove JIT resources.
/// Look at [`JITDylib`] for further information.
#[llvm_versions(12.0..=latest)]
#[derive(Debug)]
pub struct ResourceTracker {
    rt: LLVMOrcResourceTrackerRef,
    _owned_by_es: ExecutionSession,
    default: bool,
}

#[llvm_versions(12.0..=latest)]
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

#[llvm_versions(12.0..=latest)]
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

    #[llvm_versions(12.0..=latest)]
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

    #[llvm_versions(12.0..=latest)]
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

    #[llvm_versions(12.0..=latest)]
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

#[llvm_versions(12.0..=latest)]
#[derive(Debug)]
pub struct ObjectLayer {
    object_layer: LLVMOrcObjectLayerRef,
}

#[llvm_versions(12.0..=latest)]
impl ObjectLayer {
    unsafe fn new_non_owning(object_layer: LLVMOrcObjectLayerRef) -> Self {
        assert!(!object_layer.is_null());
        ObjectLayer { object_layer }
    }

    unsafe fn transfer_ownership_to_llvm(mut self) {
        self.object_layer = null_mut();
    }

    #[llvm_versions(13.0..=latest)]
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

    #[llvm_versions(13.0..=latest)]
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

#[llvm_versions(12.0..=latest)]
impl Drop for ObjectLayer {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeObjectLayer(self.object_layer);
        }
    }
}

#[llvm_versions(13.0..=latest)]
#[derive(Debug)]
pub struct ObjectTransformLayer {
    object_transform_layer: LLVMOrcObjectTransformLayerRef,
}

#[llvm_versions(13.0..=latest)]
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

#[llvm_versions(13.0..=latest)]
#[derive(Debug)]
pub struct IRTransformLayer {
    ir_transform_layer: LLVMOrcIRTransformLayerRef,
}

#[llvm_versions(13.0..=latest)]
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

    #[llvm_versions(12.0..=latest)]
    pub fn get_string(&self) -> &CStr {
        unsafe { CStr::from_ptr(LLVMOrcSymbolStringPoolEntryStr(self.entry)) }
    }
}

#[llvm_versions(12.0..=latest)]
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
