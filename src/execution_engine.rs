use libc::c_int;
use llvm_sys::execution_engine::{
    LLVMAddGlobalMapping, LLVMAddModule, LLVMDisposeExecutionEngine, LLVMExecutionEngineRef, LLVMFindFunction,
    LLVMFreeMachineCodeForFunction, LLVMGenericValueRef, LLVMGetExecutionEngineTargetData, LLVMGetFunctionAddress,
    LLVMLinkInInterpreter, LLVMLinkInMCJIT, LLVMRemoveModule, LLVMRunFunction, LLVMRunFunctionAsMain,
    LLVMRunStaticConstructors, LLVMRunStaticDestructors,
};

use crate::context::Context;
use crate::module::Module;
use crate::support::{LLVMString, to_c_str};
use crate::targets::TargetData;
use crate::values::{AnyValue, AsValueRef, FunctionValue, GenericValue};

use llvm_sys::LLVMModule;
use llvm_sys::prelude::LLVMModuleRef;

use std::cell::RefCell;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;
use std::mem::{MaybeUninit, forget, size_of, transmute_copy};
use std::ptr::NonNull;

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionLookupError {
    JITNotEnabled,
    FunctionNotFound, // 404!
}

impl Error for FunctionLookupError {}

impl FunctionLookupError {
    fn as_str(&self) -> &str {
        match self {
            FunctionLookupError::JITNotEnabled => "ExecutionEngine does not have JIT functionality enabled",
            FunctionLookupError::FunctionNotFound => "Function not found in ExecutionEngine",
        }
    }
}

impl Display for FunctionLookupError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "FunctionLookupError({})", self.as_str())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveModuleError {
    /// The `ExecutionEngine` does not own the module behind this [`EngineModule`] (it belongs
    /// to another engine, or that engine has been dropped).
    ModuleNotOwned,
    LLVMError(LLVMString),
}

impl Error for RemoveModuleError {
    // This method is deprecated on nighty so it's probably not
    // something we should worry about
    fn description(&self) -> &str {
        self.as_str()
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl RemoveModuleError {
    fn as_str(&self) -> &str {
        match self {
            RemoveModuleError::ModuleNotOwned => "Module is not owned by this Execution Engine",
            RemoveModuleError::LLVMError(string) => string.to_str().unwrap_or("LLVMError with invalid unicode"),
        }
    }
}

impl Display for RemoveModuleError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "RemoveModuleError({})", self.as_str())
    }
}

/// An LLVM execution engine (MCJIT or interpreter).
///
/// The engine owns the modules handed to it -- by [`Module::create_jit_execution_engine`] and
/// friends, or by [`add_module`](Self::add_module) -- and destroys them with itself, mirroring
/// LLVM's `ExecutionEngine`, which takes `std::unique_ptr<Module>`s. Each owned module is
/// represented by an [`EngineModule`] handle, which [`remove_module`](Self::remove_module)
/// turns back into a [`Module`]. A module must be complete when it is handed over: MCJIT
/// compiles it as a whole the first time one of its symbols is requested (or on
/// [`run_static_constructors`](Self::run_static_constructors)) and ignores any later change.
/// Add new code as new modules.
///
/// If several modules define the same symbol, MCJIT does not report an error: whichever
/// definition is linked last wins, and which module gets compiled for a lookup is unspecified.
#[derive(Debug)]
pub struct ExecutionEngine<'ctx> {
    execution_engine: NonNull<llvm_sys::execution_engine::LLVMOpaqueExecutionEngine>,
    /// Owned by the engine; forgotten rather than dropped.
    target_data: Option<TargetData>,
    /// The modules the engine currently owns. `LLVMRemoveModule` does not check ownership, so
    /// this is what `remove_module` validates an `EngineModule` against.
    modules: RefCell<Vec<NonNull<LLVMModule>>>,
    jit_mode: bool,
    _marker: PhantomData<&'ctx Context>,
}

impl PartialEq for ExecutionEngine<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.execution_engine == other.execution_engine
    }
}

impl Eq for ExecutionEngine<'_> {}

impl<'ctx> ExecutionEngine<'ctx> {
    /// Wraps a raw execution engine, taking ownership.
    ///
    /// # Safety
    ///
    /// `execution_engine` must be valid and uniquely owned by the caller, and the context it
    /// was created for must outlive the returned value.
    pub unsafe fn new(execution_engine: LLVMExecutionEngineRef, jit_mode: bool) -> Self {
        unsafe {
            let execution_engine = NonNull::new(execution_engine).expect("execution engine must be non-null");

            // REVIEW: Will we have to do this for LLVMGetExecutionEngineTargetMachine too?
            let target_data = LLVMGetExecutionEngineTargetData(execution_engine.as_ptr());

            ExecutionEngine {
                execution_engine,
                target_data: Some(TargetData::new(target_data)),
                modules: RefCell::new(Vec::new()),
                jit_mode,
                _marker: PhantomData,
            }
        }
    }

    /// Acquires the underlying raw pointer belonging to this `ExecutionEngine` type.
    pub fn as_mut_ptr(&self) -> LLVMExecutionEngineRef {
        self.execution_engine.as_ptr()
    }

    #[inline]
    pub(crate) fn execution_engine_inner(&self) -> LLVMExecutionEngineRef {
        self.execution_engine.as_ptr()
    }

    /// Records a module LLVM has taken ownership of and returns its handle.
    pub(crate) fn track_module(&self, module: LLVMModuleRef) -> EngineModule<'ctx> {
        let module = NonNull::new(module).expect("module must be non-null");

        self.modules.borrow_mut().push(module);

        EngineModule {
            module,
            _marker: PhantomData,
        }
    }

    // This function is noop, but required for proper MCJIT initialization and
    // registration. Without it, LTO is free to eliminate it
    pub(crate) fn link_in_mc_jit() {
        unsafe { LLVMLinkInMCJIT() }
    }

    // This function is noop, but required for proper MCJIT initialization and
    // registration. Without it, LTO is free to eliminate it
    pub(crate) fn link_in_interpreter() {
        unsafe {
            LLVMLinkInInterpreter();
        }
    }

    /// Maps the specified value to an address.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::targets::{InitializationConfig, Target};
    /// use inkwell::context::Context;
    /// use inkwell::OptimizationLevel;
    ///
    /// Target::initialize_native(&InitializationConfig::default()).unwrap();
    ///
    /// extern fn sumf(a: f64, b: f64) -> f64 {
    ///     a + b
    /// }
    ///
    /// let context = Context::create();
    /// let module = context.create_module("test");
    /// let builder = context.create_builder();
    ///
    /// let ft = context.f64_type();
    /// let fnt = ft.fn_type(&[], false);
    ///
    /// let f = module.add_function("test_fn", fnt, None);
    /// let b = context.append_basic_block(f, "entry");
    ///
    /// builder.position_at_end(b);
    ///
    /// let extf = module.add_function("sumf", ft.fn_type(&[ft.into(), ft.into()], false), None);
    ///
    /// let argf = ft.const_float(64.);
    /// let call_site_value = builder.build_call(extf, &[argf.into(), argf.into()], "retv").unwrap();
    /// let retv = call_site_value.try_as_basic_value().unwrap_basic().into_float_value();
    ///
    /// builder.build_return(Some(&retv)).unwrap();
    ///
    /// let (ee, _) = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    /// ee.add_global_mapping(&extf, sumf as usize);
    ///
    /// let result = unsafe { ee.run_function(f, &[]) }.as_float(&ft);
    ///
    /// assert_eq!(result, 128.);
    /// ```
    pub fn add_global_mapping(&self, value: &dyn AnyValue<'ctx>, addr: usize) {
        unsafe { LLVMAddGlobalMapping(self.execution_engine_inner(), value.as_value_ref(), addr as *mut _) }
    }

    /// Hands a complete module to the engine, which owns it from now on.
    ///
    /// The returned [`EngineModule`] is the only way to refer to the module afterwards, and it
    /// gives no access to the module's contents: MCJIT compiles a module as a whole and ignores
    /// later changes.
    ///
    /// ```rust,no_run
    /// use inkwell::targets::{InitializationConfig, Target};
    /// use inkwell::context::Context;
    /// use inkwell::OptimizationLevel;
    ///
    /// Target::initialize_native(&InitializationConfig::default()).unwrap();
    ///
    /// let context = Context::create();
    /// let (ee, _main) = context.create_module("main").create_jit_execution_engine(OptimizationLevel::None).unwrap();
    ///
    /// let helpers = ee.add_module(context.create_module("helpers"));
    /// // ... look up and run functions ...
    /// let helpers = ee.remove_module(helpers).unwrap();
    /// assert_eq!(helpers.get_name().to_str(), Ok("helpers"));
    /// ```
    pub fn add_module(&self, module: Module<'ctx>) -> EngineModule<'ctx> {
        let module = module.into_raw();

        unsafe { LLVMAddModule(self.execution_engine_inner(), module) }

        self.track_module(module)
    }

    /// Takes a module back from the engine.
    ///
    /// Machine code already generated from the module stays valid until the engine is dropped,
    /// and its symbols keep resolving through [`get_function`](Self::get_function). Values of
    /// the returned module must no longer be passed to this engine (e.g.
    /// [`run_function`](Self::run_function)).
    ///
    /// # Errors
    ///
    /// [`ModuleNotOwned`](RemoveModuleError::ModuleNotOwned) if this engine does not own the
    /// module (the handle came from another engine, which keeps its module), or
    /// [`LLVMError`](RemoveModuleError::LLVMError) if LLVM refuses the removal.
    pub fn remove_module(&self, module: EngineModule<'ctx>) -> Result<Module<'ctx>, RemoveModuleError> {
        // `LLVMRemoveModule` does not check ownership itself: it would hand back a module
        // another engine still owns, or one that no longer exists.
        let index = self
            .modules
            .borrow()
            .iter()
            .position(|m| *m == module.module)
            .ok_or(RemoveModuleError::ModuleNotOwned)?;

        let mut new_module = MaybeUninit::uninit();
        let mut err_string: *mut ::libc::c_char = ::core::ptr::null_mut();

        let code = unsafe {
            LLVMRemoveModule(
                self.execution_engine_inner(),
                module.as_mut_ptr(),
                new_module.as_mut_ptr(),
                &mut err_string,
            )
        };

        if code == 1 {
            unsafe {
                return Err(RemoveModuleError::LLVMError(LLVMString::new(err_string)));
            }
        }

        self.modules.borrow_mut().remove(index);

        Ok(unsafe { Module::new(new_module.assume_init()) })
    }

    /// Try to load a function from the execution engine.
    ///
    /// If a target hasn't already been initialized, spurious "function not
    /// found" errors may be encountered.
    ///
    /// The [`UnsafeFunctionPointer`] trait is designed so only `unsafe extern
    /// "C"` functions can be retrieved via the `get_function()` method. If you
    /// get funny type errors then it's probably because you have specified the
    /// wrong calling convention or forgotten to specify the retrieved function
    /// as `unsafe`.
    ///
    /// # Examples
    ///
    ///
    /// ```rust,no_run
    /// # use inkwell::targets::{InitializationConfig, Target};
    /// # use inkwell::context::Context;
    /// # use inkwell::OptimizationLevel;
    /// # Target::initialize_native(&InitializationConfig::default()).unwrap();
    /// let context = Context::create();
    /// let module = context.create_module("test");
    /// let builder = context.create_builder();
    ///
    /// // Set up the function signature
    /// let double = context.f64_type();
    /// let sig = double.fn_type(&[], false);
    ///
    /// // Add the function to our module
    /// let f = module.add_function("test_fn", sig, None);
    /// let b = context.append_basic_block(f, "entry");
    /// builder.position_at_end(b);
    ///
    /// // Insert a return statement
    /// let ret = double.const_float(64.0);
    /// builder.build_return(Some(&ret)).unwrap();
    ///
    /// // create the JIT engine
    /// let (ee, _) = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    ///
    /// // fetch our JIT'd function and execute it
    /// unsafe {
    ///     let test_fn = ee.get_function::<unsafe extern "C" fn() -> f64>("test_fn").unwrap();
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
    /// The `JitFunction` wrapper borrows the execution engine, so a function cannot
    /// outlive the engine it came from. MCJIT never frees generated code before the
    /// engine is dropped, so a function fetched again after
    /// [`remove_module`](Self::remove_module) is still valid.
    ///
    /// [`UnsafeFunctionPointer`]: trait.UnsafeFunctionPointer.html
    pub unsafe fn get_function<F>(&self, fn_name: &str) -> Result<JitFunction<'_, F>, FunctionLookupError>
    where
        F: UnsafeFunctionPointer,
    {
        unsafe {
            if !self.jit_mode {
                return Err(FunctionLookupError::JITNotEnabled);
            }

            let address = self.get_function_address(fn_name)?;

            assert_eq!(
                size_of::<F>(),
                size_of::<usize>(),
                "The type `F` must have the same size as a function pointer"
            );

            Ok(JitFunction {
                inner: transmute_copy(&address),
                _marker: PhantomData,
            })
        }
    }

    /// Attempts to look up a function's address by its name. May return Err if the function cannot be
    /// found or some other unknown error has occurred.
    ///
    /// It is recommended to use `get_function` instead of this method when intending to call the function
    /// pointer so that you don't have to do error-prone transmutes yourself.
    pub fn get_function_address(&self, fn_name: &str) -> Result<usize, FunctionLookupError> {
        let c_string = to_c_str(fn_name);
        let address = unsafe { LLVMGetFunctionAddress(self.execution_engine_inner(), c_string.as_ptr()) };

        // REVIEW: Can also return 0 if no targets are initialized.
        // One option might be to set a (thread local?) global to true if any at all of the targets have been
        // initialized (maybe we could figure out which config in particular is the trigger)
        // and if not return an "NoTargetsInitialized" error, instead of not found.
        if address == 0 {
            return Err(FunctionLookupError::FunctionNotFound);
        }

        Ok(address as usize)
    }

    // REVIEW: Not sure if an EE's target data can change.. if so we might want to update the value
    // when making this call
    pub fn get_target_data(&self) -> &TargetData {
        self.target_data
            .as_ref()
            .expect("TargetData should always exist until Drop")
    }

    // REVIEW: Can also find nothing if no targeting is initialized. Maybe best to
    // do have a global flag for anything initialized. Catch is that it must be initialized
    // before EE is created
    // REVIEW: Should FunctionValue lifetime be tied to self not 'ctx?
    pub fn get_function_value(&self, fn_name: &str) -> Result<FunctionValue<'ctx>, FunctionLookupError> {
        if !self.jit_mode {
            return Err(FunctionLookupError::JITNotEnabled);
        }

        let c_string = to_c_str(fn_name);
        let mut function = MaybeUninit::uninit();

        let code = unsafe { LLVMFindFunction(self.execution_engine_inner(), c_string.as_ptr(), function.as_mut_ptr()) };

        if code == 0 {
            return unsafe { FunctionValue::new(function.assume_init()).ok_or(FunctionLookupError::FunctionNotFound) };
        };

        Err(FunctionLookupError::FunctionNotFound)
    }

    // TODOC: Marked as unsafe because input function could very well do something unsafe. It's up to the caller
    // to ensure that doesn't happen by defining their function correctly.
    pub unsafe fn run_function(
        &self,
        function: FunctionValue<'ctx>,
        args: &[&GenericValue<'ctx>],
    ) -> GenericValue<'ctx> {
        unsafe {
            let mut args: Vec<LLVMGenericValueRef> = args.iter().map(|val| val.generic_value.as_ptr()).collect();

            let value = LLVMRunFunction(
                self.execution_engine_inner(),
                function.as_value_ref(),
                args.len() as u32,
                args.as_mut_ptr(),
            ); // REVIEW: usize to u32 ok??

            GenericValue::new(value)
        }
    }

    // TODOC: Marked as unsafe because input function could very well do something unsafe. It's up to the caller
    // to ensure that doesn't happen by defining their function correctly.
    // SubType: Only for JIT EEs?
    pub unsafe fn run_function_as_main(&self, function: FunctionValue<'ctx>, args: &[&str]) -> c_int {
        unsafe {
            let cstring_args: Vec<_> = args.iter().map(|&arg| to_c_str(arg)).collect();
            let raw_args: Vec<*const _> = cstring_args.iter().map(|arg| arg.as_ptr()).collect();

            let environment_variables = []; // TODO: Support envp. Likely needs to be null terminated

            LLVMRunFunctionAsMain(
                self.execution_engine_inner(),
                function.as_value_ref(),
                raw_args.len() as u32,
                raw_args.as_ptr(),
                environment_variables.as_ptr(),
            ) // REVIEW: usize to u32 cast ok??
        }
    }

    pub fn free_fn_machine_code(&self, function: FunctionValue<'ctx>) {
        unsafe { LLVMFreeMachineCodeForFunction(self.execution_engine_inner(), function.as_value_ref()) }
    }

    // REVIEW: Is this actually safe?
    pub fn run_static_constructors(&self) {
        unsafe { LLVMRunStaticConstructors(self.execution_engine_inner()) }
    }

    // REVIEW: Is this actually safe? Can you double destruct/free?
    pub fn run_static_destructors(&self) {
        unsafe { LLVMRunStaticDestructors(self.execution_engine_inner()) }
    }
}

impl Drop for ExecutionEngine<'_> {
    fn drop(&mut self) {
        // The target data belongs to the engine.
        forget(
            self.target_data
                .take()
                .expect("TargetData should always exist until Drop"),
        );

        // Disposing the engine destroys every module it still owns and frees all generated
        // code. `JitFunction`s borrow `self`, so none can be alive here; `EngineModule`s may
        // outlive the engine but are inert without it.
        unsafe { LLVMDisposeExecutionEngine(self.execution_engine.as_ptr()) }
    }
}

/// A module owned by an [`ExecutionEngine`], as returned by the engine constructors on
/// [`Module`] and by [`ExecutionEngine::add_module`].
///
/// The handle is the only way to refer to the module afterwards: pass it to
/// [`ExecutionEngine::remove_module`] to take the module back. It deliberately gives no access
/// to the module's contents -- MCJIT compiles a module as a whole and ignores later changes, so
/// a module must be complete before it is handed over. On its own the handle does nothing, so
/// it may outlive its engine; the engine checks that it still owns the module before acting on
/// a handle.
#[derive(Debug)]
pub struct EngineModule<'ctx> {
    module: NonNull<LLVMModule>,
    _marker: PhantomData<&'ctx Context>,
}

impl EngineModule<'_> {
    /// Acquires the underlying raw pointer belonging to this `EngineModule` type.
    pub fn as_mut_ptr(&self) -> LLVMModuleRef {
        self.module.as_ptr()
    }
}

/// A wrapper around a function pointer which ensures the function being pointed
/// to doesn't accidentally outlive its execution engine: it borrows the engine.
#[derive(Clone)]
pub struct JitFunction<'ee, F> {
    inner: F,
    _marker: PhantomData<&'ee ()>,
}

impl<F: Copy> JitFunction<'_, F> {
    /// Returns the raw function pointer, consuming self in the process.
    /// This function is unsafe because the function pointer may dangle
    /// if the ExecutionEngine it came from is dropped. The caller is
    /// thus responsible for ensuring the ExecutionEngine remains valid.
    pub unsafe fn into_raw(self) -> F {
        self.inner
    }

    /// Returns the raw function pointer.
    /// This function is unsafe because the function pointer may dangle
    /// if the ExecutionEngine it came from is dropped. The caller is
    /// thus responsible for ensuring the ExecutionEngine remains valid.
    pub unsafe fn as_raw(&self) -> F {
        self.inner
    }
}

impl<F> Debug for JitFunction<'_, F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_tuple("JitFunction").field(&"<unnamed>").finish()
    }
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

        impl<Output, $( $param ),*> JitFunction<'_, unsafe extern "C" fn($( $param ),*) -> Output> {
            /// This method allows you to call the underlying function while making
            /// sure that the backing storage is not dropped too early and
            /// preserves the `unsafe` marker for any calls.
            #[allow(non_snake_case)]
            #[inline(always)]
            pub unsafe fn call(&self, $( $param: $param ),*) -> Output { unsafe {
                (self.inner)($( $param ),*)
            }}
        }

        impl_unsafe_fn!(@recurse $( $param ),*);
    };
}

impl_unsafe_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M);

#[cfg(feature = "experimental")]
pub mod experimental {
    use llvm_sys::error::{LLVMConsumeError, LLVMErrorRef, LLVMErrorTypeId, LLVMGetErrorMessage, LLVMGetErrorTypeId};
    use llvm_sys::orc::{
        LLVMOrcAddEagerlyCompiledIR, LLVMOrcAddLazilyCompiledIR, LLVMOrcCreateInstance, LLVMOrcDisposeInstance,
        LLVMOrcDisposeMangledSymbol, LLVMOrcGetErrorMsg, LLVMOrcGetMangledSymbol, LLVMOrcJITStackRef,
    };

    use crate::module::Module;
    use crate::support::to_c_str;
    use crate::targets::TargetMachine;

    use std::ffi::{CStr, CString};
    use std::mem::MaybeUninit;
    use std::ops::Deref;

    #[derive(Debug)]
    pub struct MangledSymbol(*mut libc::c_char);

    impl Deref for MangledSymbol {
        type Target = CStr;

        fn deref(&self) -> &CStr {
            unsafe { CStr::from_ptr(self.0) }
        }
    }

    impl Drop for MangledSymbol {
        fn drop(&mut self) {
            unsafe { LLVMOrcDisposeMangledSymbol(self.0) }
        }
    }

    #[derive(Debug)]
    pub struct LLVMError(LLVMErrorRef);

    impl LLVMError {
        // Null type id == success
        pub fn get_type_id(&self) -> LLVMErrorTypeId {
            // FIXME: Don't expose LLVMErrorTypeId
            unsafe { LLVMGetErrorTypeId(self.0) }
        }
    }

    impl Deref for LLVMError {
        type Target = CStr;

        fn deref(&self) -> &CStr {
            unsafe {
                CStr::from_ptr(LLVMGetErrorMessage(self.0)) // FIXME: LLVMGetErrorMessage consumes the error, needs LLVMDisposeErrorMessage after
            }
        }
    }

    impl Drop for LLVMError {
        fn drop(&mut self) {
            unsafe { LLVMConsumeError(self.0) }
        }
    }

    // TODO
    #[derive(Debug)]
    pub struct Orc(LLVMOrcJITStackRef);

    impl Orc {
        pub fn create(target_machine: TargetMachine) -> Self {
            let stack_ref = unsafe { LLVMOrcCreateInstance(target_machine.target_machine.as_ptr()) };

            Orc(stack_ref)
        }

        pub fn add_compiled_ir<'ctx>(&self, module: &Module<'ctx>, lazily: bool) -> Result<(), ()> {
            // let handle = MaybeUninit::uninit();
            // let _err =  if lazily {
            //     unsafe { LLVMOrcAddLazilyCompiledIR(self.0, handle.as_mut_ptr(), module.module.get(), sym_resolve, sym_resolve_ctx) }
            // } else {
            //     unsafe { LLVMOrcAddEagerlyCompiledIR(self.0, handle.as_mut_ptr(), module.module.get(), sym_resolve, sym_resolve_ctx) }
            // };

            Ok(())
        }

        /// Obtains an error message owned by the ORC JIT stack.
        pub fn get_error(&self) -> &CStr {
            let err_str = unsafe { LLVMOrcGetErrorMsg(self.0) };

            if err_str.is_null() {
                panic!("Needs to be optional")
            }

            unsafe { CStr::from_ptr(err_str) }
        }

        pub fn get_mangled_symbol(&self, symbol: &str) -> MangledSymbol {
            let mut mangled_symbol = MaybeUninit::uninit();
            let c_symbol = to_c_str(symbol);

            unsafe { LLVMOrcGetMangledSymbol(self.0, mangled_symbol.as_mut_ptr(), c_symbol.as_ptr()) };

            MangledSymbol(unsafe { mangled_symbol.assume_init() })
        }
    }

    impl Drop for Orc {
        fn drop(&mut self) {
            // REVIEW: This returns an LLVMErrorRef, not sure what we can do with it...
            // print to stderr maybe?
            LLVMError(unsafe { LLVMOrcDisposeInstance(self.0) });
        }
    }

    #[test]
    fn test_mangled_str() {
        use crate::OptimizationLevel;
        use crate::targets::{CodeModel, InitializationConfig, RelocMode, Target};

        Target::initialize_native(&InitializationConfig::default()).unwrap();

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).unwrap();
        let target_machine = target
            .create_target_machine(
                &target_triple,
                &"",
                &"",
                OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap();
        let orc = Orc::create(target_machine);

        assert_eq!(orc.get_error().to_str().unwrap(), "");

        let mangled_symbol = orc.get_mangled_symbol("MyStructName");

        assert_eq!(orc.get_error().to_str().unwrap(), "");

        // REVIEW: This doesn't seem very mangled...
        assert_eq!(mangled_symbol.to_str().unwrap(), "MyStructName");
    }
}
