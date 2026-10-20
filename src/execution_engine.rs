use libc::{c_int, c_void};
use llvm_sys::core::LLVMDisposeModule;
use llvm_sys::execution_engine::{
    LLVMAddGlobalMapping, LLVMAddModule, LLVMCreateExecutionEngineForModule, LLVMCreateInterpreterForModule,
    LLVMCreateJITCompilerForModule, LLVMCreateMCJITCompilerForModule, LLVMCreateSimpleMCJITMemoryManager,
    LLVMDisposeExecutionEngine, LLVMExecutionEngineRef, LLVMFindFunction, LLVMFreeMachineCodeForFunction,
    LLVMGenericValueRef, LLVMGetExecutionEngineTargetData, LLVMGetExecutionEngineTargetMachine, LLVMGetFunctionAddress,
    LLVMInitializeMCJITCompilerOptions, LLVMLinkInInterpreter, LLVMLinkInMCJIT, LLVMMCJITCompilerOptions,
    LLVMRemoveModule, LLVMRunFunction, LLVMRunFunctionAsMain, LLVMRunStaticConstructors, LLVMRunStaticDestructors,
};

use crate::OptimizationLevel;
use crate::context::Context;
use crate::data_layout::DataLayoutRef;
use crate::memory_manager::{
    McjitMemoryManager, MemoryManagerAdapter, allocate_code_section_adapter, allocate_data_section_adapter,
    destroy_adapter, finalize_memory_adapter,
};
use crate::module::Module;
use crate::support::{LLVMString, to_c_str};
use crate::targets::{CodeModel, InitializationConfig, Target, TargetMachineRef};
use crate::values::{AnyValue, AsValueRef, FunctionValue, GenericValue};

use llvm_sys::LLVMModule;
use llvm_sys::prelude::{LLVMBool, LLVMModuleRef};

use std::cell::Cell;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;
use std::mem::{MaybeUninit, size_of, transmute_copy};
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
pub enum RemoveModuleError<'ee> {
    /// The `ExecutionEngine` does not own the module behind the handle: it came from another
    /// engine. The handle is given back, so the module can still be removed from that engine.
    ModuleNotOwned(EngineModule<'ee>),
    /// LLVM refused the removal. (`LLVMRemoveModule` never does as of LLVM 24: it ignores the
    /// result of `ExecutionEngine::removeModule` and returns 0.)
    LLVMError(LLVMString),
}

impl Error for RemoveModuleError<'_> {
    // This method is deprecated on nighty so it's probably not
    // something we should worry about
    fn description(&self) -> &str {
        self.as_str()
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl RemoveModuleError<'_> {
    fn as_str(&self) -> &str {
        match self {
            RemoveModuleError::ModuleNotOwned(_) => "Module is not owned by this Execution Engine",
            RemoveModuleError::LLVMError(string) => string.to_str().unwrap_or("LLVMError with invalid unicode"),
        }
    }
}

impl Display for RemoveModuleError<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "RemoveModuleError({})", self.as_str())
    }
}

/// An LLVM execution engine (MCJIT or interpreter).
///
/// The engine owns the modules handed to it -- by [`Module::create_jit_execution_engine`] and
/// friends, or by [`add_module`](Self::add_module) -- and destroys them with itself, mirroring
/// LLVM's `ExecutionEngine`, which takes `std::unique_ptr<Module>`s. (The raw counterparts, for
/// modules and engines handled through `llvm-sys`, are [`create_jit_for_module`](Self::create_jit_for_module)
/// and friends, [`new`](Self::new) and [`track_module`](Self::track_module).) Each owned module is
/// represented by an [`EngineModule`] handle ([`take_initial_loaded_module`](Self::take_initial_loaded_module) for the
/// one the engine was created around), which [`remove_module`](Self::remove_module) turns back
/// into a [`Module`]. A module must be complete when it is handed over: MCJIT compiles it as a
/// whole the first time one of its symbols is requested (or on
/// [`run_static_constructors`](Self::run_static_constructors)) and ignores any later change;
/// add new code as new modules. [`get_function_value`](Self::get_function_value) looks into
/// the owned modules.
///
/// If several modules define the same symbol, MCJIT does not report an error: whichever
/// definition is linked last wins, and which module gets compiled for a lookup is unspecified.
#[derive(Debug)]
pub struct ExecutionEngine<'ctx> {
    execution_engine: NonNull<llvm_sys::execution_engine::LLVMOpaqueExecutionEngine>,
    /// The module the engine was created around, until `initial_module` hands out its handle.
    initial_module: Cell<Option<NonNull<LLVMModule>>>,
    /// How many modules the engine owns, as far as this wrapper knows (see `can_finalize`).
    module_count: Cell<usize>,
    /// Whether MCJIT has loaded at least one object. `finalizeObject` (run by
    /// `LLVMRunStaticConstructors`/`Destructors`) compiles the owned modules and then
    /// dereferences the `RuntimeDyld`, which only exists once an object has been loaded; with
    /// no owned modules and nothing ever loaded it crashes, so that case is skipped.
    has_loaded_object: Cell<bool>,
    _marker: PhantomData<&'ctx Context>,
}

impl PartialEq for ExecutionEngine<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.execution_engine == other.execution_engine
    }
}

impl Eq for ExecutionEngine<'_> {}

impl<'ctx> ExecutionEngine<'ctx> {
    /// Wraps a raw execution engine, taking ownership of it and of the module it was created
    /// around. [`take_initial_loaded_module`](Self::take_initial_loaded_module) hands that
    /// module's handle out; modules added through the raw API afterwards are registered with
    /// [`track_module`](Self::track_module).
    ///
    /// # Safety
    ///
    /// `execution_engine` must be valid and uniquely owned by the caller, created by one of
    /// LLVM's `LLVMCreate*ForModule` functions around `initial_module`, and must own no other
    /// module. The context the module belongs to must outlive the returned value.
    pub unsafe fn new(execution_engine: LLVMExecutionEngineRef, initial_module: LLVMModuleRef) -> Self {
        ExecutionEngine {
            execution_engine: NonNull::new(execution_engine).expect("execution engine must be non-null"),
            initial_module: Cell::new(Some(NonNull::new(initial_module).expect("module must be non-null"))),
            // LLVM cannot create an engine without a module (the C API only has `*ForModule`
            // constructors, and `ExecutionEngine::init` asserts on a null one), so a fresh
            // engine owns exactly the one it was created around.
            module_count: Cell::new(1),
            has_loaded_object: Cell::new(false),
            _marker: PhantomData,
        }
    }

    /// Creates an execution engine of LLVM's default kind -- MCJIT if the target has a JIT,
    /// else the interpreter -- around a raw module. The safe form is
    /// [`Module::create_execution_engine`].
    ///
    /// # Safety
    ///
    /// `module` must be valid and owned by nobody else: the engine owns it from here on, and it
    /// is destroyed if creation fails. The module's context must outlive the returned value.
    pub unsafe fn create_for_module(module: LLVMModuleRef) -> Result<Self, LLVMString> {
        Self::link_in_mc_jit();
        Self::link_in_interpreter();

        unsafe { Self::create(module, |out, err| LLVMCreateExecutionEngineForModule(out, module, err)) }
    }

    /// Creates an interpreter around a raw module. The safe form is
    /// [`Module::create_interpreter_execution_engine`].
    ///
    /// # Safety
    ///
    /// As for [`create_for_module`](Self::create_for_module).
    pub unsafe fn create_interpreter_for_module(module: LLVMModuleRef) -> Result<Self, LLVMString> {
        Self::link_in_interpreter();

        unsafe { Self::create(module, |out, err| LLVMCreateInterpreterForModule(out, module, err)) }
    }

    /// Creates an MCJIT engine around a raw module. The safe form is
    /// [`Module::create_jit_execution_engine`].
    ///
    /// # Safety
    ///
    /// As for [`create_for_module`](Self::create_for_module).
    pub unsafe fn create_jit_for_module(
        module: LLVMModuleRef,
        opt_level: OptimizationLevel,
    ) -> Result<Self, LLVMString> {
        Self::link_in_mc_jit();

        unsafe {
            Self::create(module, |out, err| {
                LLVMCreateJITCompilerForModule(out, module, opt_level as u32, err)
            })
        }
    }

    /// Creates an MCJIT engine around a raw module with a custom memory manager. The safe form,
    /// with the parameters documented, is
    /// [`Module::create_mcjit_execution_engine_with_memory_manager`].
    ///
    /// # Safety
    ///
    /// As for [`create_for_module`](Self::create_for_module).
    pub unsafe fn create_mcjit_for_module_with_memory_manager(
        module: LLVMModuleRef,
        memory_manager: impl McjitMemoryManager + 'static,
        opt_level: OptimizationLevel,
        code_model: CodeModel,
        no_frame_pointer_elim: bool,
        enable_fast_isel: bool,
    ) -> Result<Self, LLVMString> {
        Self::link_in_mc_jit();

        // Box the memory manager into an adapter LLVM calls back into; `destroy_adapter` reclaims
        // it with `Box::from_raw` when the engine is destroyed.
        let adapter = Box::new(MemoryManagerAdapter {
            memory_manager: Box::new(memory_manager),
        });
        let opaque = Box::into_raw(adapter) as *mut c_void;

        let mmgr = unsafe {
            LLVMCreateSimpleMCJITMemoryManager(
                opaque,
                allocate_code_section_adapter,
                allocate_data_section_adapter,
                finalize_memory_adapter,
                Some(destroy_adapter),
            )
        };
        if mmgr.is_null() {
            // LLVM took neither the adapter nor the module; both are still ours.
            unsafe {
                drop(Box::from_raw(opaque as *mut MemoryManagerAdapter));
                LLVMDisposeModule(module);
            }
            return Err(LLVMString::create_from_str(
                "Failed to create SimpleMCJITMemoryManager.\0",
            ));
        }

        let mut options = MaybeUninit::<LLVMMCJITCompilerOptions>::zeroed();
        // `options` is a plain C struct that LLVM fills in with its defaults.
        let mut options = unsafe {
            LLVMInitializeMCJITCompilerOptions(options.as_mut_ptr(), size_of::<LLVMMCJITCompilerOptions>());
            options.assume_init()
        };
        options.OptLevel = opt_level as u32;
        options.CodeModel = code_model.into();
        options.NoFramePointerElim = no_frame_pointer_elim as i32;
        options.EnableFastISel = enable_fast_isel as i32;
        options.MCJMM = mmgr;

        unsafe {
            Self::create(module, |out, err| {
                LLVMCreateMCJITCompilerForModule(out, module, &mut options, size_of::<LLVMMCJITCompilerOptions>(), err)
            })
        }
    }

    /// Shared body of the `create_*_for_module` constructors: `create` is the
    /// `LLVMCreate*ForModule` call, which takes ownership of `module` and destroys it on failure.
    unsafe fn create(
        module: LLVMModuleRef,
        create: impl FnOnce(*mut LLVMExecutionEngineRef, *mut *mut ::libc::c_char) -> LLVMBool,
    ) -> Result<Self, LLVMString> {
        if let Err(mut err_string) = Target::initialize_native(&InitializationConfig::default()) {
            // The module is ours whether or not an engine gets created.
            unsafe { LLVMDisposeModule(module) };
            err_string.push('\0');

            return Err(LLVMString::create_from_str(&err_string));
        }

        let mut execution_engine = MaybeUninit::uninit();
        let mut err_string: *mut ::libc::c_char = ::core::ptr::null_mut();
        if create(execution_engine.as_mut_ptr(), &mut err_string) == 1 {
            return Err(unsafe { LLVMString::new(err_string) });
        }

        // LLVM created the engine around `module` and owns nothing else yet.
        Ok(unsafe { Self::new(execution_engine.assume_init(), module) })
    }

    /// Whether `finalizeObject` has an object to work on: something already loaded, or owned
    /// modules it will compile first.
    fn can_finalize(&self) -> bool {
        self.has_loaded_object.get() || self.module_count.get() > 0
    }

    /// Acquires the underlying raw pointer belonging to this `ExecutionEngine` type.
    pub fn as_mut_ptr(&self) -> LLVMExecutionEngineRef {
        self.execution_engine.as_ptr()
    }

    #[inline]
    pub(crate) fn execution_engine_inner(&self) -> LLVMExecutionEngineRef {
        self.execution_engine.as_ptr()
    }

    /// Registers a module added to this engine through the raw API (`LLVMAddModule` on
    /// [`as_mut_ptr`](Self::as_mut_ptr)) and returns its handle, so that it can be taken back
    /// with [`remove_module`](Self::remove_module).
    ///
    /// # Safety
    ///
    /// `module` must be owned by this engine and not be known to this wrapper already: neither
    /// added with [`add_module`](Self::add_module) nor tracked before, and not the module the
    /// engine was created around (see [`take_initial_loaded_module`](Self::take_initial_loaded_module)). A second
    /// handle would let the module be removed twice, and `LLVMRemoveModule` hands back a module
    /// it does not own.
    pub unsafe fn track_module(&self, module: LLVMModuleRef) -> EngineModule<'_> {
        let module = NonNull::new(module).expect("module must be non-null");

        self.module_count.set(self.module_count.get() + 1);

        EngineModule {
            module,
            engine: self.execution_engine,
            _marker: PhantomData,
        }
    }

    /// Takes the handle of the module this engine was created around (its initial loaded
    /// module), so that it can be removed with [`remove_module`](Self::remove_module) like any
    /// other module.
    ///
    /// The handle can be taken **once**: it is consumed by removal and must be unique, so later
    /// calls return `None`.
    pub fn take_initial_loaded_module(&self) -> Option<EngineModule<'_>> {
        self.initial_module.take().map(|module| EngineModule {
            module,
            engine: self.execution_engine,
            _marker: PhantomData,
        })
    }

    // This function is noop, but required for proper MCJIT initialization and
    // registration. Without it, LTO is free to eliminate it
    fn link_in_mc_jit() {
        unsafe { LLVMLinkInMCJIT() }
    }

    // This function is noop, but required for proper MCJIT initialization and
    // registration. Without it, LTO is free to eliminate it
    fn link_in_interpreter() {
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
    /// let ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    /// ee.add_global_mapping(&extf, sumf as usize);
    ///
    /// let result = unsafe { ee.run_function(f, &[]) }.as_float(&ft);
    ///
    /// assert_eq!(result, 128.);
    /// ```
    pub fn add_global_mapping(&self, value: &dyn AnyValue<'_>, addr: usize) {
        unsafe { LLVMAddGlobalMapping(self.execution_engine_inner(), value.as_value_ref(), addr as *mut _) }
    }

    /// Hands a complete module to the engine, which owns it from now on.
    ///
    /// The returned [`EngineModule`] is the only way to refer to the module afterwards, gives no
    /// access to its contents, and borrows the engine, so it cannot outlive it.
    ///
    /// ```rust,no_run
    /// use inkwell::targets::{InitializationConfig, Target};
    /// use inkwell::context::Context;
    /// use inkwell::OptimizationLevel;
    ///
    /// Target::initialize_native(&InitializationConfig::default()).unwrap();
    ///
    /// let context = Context::create();
    /// let ee = context.create_module("main").create_jit_execution_engine(OptimizationLevel::None).unwrap();
    ///
    /// let helpers = ee.add_module(context.create_module("helpers"));
    /// // ... look up and run functions ...
    /// let helpers = ee.remove_module(helpers).unwrap();
    /// assert_eq!(helpers.get_name().to_str(), Ok("helpers"));
    /// ```
    pub fn add_module(&self, module: Module<'ctx>) -> EngineModule<'_> {
        let module = module.into_raw();

        unsafe { LLVMAddModule(self.execution_engine_inner(), module) }

        // SAFETY: just added to this engine, and not tracked before.
        unsafe { self.track_module(module) }
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
    /// module (the handle came from another engine, which keeps its module; the handle is
    /// returned inside the error), or
    /// [`LLVMError`](RemoveModuleError::LLVMError) if LLVM refuses the removal.
    pub fn remove_module<'a>(&self, module: EngineModule<'a>) -> Result<Module<'ctx>, RemoveModuleError<'a>> {
        // `LLVMRemoveModule` does not check ownership itself: it would hand back a module
        // another engine still owns. The handle borrows its engine, so it cannot be stale, and
        // removal consumes it, so a module cannot be removed twice; only a handle from another
        // live engine has to be refused.
        if module.engine != self.execution_engine {
            return Err(RemoveModuleError::ModuleNotOwned(module));
        }

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

        self.module_count.set(self.module_count.get() - 1);

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
    /// let ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
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
            if self.get_target_machine().is_none() {
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
        // The interpreter has no code to address: `ExecutionEngine::getFunctionAddress`'s
        // default returns 0, which would read as "not found".
        if self.get_target_machine().is_none() {
            return Err(FunctionLookupError::JITNotEnabled);
        }

        let c_string = to_c_str(fn_name);
        let address = unsafe { LLVMGetFunctionAddress(self.execution_engine_inner(), c_string.as_ptr()) };

        // REVIEW: Can also return 0 if no targets are initialized.
        // One option might be to set a (thread local?) global to true if any at all of the targets have been
        // initialized (maybe we could figure out which config in particular is the trigger)
        // and if not return an "NoTargetsInitialized" error, instead of not found.
        if address == 0 {
            return Err(FunctionLookupError::FunctionNotFound);
        }

        // A found symbol means its module's object has been loaded.
        self.has_loaded_object.set(true);

        Ok(address as usize)
    }

    /// MCJIT's target machine, borrowed from the engine; `None` for the interpreter, which has
    /// none (`ExecutionEngine::getTargetMachine` returns null and only `MCJIT` overrides it).
    /// This is also how to tell which engine [`create_for_module`](Self::create_for_module)
    /// built, and what [`get_function`](Self::get_function) checks before looking a symbol up.
    pub fn get_target_machine(&self) -> Option<TargetMachineRef<'_>> {
        let target_machine = unsafe { LLVMGetExecutionEngineTargetMachine(self.execution_engine_inner()) };

        if target_machine.is_null() {
            None
        } else {
            Some(unsafe { TargetMachineRef::new(target_machine) })
        }
    }

    /// The engine's data layout, borrowed from it: `LLVMGetExecutionEngineTargetData` returns a
    /// pointer to the engine's own `DataLayout` member, which is immutable and lives exactly as
    /// long as the engine.
    pub fn get_data_layout(&self) -> DataLayoutRef<'_> {
        unsafe { DataLayoutRef::new(LLVMGetExecutionEngineTargetData(self.execution_engine_inner())) }
    }

    // REVIEW: Can also find nothing if no targeting is initialized. Maybe best to
    // do have a global flag for anything initialized. Catch is that it must be initialized
    // before EE is created
    // REVIEW: Should FunctionValue lifetime be tied to self not 'ctx?
    pub fn get_function_value(&self, fn_name: &str) -> Result<FunctionValue<'ctx>, FunctionLookupError> {
        // `LLVMFindFunction` searches the IR of the owned modules, so it works for the
        // interpreter too -- which needs it, having no other way to name a function to run.
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
    pub unsafe fn run_function(&self, function: FunctionValue<'_>, args: &[&GenericValue<'ctx>]) -> GenericValue<'ctx> {
        unsafe {
            let mut args: Vec<LLVMGenericValueRef> = args.iter().map(|val| val.generic_value.as_ptr()).collect();

            let value = LLVMRunFunction(
                self.execution_engine_inner(),
                function.as_value_ref(),
                args.len() as u32,
                args.as_mut_ptr(),
            ); // REVIEW: usize to u32 ok??

            // Running a function compiles and loads its module.
            self.has_loaded_object.set(true);

            GenericValue::new(value)
        }
    }

    // TODOC: Marked as unsafe because input function could very well do something unsafe. It's up to the caller
    // to ensure that doesn't happen by defining their function correctly.
    // SubType: Only for JIT EEs?
    pub unsafe fn run_function_as_main(&self, function: FunctionValue<'_>, args: &[&str]) -> c_int {
        unsafe {
            let cstring_args: Vec<_> = args.iter().map(|&arg| to_c_str(arg)).collect();
            let raw_args: Vec<*const _> = cstring_args.iter().map(|arg| arg.as_ptr()).collect();

            let environment_variables = []; // TODO: Support envp. Likely needs to be null terminated

            let code = LLVMRunFunctionAsMain(
                self.execution_engine_inner(),
                function.as_value_ref(),
                raw_args.len() as u32,
                raw_args.as_ptr(),
                environment_variables.as_ptr(),
            ); // REVIEW: usize to u32 cast ok??

            // Running a function compiles and loads its module.
            self.has_loaded_object.set(true);

            code
        }
    }

    pub fn free_fn_machine_code(&self, function: FunctionValue<'_>) {
        unsafe { LLVMFreeMachineCodeForFunction(self.execution_engine_inner(), function.as_value_ref()) }
    }

    /// Compiles every module the engine owns that is not compiled yet, then runs the static
    /// constructors of the owned modules. Does nothing if the engine owns no module and has
    /// never compiled one.
    // REVIEW: Is this actually safe?
    pub fn run_static_constructors(&self) {
        if !self.can_finalize() {
            return;
        }

        unsafe { LLVMRunStaticConstructors(self.execution_engine_inner()) }

        self.has_loaded_object.set(true);
    }

    /// Compiles every module the engine owns that is not compiled yet, then runs the static
    /// destructors of the owned modules. Does nothing if the engine owns no module and has
    /// never compiled one.
    // REVIEW: Is this actually safe? Can you double destruct/free?
    pub fn run_static_destructors(&self) {
        if !self.can_finalize() {
            return;
        }

        unsafe { LLVMRunStaticDestructors(self.execution_engine_inner()) }

        self.has_loaded_object.set(true);
    }
}

impl Drop for ExecutionEngine<'_> {
    fn drop(&mut self) {
        // Disposing the engine destroys every module it still owns and frees all generated
        // code. `JitFunction`s, `EngineModule`s and `DataLayoutRef`s borrow `self`, so none can
        // be alive here.
        unsafe { LLVMDisposeExecutionEngine(self.execution_engine.as_ptr()) }
    }
}

/// A module owned by an [`ExecutionEngine`], as handed out by [`ExecutionEngine::add_module`],
/// [`ExecutionEngine::take_initial_loaded_module`] and [`ExecutionEngine::track_module`].
///
/// The handle is the only way to refer to the module afterwards: pass it to
/// [`ExecutionEngine::remove_module`] to take the module back. It deliberately gives no access
/// to the module's contents -- MCJIT compiles a module as a whole and ignores later changes, so
/// a module must be complete before it is handed over;
/// [`ExecutionEngine::get_function_value`] looks up its functions.
///
/// The handle borrows its engine, so it cannot outlive it, and removal consumes it, so a module
/// cannot be removed twice; all the engine has left to check is that the handle is its own.
/// That is also why it is not a `Copy` view like `DataLayoutRef`: a copy left behind would
/// point at the returned, user-owned module. Outliving the engine is a compile error:
///
/// ```compile_fail,E0505
/// use inkwell::context::Context;
/// use inkwell::OptimizationLevel;
///
/// let context = Context::create();
/// let ee = context.create_module("main").create_jit_execution_engine(OptimizationLevel::None).unwrap();
/// let helpers = ee.add_module(context.create_module("helpers"));
/// drop(ee); // error: `ee` is borrowed by `helpers`
/// let _ = helpers.as_mut_ptr();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct EngineModule<'ee> {
    module: NonNull<LLVMModule>,
    /// Identity of the owning engine, for `remove_module`'s check.
    engine: NonNull<llvm_sys::execution_engine::LLVMOpaqueExecutionEngine>,
    _marker: PhantomData<&'ee ()>,
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
