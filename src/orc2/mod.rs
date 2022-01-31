pub mod lljit;

use std::{
    ffi::CStr,
    fmt::Debug,
    marker::PhantomData,
    mem::{forget, transmute},
    ptr,
};

use libc::c_void;
#[llvm_versions(12.0..=latest)]
use llvm_sys::orc2::{
    ee::LLVMOrcCreateRTDyldObjectLinkingLayerWithSectionMemoryManager, LLVMOrcDisposeObjectLayer,
    LLVMOrcExecutionSessionCreateBareJITDylib, LLVMOrcExecutionSessionCreateJITDylib,
    LLVMOrcExecutionSessionGetJITDylibByName, LLVMOrcJITDylibClear,
    LLVMOrcJITDylibCreateResourceTracker, LLVMOrcJITDylibGetDefaultResourceTracker,
    LLVMOrcObjectLayerRef, LLVMOrcReleaseResourceTracker, LLVMOrcResourceTrackerRef,
    LLVMOrcResourceTrackerRemove, LLVMOrcResourceTrackerTransferTo,
    LLVMOrcRetainSymbolStringPoolEntry, LLVMOrcSymbolStringPoolEntryStr,
};
use llvm_sys::orc2::{
    LLVMOrcCreateNewThreadSafeContext, LLVMOrcCreateNewThreadSafeModule,
    LLVMOrcDisposeJITTargetMachineBuilder, LLVMOrcDisposeThreadSafeContext,
    LLVMOrcDisposeThreadSafeModule, LLVMOrcExecutionSessionIntern, LLVMOrcExecutionSessionRef,
    LLVMOrcJITDylibRef, LLVMOrcJITTargetMachineBuilderCreateFromTargetMachine,
    LLVMOrcJITTargetMachineBuilderDetectHost, LLVMOrcJITTargetMachineBuilderRef,
    LLVMOrcReleaseSymbolStringPoolEntry, LLVMOrcSymbolStringPoolEntryRef,
    LLVMOrcThreadSafeContextGetContext, LLVMOrcThreadSafeContextRef, LLVMOrcThreadSafeModuleRef,
};
#[llvm_versions(13.0..=latest)]
use llvm_sys::{
    error::LLVMErrorRef,
    orc2::{
        LLVMOrcIRTransformLayerRef, LLVMOrcJITTargetMachineBuilderGetTargetTriple,
        LLVMOrcJITTargetMachineBuilderSetTargetTriple, LLVMOrcObjectLayerAddObjectFile,
        LLVMOrcObjectTransformLayerRef, LLVMOrcObjectTransformLayerSetTransform,
    },
    prelude::LLVMMemoryBufferRef,
};
// #[llvm_versions(14.0..=latest)]
// use llvm_sys::orc2::LLVMOrcObjectLayerAddObjectFileWithRT;

use crate::{
    context::Context,
    error::LLVMError,
    memory_buffer::{MemoryBuffer, MemoryBufferRef},
    module::Module,
    support::{to_c_str, LLVMString, OwnedOrBorrowedPtr},
    targets::TargetMachine,
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
        self.context.context = ptr::null_mut(); // context is already disposed
    }
}

/// The thread safe variant of [`Module`] used in this module. The underlying
/// llvm module will be disposed when this object is dropped, except when
/// it is owned by an execution engine.
#[derive(Debug)]
pub struct ThreadSafeModule<'ctx> {
    thread_safe_module: LLVMOrcThreadSafeModuleRef,
    module: Module<'ctx>,
}

impl<'ctx> ThreadSafeModule<'ctx> {
    unsafe fn new(thread_safe_module: LLVMOrcThreadSafeModuleRef, module: Module<'ctx>) -> Self {
        assert!(!thread_safe_module.is_null());
        ThreadSafeModule {
            thread_safe_module,
            module,
        }
    }
}
impl<'ctx> Drop for ThreadSafeModule<'ctx> {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeThreadSafeModule(self.thread_safe_module);
        }
        self.module.module.set(ptr::null_mut()); // module is already disposed
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

    pub fn detect_host() -> Result<Self, LLVMError> {
        let mut builder = ptr::null_mut();
        unsafe {
            LLVMError::new(LLVMOrcJITTargetMachineBuilderDetectHost(&mut builder))?;
            Ok(JITTargetMachineBuilder::new(builder))
        }
    }

    #[llvm_versions(12.0..=latest)]
    pub fn create_from_target_machine(target_machine: TargetMachine) -> Self {
        let jit_target_machine_builder = unsafe {
            JITTargetMachineBuilder::new(LLVMOrcJITTargetMachineBuilderCreateFromTargetMachine(
                target_machine.target_machine,
            ))
        };
        forget(target_machine);
        jit_target_machine_builder
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
pub struct JITDylib<'jit> {
    jit_dylib: LLVMOrcJITDylibRef,
    _marker: PhantomData<&'jit ()>,
}

impl<'jit> JITDylib<'jit> {
    unsafe fn new(jit_dylib: LLVMOrcJITDylibRef) -> Self {
        assert!(!jit_dylib.is_null());
        JITDylib {
            jit_dylib,
            _marker: PhantomData,
        }
    }

    /// Returns the default [`ResourceTracker`].
    /// ```
    /// use inkwell::orc2::lljit::LLJIT;
    ///
    /// let lljit = LLJIT::create().expect("LLJIT::create failed");
    /// let main_jd = lljit.get_main_jit_dylib();
    /// let rt = main_jd.get_default_resource_tracker();
    /// ```
    #[llvm_versions(12.0..=latest)]
    pub fn get_default_resource_tracker(&self) -> ResourceTracker {
        unsafe {
            ResourceTracker::new(
                LLVMOrcJITDylibGetDefaultResourceTracker(self.jit_dylib),
                true,
            )
        }
    }

    /// Creates a new [`ResourceTracker`].
    /// ```
    /// use inkwell::orc2::lljit::LLJIT;
    ///
    /// let lljit = LLJIT::create().expect("LLJIT::create failed");
    /// let main_jd = lljit.get_main_jit_dylib();
    /// let rt = main_jd.create_resource_tracker();
    /// ```
    #[llvm_versions(12.0..=latest)]
    pub fn create_resource_tracker(&self) -> ResourceTracker<'jit> {
        unsafe { ResourceTracker::new(LLVMOrcJITDylibCreateResourceTracker(self.jit_dylib), false) }
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
pub struct ResourceTracker<'jit> {
    rt: LLVMOrcResourceTrackerRef,
    default: bool,
    _marker: PhantomData<&'jit ()>,
}

#[llvm_versions(12.0..=latest)]
impl<'jit> ResourceTracker<'jit> {
    unsafe fn new(rt: LLVMOrcResourceTrackerRef, default: bool) -> Self {
        assert!(!rt.is_null());
        ResourceTracker {
            rt,
            default,
            _marker: PhantomData,
        }
    }

    pub fn transfer_to(&self, destination: &ResourceTracker) {
        unsafe {
            LLVMOrcResourceTrackerTransferTo(self.rt, destination.rt);
        }
    }

    pub fn remove(self) -> Result<(), LLVMError> {
        LLVMError::new(unsafe { LLVMOrcResourceTrackerRemove(self.rt) })
    }
}

#[llvm_versions(12.0..=latest)]
impl Drop for ResourceTracker<'_> {
    fn drop(&mut self) {
        if !self.default && !self.rt.is_null() {
            unsafe {
                LLVMOrcReleaseResourceTracker(self.rt);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionSession<'jit> {
    execution_session: LLVMOrcExecutionSessionRef,
    _marker: PhantomData<&'jit ()>,
}

impl<'jit> ExecutionSession<'jit> {
    unsafe fn new_borrowed(execution_session: LLVMOrcExecutionSessionRef) -> Self {
        assert!(!execution_session.is_null());
        ExecutionSession {
            execution_session,
            _marker: PhantomData,
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
            JITDylib::new(LLVMOrcExecutionSessionCreateBareJITDylib(
                self.execution_session,
                to_c_str(name).as_ptr(),
            ))
        }
    }

    #[llvm_versions(12.0..=latest)]
    pub fn create_jit_dylib(&self, name: &str) -> Result<JITDylib, LLVMError> {
        let mut jit_dylib = ptr::null_mut();
        unsafe {
            LLVMError::new(LLVMOrcExecutionSessionCreateJITDylib(
                self.execution_session,
                &mut jit_dylib,
                to_c_str(name).as_ptr(),
            ))?;
            Ok(JITDylib::new(jit_dylib))
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
            Some(unsafe { JITDylib::new(jit_dylib) })
        }
    }

    #[llvm_versions(12.0..=latest)]
    pub fn create_rt_dyld_object_linking_layer_with_section_memory_manager(
        &self,
    ) -> RTDyldObjectLinkingLayer {
        let object_layer = unsafe {
            LLVMOrcCreateRTDyldObjectLinkingLayerWithSectionMemoryManager(self.execution_session)
        };
        RTDyldObjectLinkingLayer { object_layer }
    }
}

#[llvm_versions(12.0..=latest)]
impl_owned_ptr!(
    ObjectLayerRef,
    LLVMOrcObjectLayerRef,
    LLVMOrcDisposeObjectLayer
);

#[llvm_versions(12.0..=latest)]
#[derive(Debug)]
pub struct ObjectLayer<'jit> {
    object_layer: OwnedOrBorrowedPtr<'jit, ObjectLayerRef>,
}

#[llvm_versions(12.0..=latest)]
impl<'jit> ObjectLayer<'jit> {
    unsafe fn new_borrowed(object_layer: LLVMOrcObjectLayerRef) -> Self {
        assert!(!object_layer.is_null());
        ObjectLayer {
            object_layer: OwnedOrBorrowedPtr::borrowed(object_layer),
        }
    }
    unsafe fn new_owned(object_layer: LLVMOrcObjectLayerRef) -> Self {
        assert!(!object_layer.is_null());
        ObjectLayer {
            object_layer: OwnedOrBorrowedPtr::Owned(ObjectLayerRef(object_layer)),
        }
    }

    #[llvm_versions(13.0..=latest)]
    pub fn add_object_file(
        &self,
        jit_dylib: &JITDylib,
        object_buffer: MemoryBuffer,
    ) -> Result<(), LLVMError> {
        let result = LLVMError::new(unsafe {
            LLVMOrcObjectLayerAddObjectFile(
                self.object_layer.as_ptr(),
                jit_dylib.jit_dylib,
                object_buffer.memory_buffer,
            )
        });
        forget(object_buffer);
        result
    }

    // #[llvm_versions(14.0..=latest)]
    // pub fn add_object_file_with_rt(
    //     &self,
    //     rt: &ResourceTracker,
    //     object_buffer: MemoryBuffer,
    // ) -> Result<(), LLVMError> {
    //     let result = LLVMError::new(unsafe {
    //         LLVMOrcObjectLayerAddObjectFileWithRT(
    //             self.object_layer.as_ptr(),
    //             rt.rt,
    //             object_buffer.memory_buffer,
    //         )
    //     });
    //     forget(object_buffer);
    //     result
    // }

    pub fn emit() {
        todo!();
    }
}

#[llvm_versions(12.0..=latest)]
impl From<RTDyldObjectLinkingLayer> for ObjectLayer<'static> {
    fn from(rt_dyld_object_linking_layer: RTDyldObjectLinkingLayer) -> Self {
        unsafe { ObjectLayer::new_borrowed(rt_dyld_object_linking_layer.object_layer) }
    }
}

// Does not have a drop implementation as calling LLVMOrcDisposeObjectLayer
// on an RTDyldObjectLinkingLayer segfaults. Does leak memory!!!
#[llvm_versions(12.0..=latest)]
#[derive(Debug)]
#[repr(transparent)]
#[must_use]
pub struct RTDyldObjectLinkingLayer {
    object_layer: LLVMOrcObjectLayerRef,
}

#[llvm_versions(12.0..=latest)]
impl RTDyldObjectLinkingLayer {
    pub fn get<'a>(&'a self) -> ObjectLayer<'a> {
        unsafe { ObjectLayer::new_borrowed(self.object_layer) }
    }
}

#[llvm_versions(13.0..=latest)]
#[derive(Debug)]
pub struct ObjectTransformLayer<'a, 'jit: 'a> {
    object_transform_layer: LLVMOrcObjectTransformLayerRef,
    lljit: &'a LLJIT<'jit>,
}

#[llvm_versions(13.0..=latest)]
impl<'a, 'jit: 'a> ObjectTransformLayer<'a, 'jit> {
    unsafe fn new_borrowed(
        object_transform_layer: LLVMOrcObjectTransformLayerRef,
        lljit: &'a LLJIT<'jit>,
    ) -> Self {
        assert!(!object_transform_layer.is_null());
        ObjectTransformLayer {
            object_transform_layer,
            lljit,
        }
    }

    /// Sets the [`ObjectTransformer`] for this instance.
    /// ```
    /// use inkwell::{
    ///     memory_buffer::MemoryBufferRef,
    ///     orc2::{lljit::LLJIT, ObjectTransformer},
    /// };
    ///
    /// let lljit = LLJIT::create().expect("LLJIT::create failed");
    /// let mut object_transform_layer = lljit.get_object_transform_layer();
    ///
    /// let object_transformer: Box<dyn ObjectTransformer> = Box::new(|buffer: MemoryBufferRef| {
    ///     // Transformer implementation...
    ///     Ok(())
    /// });
    /// object_transform_layer.set_transformer(object_transformer);
    /// ```
    #[llvm_versions(13.0..=latest)]
    pub fn set_transformer(self, object_transformer: Box<dyn ObjectTransformer + 'jit>) {
        *self.lljit.object_transformer.borrow_mut() = Some(object_transformer);
        unsafe {
            LLVMOrcObjectTransformLayerSetTransform(
                self.object_transform_layer,
                object_transform_layer_transform_function,
                transmute(self.lljit.object_transformer.borrow().as_ref().unwrap()),
            );
        }
    }
}

#[llvm_versions(13.0..=latest)]
#[no_mangle]
extern "C" fn object_transform_layer_transform_function(
    ctx: *mut c_void,
    object_in_out: *mut LLVMMemoryBufferRef,
) -> LLVMErrorRef {
    let object_transformer: &mut Box<dyn ObjectTransformer> = unsafe { transmute(ctx) };
    match object_transformer.transform(MemoryBufferRef::new(object_in_out)) {
        Ok(()) => ptr::null_mut(),
        Err(llvm_error) => {
            unsafe {
                MemoryBufferRef::new(object_in_out).set_memory_buffer_unsafe(ptr::null_mut());
            }
            llvm_error.error
        }
    }
}

pub trait ObjectTransformer {
    fn transform(&mut self, object_in_out: MemoryBufferRef) -> Result<(), LLVMError>;
}

impl<F> ObjectTransformer for F
where
    F: FnMut(MemoryBufferRef) -> Result<(), LLVMError>,
{
    fn transform(&mut self, object_in_out: MemoryBufferRef) -> Result<(), LLVMError> {
        self(object_in_out)
    }
}

#[llvm_versions(13.0..=latest)]
#[derive(Debug)]
pub struct IRTransformLayer<'jit> {
    ir_transform_layer: LLVMOrcIRTransformLayerRef,
    _marker: PhantomData<&'jit ()>,
}

#[llvm_versions(13.0..=latest)]
impl<'jit> IRTransformLayer<'jit> {
    unsafe fn new_borrowed(ir_transform_layer: LLVMOrcIRTransformLayerRef) -> Self {
        assert!(!ir_transform_layer.is_null());
        IRTransformLayer {
            ir_transform_layer,
            _marker: PhantomData,
        }
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
