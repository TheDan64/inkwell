use std::{
    ffi::CStr,
    fmt::{self, Debug, Formatter},
    mem::{transmute, transmute_copy},
    ptr,
    rc::Rc,
};

use libc::{c_char, c_void};
#[llvm_versions(13.0..=latest)]
use llvm_sys::orc2::lljit::{
    LLVMOrcLLJITGetDataLayoutStr, LLVMOrcLLJITGetIRTransformLayer, LLVMOrcLLJITGetObjLinkingLayer,
    LLVMOrcLLJITGetObjTransformLayer,
};
use llvm_sys::orc2::LLVMOrcExecutionSessionRef;
#[llvm_versions(12.0..=latest)]
use llvm_sys::orc2::{
    lljit::{
        LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcDisposeLLJIT,
        LLVMOrcDisposeLLJITBuilder, LLVMOrcLLJITAddLLVMIRModule, LLVMOrcLLJITAddLLVMIRModuleWithRT,
        LLVMOrcLLJITAddObjectFile, LLVMOrcLLJITAddObjectFileWithRT, LLVMOrcLLJITBuilderRef,
        LLVMOrcLLJITBuilderSetJITTargetMachineBuilder,
        LLVMOrcLLJITBuilderSetObjectLinkingLayerCreator, LLVMOrcLLJITGetExecutionSession,
        LLVMOrcLLJITGetGlobalPrefix, LLVMOrcLLJITGetMainJITDylib, LLVMOrcLLJITGetTripleString,
        LLVMOrcLLJITLookup, LLVMOrcLLJITMangleAndIntern, LLVMOrcLLJITRef,
    },
    LLVMOrcObjectLayerRef,
};
#[llvm_versions(11.0)]
use llvm_sys::orc2::{
    LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcDisposeLLJIT, LLVMOrcDisposeLLJITBuilder,
    LLVMOrcLLJITAddLLVMIRModule, LLVMOrcLLJITAddObjectFile, LLVMOrcLLJITBuilderRef,
    LLVMOrcLLJITBuilderSetJITTargetMachineBuilder, LLVMOrcLLJITGetExecutionSession,
    LLVMOrcLLJITGetGlobalPrefix, LLVMOrcLLJITGetMainJITDylib, LLVMOrcLLJITGetTripleString,
    LLVMOrcLLJITLookup, LLVMOrcLLJITMangleAndIntern, LLVMOrcLLJITRef,
};

use crate::{
    data_layout::DataLayout,
    error::LLVMError,
    memory_buffer::MemoryBuffer,
    support::{to_c_str, LLVMStringOrRaw},
    targets::{InitializationConfig, Target, TargetTriple},
};

use super::{
    ExecutionSession, JITDylib, JITTargetMachineBuilder, SymbolStringPoolEntry, ThreadSafeModule,
};
#[llvm_versions(13.0..=latest)]
use super::{IRTransformLayer, ObjectTransformLayer};
#[llvm_versions(12.0..=latest)]
use super::{ObjectLayer, ResourceTracker};

#[derive(Debug)]
pub enum Error {
    LLVMError(LLVMError),
    String(String),
}

/// Represents a reference to an LLVM `LLJIT` execution engine.
#[derive(Debug, Clone)]
pub struct LLJIT {
    inner: Rc<LLJITInner>,
}

impl LLJIT {
    unsafe fn create_with_builder(
        builder: LLVMOrcLLJITBuilderRef,
        #[cfg(not(feature = "llvm11-0"))] object_linking_layer_creator: Option<
            Box<Box<dyn ObjectLinkingLayerCreator>>,
        >,
    ) -> Result<Self, Error> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| Error::String(e))?;

        let mut lljit: LLVMOrcLLJITRef = ptr::null_mut();
        let error = LLVMOrcCreateLLJIT(&mut lljit, builder);
        LLVMError::new(error).map_err(|e| Error::LLVMError(e))?;
        assert!(!lljit.is_null());
        Ok(LLJIT {
            inner: Rc::new(LLJITInner {
                lljit,
                #[cfg(not(feature = "llvm11-0"))]
                object_linking_layer_creator,
            }),
        })
    }

    /// Creates a `LLJIT` instance with the default builder.
    /// ```
    /// use inkwell::orc2::lljit::LLJIT;
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// ```
    pub fn create() -> Result<Self, Error> {
        unsafe {
            LLJIT::create_with_builder(
                ptr::null_mut(),
                #[cfg(not(feature = "llvm11-0"))]
                None,
            )
        }
    }

    /// Returns the main [`JITDylib`].
    /// ```
    /// use inkwell::orc2::lljit::LLJIT;
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let main_jd = jit.get_main_jit_dylib();
    /// ```
    pub fn get_main_jit_dylib(&self) -> JITDylib {
        unsafe {
            JITDylib::new(
                LLVMOrcLLJITGetMainJITDylib(self.inner.lljit),
                self.get_execution_session(),
            )
        }
    }

    /// Adds `module` to `jit_dylib` in the execution engine.
    /// ```
    /// use inkwell::orc2::{lljit::LLJIT, ThreadSafeContext};
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
            LLVMOrcLLJITAddLLVMIRModule(
                self.inner.lljit,
                jit_dylib.jit_dylib,
                module.thread_safe_module,
            )
        };
        // module.owned_by_lljit should be None at this point.
        *module.owned_by_lljit.borrow_mut() = Some(self.clone());
        LLVMError::new(error)
    }

    /// Adds `module` to `rt`'s JITDylib in the execution engine.
    /// ```
    /// use inkwell::orc2::{lljit::LLJIT, ThreadSafeContext};
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
    #[llvm_versions(12.0..=latest)]
    pub fn add_module_with_rt<'ctx>(
        &self,
        rt: &ResourceTracker,
        module: ThreadSafeModule<'ctx>,
    ) -> Result<(), LLVMError> {
        let error = unsafe {
            LLVMOrcLLJITAddLLVMIRModuleWithRT(self.inner.lljit, rt.rt, module.thread_safe_module)
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
                self.inner.lljit,
                jit_dylib.jit_dylib,
                object_buffer.transfer_ownership_to_llvm(),
            ))
        }
    }

    #[llvm_versions(12.0..=latest)]
    pub fn add_object_file_with_rt(
        &self,
        rt: &ResourceTracker,
        object_buffer: MemoryBuffer,
    ) -> Result<(), LLVMError> {
        unsafe {
            LLVMError::new(LLVMOrcLLJITAddObjectFileWithRT(
                self.inner.lljit,
                rt.rt,
                object_buffer.transfer_ownership_to_llvm(),
            ))
        }
    }

    /// Attempts to lookup a function by `name` in the execution engine.
    ///
    /// It is recommended to use [`get_function()`](LLJIT::get_function())
    /// instead of this method when intending to call the function
    /// pointer so that you don't have to do error-prone transmutes yourself.
    pub fn get_function_address(&self, name: &str) -> Result<u64, LLVMError> {
        let mut function_address = 0;
        let error = unsafe {
            LLVMOrcLLJITLookup(
                self.inner.lljit,
                &mut function_address,
                to_c_str(name).as_ptr(),
            )
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
    /// use inkwell::orc2::{ThreadSafeContext, lljit::LLJIT};
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
            ExecutionSession::new_borrowed(
                LLVMOrcLLJITGetExecutionSession(self.inner.lljit),
                Some(self.clone()),
            )
        }
    }

    ///
    /// ```
    /// use inkwell::{orc2::lljit::LLJIT, targets::TargetMachine};
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let triple = jit.get_triple();
    /// assert_eq!(triple, TargetMachine::get_default_triple());
    /// ```
    pub fn get_triple<'jit>(&'jit self) -> TargetTriple<'jit> {
        TargetTriple::new(unsafe {
            LLVMStringOrRaw::borrowed(LLVMOrcLLJITGetTripleString(self.inner.lljit))
        })
    }

    pub fn get_global_prefix(&self) -> i8 {
        unsafe { LLVMOrcLLJITGetGlobalPrefix(self.inner.lljit) }
    }

    pub fn mangle_and_intern(&self, unmangled_name: &str) -> SymbolStringPoolEntry {
        unsafe {
            SymbolStringPoolEntry::new(LLVMOrcLLJITMangleAndIntern(
                self.inner.lljit,
                to_c_str(unmangled_name).as_ptr(),
            ))
        }
    }

    #[llvm_versions(13.0..=latest)]
    pub fn get_object_linking_layer(&self) -> ObjectLayer {
        unsafe { ObjectLayer::new_borrowed(LLVMOrcLLJITGetObjLinkingLayer(self.inner.lljit)) }
    }

    #[llvm_versions(13.0..=latest)]
    pub fn get_object_transform_layer(&self) -> ObjectTransformLayer {
        unsafe {
            ObjectTransformLayer::new_borrowed(LLVMOrcLLJITGetObjTransformLayer(self.inner.lljit))
        }
    }

    #[llvm_versions(13.0..=latest)]
    pub fn get_ir_transform_layer(&self) -> IRTransformLayer {
        unsafe { IRTransformLayer::new_borrowed(LLVMOrcLLJITGetIRTransformLayer(self.inner.lljit)) }
    }

    ///
    /// ```
    /// use inkwell::{orc2::lljit::LLJIT};
    /// # use inkwell::targets::{CodeModel, RelocMode, Target, TargetMachine};
    /// # use inkwell::OptimizationLevel;
    ///
    /// let jit = LLJIT::create().expect("LLJIT::create failed");
    /// let data_layout = jit.get_data_layout();
    /// # let target_tripple = TargetMachine::get_default_triple();
    /// # let target_cpu_features_llvm_string = TargetMachine::get_host_cpu_features();
    /// # let target_cpu_features = target_cpu_features_llvm_string
    /// #     .to_str()
    /// #     .expect("TargetMachine::get_host_cpu_features returned invalid string");
    /// # let target = Target::from_triple(&target_tripple).expect("Target::from_triple failed");
    /// # let target_machine = target
    /// #    .create_target_machine(
    /// #        &target_tripple,
    /// #        "",
    /// #        target_cpu_features,
    /// #        OptimizationLevel::Default,
    /// #        RelocMode::Default,
    /// #        CodeModel::Default,
    /// #    )
    /// #    .expect("Target::create_target_machine failed");
    /// assert_eq!(data_layout, target_machine.get_target_data().get_data_layout());
    /// ```
    /// In the example above `target_machine` is the host machine.
    #[llvm_versions(13.0..=latest)]
    pub fn get_data_layout<'jit>(&'jit self) -> DataLayout<'jit> {
        unsafe { DataLayout::new_borrowed(LLVMOrcLLJITGetDataLayoutStr(self.inner.lljit)) }
    }
}

#[llvm_versioned_item]
struct LLJITInner {
    lljit: LLVMOrcLLJITRef,
    #[llvm_versions(12.0..=latest)]
    object_linking_layer_creator: Option<Box<Box<dyn ObjectLinkingLayerCreator>>>,
}
impl Debug for LLJITInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "llvm11-0"))]
        return f
            .debug_struct("LLJITInner")
            .field("lljit", &self.lljit)
            .field(
                "object_linking_layer_creator",
                &self.object_linking_layer_creator.as_ref().map(|_| ()),
            )
            .finish();
        #[cfg(feature = "llvm11-0")]
        return f
            .debug_struct("LLJITInner")
            .field("lljit", &self.lljit)
            .finish();
    }
}
impl Drop for LLJITInner {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeLLJIT(self.lljit);
        }
    }
}

/// An `LLJITBuilder` is used to create custom [`LLJIT`] instances.
#[llvm_versioned_item]
pub struct LLJITBuilder {
    builder: LLJITBuilderRef,
    #[llvm_versions(12.0..=latest)]
    object_linking_layer_creator: Option<Box<Box<dyn ObjectLinkingLayerCreator>>>,
}

impl LLJITBuilder {
    unsafe fn new(builder: LLVMOrcLLJITBuilderRef) -> Self {
        assert!(!builder.is_null());
        LLJITBuilder {
            builder: LLJITBuilderRef { builder },
            #[cfg(not(feature = "llvm11-0"))]
            object_linking_layer_creator: None,
        }
    }

    /// Creates an `LLJITBuilder`.
    /// ```
    /// use inkwell::orc2::lljit::LLJITBuilder;
    ///
    /// let jit_builder = LLJITBuilder::create();
    /// ```
    pub fn create() -> Self {
        unsafe { LLJITBuilder::new(LLVMOrcCreateLLJITBuilder()) }
    }

    /// Builds the [`LLJIT`] instance. Consumes the `LLJITBuilder`.
    /// ```
    /// use inkwell::orc2::lljit::LLJITBuilder;
    ///
    /// let jit_builder = LLJITBuilder::create();
    /// let jit = jit_builder.build().expect("LLJITBuilder::build failed");
    /// ```
    pub fn build(self) -> Result<LLJIT, Error> {
        unsafe {
            match LLJIT::create_with_builder(
                self.builder.builder,
                #[cfg(not(feature = "llvm11-0"))]
                self.object_linking_layer_creator,
            ) {
                e @ Err(Error::String(_)) => e,
                res => {
                    self.builder.transfer_ownership_to_llvm();
                    res
                }
            }
        }
    }

    pub fn set_jit_target_machine_builder(
        self,
        jit_target_machine_builder: JITTargetMachineBuilder,
    ) -> Self {
        unsafe {
            LLVMOrcLLJITBuilderSetJITTargetMachineBuilder(
                self.builder.builder,
                jit_target_machine_builder.transfer_ownership_to_llvm(),
            );
        }
        self
    }

    #[llvm_versions(12.0..=latest)]
    pub fn set_object_linking_layer_creator(
        mut self,
        object_linking_layer_creator: Box<dyn ObjectLinkingLayerCreator>,
    ) -> Self {
        let object_linking_layer_creator = Box::new(object_linking_layer_creator);
        unsafe {
            LLVMOrcLLJITBuilderSetObjectLinkingLayerCreator(
                self.builder.builder,
                object_linking_layer_creator_function,
                transmute(object_linking_layer_creator.as_ref()),
            );
        }
        self.object_linking_layer_creator = Some(object_linking_layer_creator);
        self
    }

    #[llvm_versions(12.0..=latest)]
    pub fn set_object_linking_layer_creator_raw<'ctx, Ctx>(
        self,
        object_linking_layer_creator_function: extern "C" fn(
            &Ctx,
            LLVMOrcExecutionSessionRef,
            *const c_char,
        ) -> LLVMOrcObjectLayerRef,
        ctx: &'ctx mut Ctx,
    ) -> Self {
        unsafe {
            LLVMOrcLLJITBuilderSetObjectLinkingLayerCreator(
                self.builder.builder,
                transmute(object_linking_layer_creator_function),
                transmute(ctx as *mut Ctx),
            );
        }
        self
    }
}

impl Debug for LLJITBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[cfg(not(feature = "llvm11-0"))]
        return f
            .debug_struct("LLJITBuilder")
            .field("builder", &self.builder)
            .field(
                "object_linking_layer_creator",
                &self.object_linking_layer_creator.as_ref().map(|_| ()),
            )
            .finish();
        #[cfg(feature = "llvm11-0")]
        return f
            .debug_struct("LLJITBuilder")
            .field("builder", &self.builder)
            .finish();
    }
}

#[derive(Debug)]
struct LLJITBuilderRef {
    builder: LLVMOrcLLJITBuilderRef,
}

impl LLJITBuilderRef {
    unsafe fn transfer_ownership_to_llvm(mut self) -> LLVMOrcLLJITBuilderRef {
        let builder = self.builder;
        self.builder = ptr::null_mut();
        builder
    }
}
impl Drop for LLJITBuilderRef {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeLLJITBuilder(self.builder);
        }
    }
}

#[llvm_versions(12.0..=latest)]
#[no_mangle]
extern "C" fn object_linking_layer_creator_function(
    ctx: *mut c_void,
    execution_session: LLVMOrcExecutionSessionRef,
    triple: *const c_char,
) -> LLVMOrcObjectLayerRef {
    unsafe {
        let object_linking_layer_creator: &Box<dyn ObjectLinkingLayerCreator> = transmute(ctx);
        let object_layer = object_linking_layer_creator.create_object_linking_layer(
            ExecutionSession::new_borrowed(execution_session, None),
            CStr::from_ptr(triple),
        );
        object_layer.transfer_ownership_to_llvm()
    }
}

#[llvm_versions(12.0..=latest)]
pub trait ObjectLinkingLayerCreator {
    fn create_object_linking_layer(
        &self,
        execution_session: ExecutionSession,
        triple: &CStr,
    ) -> ObjectLayer;
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
