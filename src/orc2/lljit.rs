use std::{
    ffi::CStr,
    fmt::{self, Debug, Formatter},
    mem::{transmute, transmute_copy},
    ptr::null_mut,
    rc::Rc,
};

use libc::{c_char, c_void};
use llvm_sys::orc2::{
    lljit::{
        LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcDisposeLLJIT,
        LLVMOrcDisposeLLJITBuilder, LLVMOrcLLJITAddLLVMIRModule, LLVMOrcLLJITAddLLVMIRModuleWithRT,
        LLVMOrcLLJITAddObjectFile, LLVMOrcLLJITAddObjectFileWithRT, LLVMOrcLLJITBuilderRef,
        LLVMOrcLLJITBuilderSetJITTargetMachineBuilder,
        LLVMOrcLLJITBuilderSetObjectLinkingLayerCreator, LLVMOrcLLJITGetDataLayoutStr,
        LLVMOrcLLJITGetExecutionSession, LLVMOrcLLJITGetGlobalPrefix,
        LLVMOrcLLJITGetIRTransformLayer, LLVMOrcLLJITGetMainJITDylib,
        LLVMOrcLLJITGetObjLinkingLayer, LLVMOrcLLJITGetObjTransformLayer,
        LLVMOrcLLJITGetTripleString, LLVMOrcLLJITLookup, LLVMOrcLLJITMangleAndIntern,
        LLVMOrcLLJITRef,
    },
    LLVMOrcExecutionSessionRef, LLVMOrcObjectLayerRef,
};

use crate::{
    error::LLVMError,
    memory_buffer::MemoryBuffer,
    support::to_c_str,
    targets::{InitializationConfig, Target},
};

use super::{
    ExecutionSession, IRTransformLayer, JITDylib, JITTargetMachineBuilder, ObjectLayer,
    ObjectTransformLayer, ResourceTracker, SymbolStringPoolEntry, ThreadSafeModule,
};

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
    /// use inkwell::orc2::lljit::LLJIT;
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
    /// use inkwell::orc2::lljit::LLJIT;
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
