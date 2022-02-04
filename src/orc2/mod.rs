pub mod lljit;

use std::{
    ffi::CStr,
    fmt,
    marker::PhantomData,
    mem::{forget, transmute},
    ptr, slice, vec,
};

use libc::c_void;
#[llvm_versions(12.0..=latest)]
use llvm_sys::orc2::{
    ee::LLVMOrcCreateRTDyldObjectLinkingLayerWithSectionMemoryManager, LLVMJITCSymbolMapPair,
    LLVMJITSymbolFlags, LLVMOrcAbsoluteSymbols, LLVMOrcDisposeMaterializationUnit,
    LLVMOrcDisposeObjectLayer, LLVMOrcExecutionSessionCreateBareJITDylib,
    LLVMOrcExecutionSessionCreateJITDylib, LLVMOrcExecutionSessionGetJITDylibByName,
    LLVMOrcJITDylibClear, LLVMOrcJITDylibCreateResourceTracker, LLVMOrcJITDylibDefine,
    LLVMOrcJITDylibGetDefaultResourceTracker, LLVMOrcMaterializationUnitRef, LLVMOrcObjectLayerRef,
    LLVMOrcReleaseResourceTracker, LLVMOrcResourceTrackerRef, LLVMOrcResourceTrackerRemove,
    LLVMOrcResourceTrackerTransferTo, LLVMOrcRetainSymbolStringPoolEntry,
    LLVMOrcSymbolStringPoolEntryStr,
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
        LLVMOrcCDependenceMapPair, LLVMOrcCSymbolAliasMapPair, LLVMOrcCSymbolFlagsMapPair,
        LLVMOrcCSymbolFlagsMapPairs, LLVMOrcCreateCustomMaterializationUnit, LLVMOrcDisposeSymbols,
        LLVMOrcIRTransformLayerEmit, LLVMOrcIRTransformLayerRef, LLVMOrcIndirectStubsManagerRef,
        LLVMOrcJITTargetMachineBuilderGetTargetTriple,
        LLVMOrcJITTargetMachineBuilderSetTargetTriple, LLVMOrcLazyCallThroughManagerRef,
        LLVMOrcLazyReexports, LLVMOrcMaterializationResponsibilityAddDependencies,
        LLVMOrcMaterializationResponsibilityAddDependenciesForAll,
        LLVMOrcMaterializationResponsibilityDefineMaterializing,
        LLVMOrcMaterializationResponsibilityDelegate,
        LLVMOrcMaterializationResponsibilityFailMaterialization,
        LLVMOrcMaterializationResponsibilityGetExecutionSession,
        LLVMOrcMaterializationResponsibilityGetInitializerSymbol,
        LLVMOrcMaterializationResponsibilityGetRequestedSymbols,
        LLVMOrcMaterializationResponsibilityGetSymbols,
        LLVMOrcMaterializationResponsibilityGetTargetDylib,
        LLVMOrcMaterializationResponsibilityNotifyEmitted,
        LLVMOrcMaterializationResponsibilityNotifyResolved,
        LLVMOrcMaterializationResponsibilityRef, LLVMOrcMaterializationResponsibilityReplace,
        LLVMOrcObjectLayerAddObjectFile, LLVMOrcObjectLayerEmit, LLVMOrcObjectTransformLayerRef,
        LLVMOrcObjectTransformLayerSetTransform,
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
    pub fn get_default_resource_tracker(&self) -> ResourceTracker<'jit> {
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

    #[llvm_versions(12.0..=latest)]
    pub fn define(&self, materialization_unit: MaterializationUnit<'jit>) -> Result<(), LLVMError> {
        let result = LLVMError::new(unsafe {
            LLVMOrcJITDylibDefine(self.jit_dylib, materialization_unit.materialization_unit)
        });
        forget(materialization_unit);
        result
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

    #[llvm_versions(13.0..=latest)]
    pub fn emit(
        &self,
        materialization_responsibility: MaterializationResponsibility,
        object_buffer: MemoryBuffer,
    ) {
        unsafe {
            LLVMOrcObjectLayerEmit(
                self.object_layer.as_ptr(),
                materialization_responsibility.materialization_responsibility,
                object_buffer.memory_buffer,
            );
        }
    }
}

#[llvm_versions(12.0..=latest)]
impl From<RTDyldObjectLinkingLayer> for ObjectLayer<'static> {
    fn from(rt_dyld_object_linking_layer: RTDyldObjectLinkingLayer) -> Self {
        unsafe { ObjectLayer::new_borrowed(rt_dyld_object_linking_layer.object_layer) }
    }
}

/// Represents an RTDyldObjectLinkingLayer a special [`ObjectLayer`].
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

/// A ObjectTransformLayer can transform object buffers before lowering.
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
                transmute::<&ObjectTransformerCtx<'jit>, _>(
                    self.lljit.object_transformer.borrow().as_ref().unwrap(),
                ),
            );
        }
    }
}

type ObjectTransformerCtx<'jit> = Box<dyn ObjectTransformer + 'jit>;

#[llvm_versions(13.0..=latest)]
#[no_mangle]
extern "C" fn object_transform_layer_transform_function(
    ctx: *mut c_void,
    object_in_out: *mut LLVMMemoryBufferRef,
) -> LLVMErrorRef {
    let object_transformer: &mut ObjectTransformerCtx = unsafe { transmute(ctx) };
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

    pub fn emit<'ctx>(
        &self,
        materialization_responsibility: MaterializationResponsibility,
        module: ThreadSafeModule<'ctx>,
    ) {
        unsafe {
            LLVMOrcIRTransformLayerEmit(
                self.ir_transform_layer,
                materialization_responsibility.materialization_responsibility,
                module.thread_safe_module,
            );
        }
        forget(module);
    }

    pub fn set_transform() {
        todo!();
    }
}

#[llvm_versions(12.0..=latest)]
#[llvm_versioned_item]
#[derive(Debug)]
pub struct MaterializationUnit<'jit> {
    materialization_unit: LLVMOrcMaterializationUnitRef,

    #[llvm_versions(13.0..=latest)]
    materializer: Option<*mut (dyn Materializer + 'jit)>,
    _marker: PhantomData<&'jit ()>,
}

#[llvm_versions(12.0..=latest)]
impl<'jit> MaterializationUnit<'jit> {
    unsafe fn new(materialization_unit: LLVMOrcMaterializationUnitRef) -> Self {
        assert!(!materialization_unit.is_null());
        MaterializationUnit {
            materialization_unit,
            #[cfg(not(any(feature = "llvm11-0", feature = "llvm12-0")))]
            materializer: None,
            _marker: PhantomData,
        }
    }

    #[llvm_versions(13.0..=latest)]
    pub fn create(
        name: &str,
        mut symbols: SymbolFlagsMapPairs,
        static_initializer: Option<SymbolStringPoolEntry>,
        materializer: Box<(dyn Materializer + 'jit)>,
    ) -> Self {
        let mut materialization_unit = MaterializationUnit {
            materialization_unit: ptr::null_mut(),
            materializer: Some(Box::leak(materializer)),
            _marker: PhantomData,
        };
        let materialization_unit_ref = unsafe {
            LLVMOrcCreateCustomMaterializationUnit(
                to_c_str(name).as_ptr(),
                transmute::<&MaterializationUnitCtx<'jit>, _>(
                    materialization_unit.materializer.as_ref().unwrap(),
                ),
                symbols.raw_ptr(),
                symbols.len(),
                static_initializer
                    .map(|s| s.entry)
                    .unwrap_or(ptr::null_mut()),
                materialization_unit_materialize,
                materialization_unit_discard,
                materialization_unit_destroy,
            )
        };
        assert!(!materialization_unit_ref.is_null());
        materialization_unit.materialization_unit = materialization_unit_ref;
        materialization_unit
    }

    pub fn from_absolute_symbols(mut symbols: SymbolMapPairs) -> Self {
        unsafe {
            MaterializationUnit::new(LLVMOrcAbsoluteSymbols(
                symbols.pairs.as_mut_ptr(),
                symbols.pairs.len(),
            ))
        }
    }

    #[llvm_versions(13.0..=latest)]
    pub fn create_with_lazy_reexports(
        lazy_call_through_manager: LazyCallThroughManager,
        indirect_stubs_manager: IndirectStubsManager,
        source_ref: &JITDylib,
        mut callable_aliases: SymbolAliasMapPairs,
    ) -> Self {
        unsafe {
            MaterializationUnit::new(LLVMOrcLazyReexports(
                lazy_call_through_manager.lazy_call_through_manager,
                indirect_stubs_manager.indirect_stubs_manager,
                source_ref.jit_dylib,
                callable_aliases.pairs.as_mut_ptr(),
                callable_aliases.pairs.len(),
            ))
        }
    }
}

#[llvm_versions(12.0..=latest)]
impl Drop for MaterializationUnit<'_> {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeMaterializationUnit(self.materialization_unit);
        }
    }
}

#[llvm_versions(13.0..=latest)]
type MaterializationUnitCtx<'jit> = *mut (dyn Materializer + 'jit);

#[llvm_versions(13.0..=latest)]
#[no_mangle]
extern "C" fn materialization_unit_materialize(
    ctx: *mut c_void,
    materialization_responsibility: LLVMOrcMaterializationResponsibilityRef,
) {
    unsafe {
        let materializer: &MaterializationUnitCtx = transmute(ctx);
        materializer
            .as_mut()
            .unwrap()
            .materialize(MaterializationResponsibility::new(
                materialization_responsibility,
            ));
        drop(Box::from_raw(*materializer));
    }
}

#[llvm_versions(13.0..=latest)]
#[no_mangle]
extern "C" fn materialization_unit_discard(
    ctx: *mut c_void,
    jit_dylib: LLVMOrcJITDylibRef,
    symbol: LLVMOrcSymbolStringPoolEntryRef,
) {
    unsafe {
        let materializer: &MaterializationUnitCtx = transmute(ctx);
        materializer
            .as_mut()
            .unwrap()
            .discard(JITDylib::new(jit_dylib), SymbolStringPoolEntry::new(symbol));
    }
}

#[llvm_versions(13.0..=latest)]
#[no_mangle]
extern "C" fn materialization_unit_destroy(ctx: *mut c_void) {
    unsafe {
        let materializer: &MaterializationUnitCtx = transmute(ctx);
        drop(Box::from_raw(*materializer))
    }
}

#[llvm_versions(13.0..=latest)]
pub trait Materializer {
    fn materialize(&mut self, materialization_responsibility: MaterializationResponsibility);
    fn discard(&mut self, jit_dylib: JITDylib, symbol: SymbolStringPoolEntry);
}

#[llvm_versions(13.0..=latest)]
#[must_use]
#[derive(Debug)]
pub struct MaterializationResponsibility {
    materialization_responsibility: LLVMOrcMaterializationResponsibilityRef,
}

#[llvm_versions(13.0..=latest)]
impl MaterializationResponsibility {
    unsafe fn new(materialization_responsibility: LLVMOrcMaterializationResponsibilityRef) -> Self {
        assert!(!materialization_responsibility.is_null());
        MaterializationResponsibility {
            materialization_responsibility,
        }
    }

    pub fn get_target_jit_dylib<'a>(&'a self) -> JITDylib<'a> {
        unsafe {
            JITDylib::new(LLVMOrcMaterializationResponsibilityGetTargetDylib(
                self.materialization_responsibility,
            ))
        }
    }
    pub fn get_execution_session<'a>(&'a self) -> ExecutionSession<'a> {
        unsafe {
            ExecutionSession::new_borrowed(LLVMOrcMaterializationResponsibilityGetExecutionSession(
                self.materialization_responsibility,
            ))
        }
    }

    pub fn get_symbols(&self) -> SymbolFlagsMapPairs {
        let mut num_pairs: usize = 0;
        unsafe {
            let ptr = LLVMOrcMaterializationResponsibilityGetSymbols(
                self.materialization_responsibility,
                &mut num_pairs,
            );
            SymbolFlagsMapPairs::from_raw_parts(ptr, num_pairs)
        }
    }

    pub fn get_static_initializer_symbol(&self) -> Option<SymbolStringPoolEntry> {
        let ptr = unsafe {
            LLVMOrcMaterializationResponsibilityGetInitializerSymbol(
                self.materialization_responsibility,
            )
        };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { SymbolStringPoolEntry::new(ptr) })
        }
    }

    pub fn get_requested_symbols(&self) -> SymbolStringPoolEntries {
        let mut num_symbols = 0;
        unsafe {
            let ptr = LLVMOrcMaterializationResponsibilityGetRequestedSymbols(
                self.materialization_responsibility,
                &mut num_symbols,
            );

            SymbolStringPoolEntries::from_raw_parts(ptr, num_symbols)
        }
    }

    pub fn notify_resolved(&self, mut symbols: SymbolMapPairs) -> Result<(), LLVMError> {
        LLVMError::new(unsafe {
            LLVMOrcMaterializationResponsibilityNotifyResolved(
                self.materialization_responsibility,
                symbols.pairs.as_mut_ptr(),
                symbols.pairs.len(),
            )
        })
    }

    pub fn notify_emitted(&self) -> Result<(), LLVMError> {
        LLVMError::new(unsafe {
            LLVMOrcMaterializationResponsibilityNotifyEmitted(self.materialization_responsibility)
        })
    }

    pub fn define_materializing(&self, mut pairs: SymbolFlagsMapPairs) -> Result<(), LLVMError> {
        LLVMError::new(unsafe {
            LLVMOrcMaterializationResponsibilityDefineMaterializing(
                self.materialization_responsibility,
                pairs.raw_ptr(),
                pairs.len(),
            )
        })
    }

    pub fn fail_materialization(&self) {
        unsafe {
            LLVMOrcMaterializationResponsibilityFailMaterialization(
                self.materialization_responsibility,
            );
        }
    }

    pub fn replace(&self, materialization_unit: &MaterializationUnit) -> Result<(), LLVMError> {
        LLVMError::new(unsafe {
            LLVMOrcMaterializationResponsibilityReplace(
                self.materialization_responsibility,
                materialization_unit.materialization_unit,
            )
        })
    }

    pub fn delegate(
        &self,
        mut symbols: Vec<SymbolStringPoolEntry>,
    ) -> Result<MaterializationResponsibility, LLVMError> {
        let mut ptr = ptr::null_mut();
        LLVMError::new(unsafe {
            LLVMOrcMaterializationResponsibilityDelegate(
                self.materialization_responsibility,
                transmute(symbols.as_mut_ptr()),
                symbols.len(),
                &mut ptr,
            )
        })?;
        Ok(unsafe { MaterializationResponsibility::new(ptr) })
    }

    pub fn add_dependencies(
        &self,
        name: SymbolStringPoolEntry,
        mut dependencies: DependenceMapPairs,
    ) {
        unsafe {
            LLVMOrcMaterializationResponsibilityAddDependencies(
                self.materialization_responsibility,
                name.entry,
                dependencies.pairs.as_mut_ptr(),
                dependencies.pairs.len(),
            )
        }
    }

    pub fn add_dependencies_for_all(&self, mut dependencies: DependenceMapPairs) {
        unsafe {
            LLVMOrcMaterializationResponsibilityAddDependenciesForAll(
                self.materialization_responsibility,
                dependencies.pairs.as_mut_ptr(),
                dependencies.pairs.len(),
            );
        }
    }
}

#[llvm_versions(13.0..=latest)]
#[derive(Debug)]
pub struct SymbolFlagsMapPairs {
    pairs: Vec<SymbolFlagsMapPair>,
}

#[llvm_versions(13.0..=latest)]
impl SymbolFlagsMapPairs {
    unsafe fn from_raw_parts(ptr: LLVMOrcCSymbolFlagsMapPairs, num_pairs: usize) -> Self {
        SymbolFlagsMapPairs {
            pairs: Vec::from_raw_parts(transmute(ptr), num_pairs, num_pairs),
        }
    }

    pub fn new(pairs: Vec<SymbolFlagsMapPair>) -> Self {
        SymbolFlagsMapPairs { pairs }
    }

    pub fn zip<N, F>(names: N, flags: F) -> Self
    where
        N: IntoIterator<Item = SymbolStringPoolEntry>,
        F: IntoIterator<Item = SymbolFlags>,
    {
        SymbolFlagsMapPairs {
            pairs: names
                .into_iter()
                .zip(flags.into_iter())
                .map(|(name, flags)| SymbolFlagsMapPair::new(name, flags))
                .collect(),
        }
    }

    fn raw_ptr(&mut self) -> LLVMOrcCSymbolFlagsMapPairs {
        unsafe { transmute(self.pairs.as_mut_ptr()) }
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn names(&self) -> impl Iterator<Item = &SymbolStringPoolEntry> {
        self.pairs.iter().map(|pair| pair.get_name())
    }

    pub fn flags(&self) -> impl Iterator<Item = &SymbolFlags> {
        self.pairs.iter().map(|pair| pair.get_flags())
    }

    pub fn iter(&self) -> impl Iterator<Item = &SymbolFlagsMapPair> {
        self.pairs.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SymbolFlagsMapPair> {
        self.pairs.iter_mut()
    }
}

#[llvm_versions(13.0..=latest)]
impl IntoIterator for SymbolFlagsMapPairs {
    type Item = SymbolFlagsMapPair;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

#[llvm_versions(13.0..=latest)]
#[repr(transparent)]
pub struct SymbolFlagsMapPair {
    pair: LLVMOrcCSymbolFlagsMapPair,
}

#[llvm_versions(13.0..=latest)]
impl SymbolFlagsMapPair {
    pub fn new(name: SymbolStringPoolEntry, flags: SymbolFlags) -> Self {
        Self {
            pair: LLVMOrcCSymbolFlagsMapPair {
                Name: unsafe { transmute(name) },
                Flags: unsafe { transmute(flags) },
            },
        }
    }

    pub fn get_name(&self) -> &SymbolStringPoolEntry {
        unsafe { transmute(&self.pair.Name) }
    }

    pub fn get_flags(&self) -> &SymbolFlags {
        unsafe { transmute(&self.pair.Flags) }
    }
}

#[llvm_versions(13.0..=latest)]
impl fmt::Debug for SymbolFlagsMapPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolFlagsMapPair")
            .field("name", self.get_name())
            .field("flags", self.get_flags())
            .finish()
    }
}

#[llvm_versions(12.0..=latest)]
#[repr(transparent)]
pub struct SymbolFlags {
    flags: LLVMJITSymbolFlags,
}

#[llvm_versions(12.0..=latest)]
impl SymbolFlags {
    pub fn new(generic_flags: u8, target_flags: u8) -> Self {
        SymbolFlags {
            flags: LLVMJITSymbolFlags {
                GenericFlags: generic_flags,
                TargetFlags: target_flags,
            },
        }
    }

    pub fn get_generic_flags(&self) -> &u8 {
        &self.flags.GenericFlags
    }

    pub fn get_target_flags(&self) -> &u8 {
        &self.flags.TargetFlags
    }
}

#[llvm_versions(12.0..=latest)]
impl Clone for SymbolFlags {
    fn clone(&self) -> Self {
        SymbolFlags::new(self.flags.GenericFlags, self.flags.TargetFlags)
    }
}

#[llvm_versions(12.0..=latest)]
impl fmt::Debug for SymbolFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolFlags")
            .field("generic_flags", self.get_generic_flags())
            .field("target_flags", self.get_target_flags())
            .finish()
    }
}

#[llvm_versions(12.0..=latest)]
#[derive(Debug)]
pub struct SymbolMapPairs {
    pairs: Vec<LLVMJITCSymbolMapPair>,
}

#[llvm_versions(13.0..=latest)]
#[derive(Debug)]
pub struct SymbolAliasMapPairs {
    pairs: Vec<LLVMOrcCSymbolAliasMapPair>,
}

#[llvm_versions(13.0..=latest)]
#[derive(Debug)]
pub struct DependenceMapPairs {
    pairs: Vec<LLVMOrcCDependenceMapPair>,
}

#[derive(Eq, PartialEq)]
#[repr(transparent)]
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

impl fmt::Debug for SymbolStringPoolEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "llvm11-0")]
        return f
            .debug_struct("SymbolStringPoolEntry")
            .field("entry", &self.entry)
            .finish();
        #[cfg(not(feature = "llvm11-0"))]
        return f
            .debug_struct("SymbolStringPoolEntry")
            .field("entry", &self.get_string())
            .field("ptr", &self.entry)
            .finish();
    }
}

impl Drop for SymbolStringPoolEntry {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcReleaseSymbolStringPoolEntry(self.entry);
        }
    }
}

#[llvm_versions(13.0..=latest)]
pub struct SymbolStringPoolEntries {
    entries: *mut LLVMOrcSymbolStringPoolEntryRef,
    len: usize,
}

#[llvm_versions(13.0..=latest)]
impl SymbolStringPoolEntries {
    unsafe fn from_raw_parts(entries: *mut LLVMOrcSymbolStringPoolEntryRef, len: usize) -> Self {
        assert!(!entries.is_null());
        SymbolStringPoolEntries { entries, len }
    }
}

#[llvm_versions(13.0..=latest)]
impl AsRef<[SymbolStringPoolEntry]> for SymbolStringPoolEntries {
    fn as_ref(&self) -> &[SymbolStringPoolEntry] {
        unsafe { slice::from_raw_parts(transmute(self.entries), self.len) }
    }
}

#[llvm_versions(13.0..=latest)]
impl AsMut<[SymbolStringPoolEntry]> for SymbolStringPoolEntries {
    fn as_mut(&mut self) -> &mut [SymbolStringPoolEntry] {
        unsafe { slice::from_raw_parts_mut(transmute(self.entries), self.len) }
    }
}

#[llvm_versions(13.0..=latest)]
impl fmt::Debug for SymbolStringPoolEntries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

#[llvm_versions(13.0..=latest)]
impl Drop for SymbolStringPoolEntries {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeSymbols(self.entries);
        }
    }
}

#[llvm_versions(13.0..=latest)]
#[derive(Debug)]
pub struct LazyCallThroughManager {
    lazy_call_through_manager: LLVMOrcLazyCallThroughManagerRef,
}

#[llvm_versions(13.0..=latest)]
#[derive(Debug)]
pub struct IndirectStubsManager {
    indirect_stubs_manager: LLVMOrcIndirectStubsManagerRef,
}
