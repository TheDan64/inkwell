use llvm_sys::core::LLVMDisposeMessage;
use llvm_sys::execution_engine::{LLVMGetExecutionEngineTargetData, LLVMExecutionEngineRef, LLVMRunFunction, LLVMRunFunctionAsMain, LLVMDisposeExecutionEngine, LLVMGetFunctionAddress, LLVMAddModule, LLVMFindFunction, LLVMLinkInMCJIT, LLVMLinkInInterpreter, LLVMRemoveModule, LLVMGenericValueRef, LLVMFreeMachineCodeForFunction, LLVMAddGlobalMapping};

use module::Module;
use targets::TargetData;
use values::{AnyValue, AsValueRef, FunctionValue, GenericValue};

use std::rc::Rc;
use std::ops::Deref;
use std::ffi::{CStr, CString};
use std::mem::{forget, uninitialized, zeroed, transmute_copy, size_of};
use std::fmt::{self, Debug, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionLookupError {
    JITNotEnabled,
    FunctionNotFound, // 404!
}

#[derive(PartialEq, Eq, Debug)]
pub struct ExecutionEngine {
    pub(crate) execution_engine: Rc<LLVMExecutionEngineRef>,
    target_data: Option<TargetData>,
    jit_mode: bool,
}

impl ExecutionEngine {
    pub(crate) fn new(execution_engine: Rc<LLVMExecutionEngineRef>, jit_mode: bool) -> ExecutionEngine {
        assert!(!execution_engine.is_null());

        // REVIEW: Will we have to do this for LLVMGetExecutionEngineTargetMachine too?
        let target_data = unsafe {
            LLVMGetExecutionEngineTargetData(*execution_engine)
        };

        ExecutionEngine {
            execution_engine: execution_engine,
            target_data: Some(TargetData::new(target_data)),
            jit_mode: jit_mode,
        }
    }

    pub fn link_in_mc_jit() {
        unsafe {
            LLVMLinkInMCJIT()
        }
    }

    pub fn link_in_interpreter() {
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
    /// let f = module.add_function("test_fn", &fnt, None);
    /// let b = context.append_basic_block(&f, "entry");
    /// builder.position_at_end(&b);
    ///
    /// let extf = module.add_function("sumf", &ft.fn_type(&[ &ft, &ft ], false), None);
    ///
    /// let argf = ft.const_float(64.);
    /// let retv = builder.build_call(&extf, &[ &argf, &argf ], "retv", false).left().unwrap().into_float_value();
    ///
    /// builder.build_return(Some(&retv));
    ///
    /// let mut ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    /// ee.add_global_mapping(&extf, sumf as usize);
    ///
    /// let result = unsafe { ee.run_function(&f, &[]) }.as_float(&ft);
    ///
    /// assert_eq!(result, 128.);
    /// ```
    pub fn add_global_mapping(&self, value: &AnyValue, addr: usize) {
        unsafe {
            LLVMAddGlobalMapping(*self.execution_engine, value.as_value_ref(), addr as *mut _)
        }
    }

    /// Adds a module to an `ExecutionEngine`.
    ///
    /// The method will be `Ok(())` if the module does not belong to an `ExecutionEngine` already and `Err(())` otherwise.
    ///
    /// ```rust,no_run
    /// use inkwell::targets::{InitializationConfig, Target};
    /// use inkwell::context::Context;
    /// use inkwell::OptimizationLevel;
    ///
    /// Target::initialize_native(&InitializationConfig::default()).unwrap();
    ///
    /// let context = Context::create();
    /// let module = context.create_module("test");
    /// let mut ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    ///
    /// assert!(ee.add_module(&module).is_err());
    /// ```
    pub fn add_module(&self, module: &Module) -> Result<(), ()> {
        unsafe {
            LLVMAddModule(*self.execution_engine, module.module.get())
        }

        if module.owned_by_ee.borrow().is_some() {
            return Err(());
        }

        *module.owned_by_ee.borrow_mut() = Some(ExecutionEngine::new(self.execution_engine.clone(), self.jit_mode));

        Ok(())
    }

    pub fn remove_module(&self, module: &Module) -> Result<(), String> {
        match *module.owned_by_ee.borrow() {
            Some(ref ee) if *ee.execution_engine != *self.execution_engine => return Err("Module is not owned by this Execution Engine".into()),
            None => return Err("Module is not owned by an Execution Engine".into()),
            _ => ()
        }

        let mut new_module = unsafe { uninitialized() };
        let mut err_str = unsafe { zeroed() };

        let code = unsafe {
            LLVMRemoveModule(*self.execution_engine, module.module.get(), &mut new_module, &mut err_str)
        };

        if code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(err_str);

                rust_str
            };

            return Err(rust_str);
        }

        module.module.set(new_module);
        *module.owned_by_ee.borrow_mut() = None;

        Ok(())
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
    /// is `unsafe`.
    /// 
    /// # Examples
    /// 
    /// 
    /// ```rust
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
    /// let f = module.add_function("test_fn", &sig, None);
    /// let b = context.append_basic_block(&f, "entry");
    /// builder.position_at_end(&b);
    ///
    /// // Insert a return statement
    /// let ret = double.const_float(64.0);
    /// builder.build_return(Some(&ret));
    ///
    /// // create the JIT engine
    /// let mut ee = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    /// 
    /// // fetch our JIT'd function and execute it
    /// unsafe {
    ///     let test_fn = ee.get_function::<unsafe extern "C" fn() -> f64>("test_fn").unwrap();
    ///     let return_value = test_fn();
    ///     assert_eq!(return_value, 64.0);
    /// }
    /// ```
    /// 
    /// # Safety
    /// 
    /// It is the caller's responsibility to ensure they call the function with
    /// the correct signature and calling convention.
    /// 
    /// The `Symbol` wrapper ensures a function won't accidentally outlive the
    /// execution engine it came from, but adding functions after calling this
    /// method *may* invalidate the function pointer.
    /// 
    /// [`UnsafeFunctionPointer`]: trait.UnsafeFunctionPointer.html
    pub unsafe fn get_function<F>(&self, fn_name: &str) -> Result<Symbol<F>, FunctionLookupError>
    where F: UnsafeFunctionPointer 
    {
        if !self.jit_mode {
            return Err(FunctionLookupError::JITNotEnabled);
        }

        let c_string = CString::new(fn_name).expect("Conversion to CString failed unexpectedly");

        let address = LLVMGetFunctionAddress(*self.execution_engine, c_string.as_ptr());

        // REVIEW: Can also return 0 if no targets are initialized.
        // One option might be to set a global to true if any at all of the targets have been
        // initialized (maybe we could figure out which config in particular is the trigger)
        // and if not return an "NoTargetsInitialized" error, instead of not found.
        if address == 0 {
            return Err(FunctionLookupError::FunctionNotFound);
        }

        assert_eq!(size_of::<F>(), size_of::<usize>(), 
            "The type `F` must have the same size as a function pointer");

        Ok(Symbol {
            execution_engine: self.execution_engine.clone(),
            inner: transmute_copy(&address),
        })
    }

    // REVIEW: Not sure if an EE's target data can change.. if so we might want to update the value
    // when making this call
    pub fn get_target_data(&self) -> &TargetData {
        self.target_data.as_ref().expect("TargetData should always exist until Drop")
    }

    // REVIEW: Can also find nothing if no targeting is initialized. Maybe best to
    // do have a global flag for anything initialized. Catch is that it must be initialized
    // before EE is created
    pub fn get_function_value(&self, fn_name: &str) -> Result<FunctionValue, FunctionLookupError> {
        if !self.jit_mode {
            return Err(FunctionLookupError::JITNotEnabled);
        }

        let c_string = CString::new(fn_name).expect("Conversion to CString failed unexpectedly");
        let mut function = unsafe { zeroed() };

        let code = unsafe {
            LLVMFindFunction(*self.execution_engine, c_string.as_ptr(), &mut function)
        };

        if code == 0 {
            return FunctionValue::new(function).ok_or(FunctionLookupError::FunctionNotFound)
        };

        Err(FunctionLookupError::FunctionNotFound)
    }

    // TODOC: Marked as unsafe because input function could very well do something unsafe. It's up to the caller
    // to ensure that doesn't happen by defining their function correctly.
    pub unsafe fn run_function(&self, function: &FunctionValue, args: &[&GenericValue]) -> GenericValue {
        let mut args: Vec<LLVMGenericValueRef> = args.iter()
                                                     .map(|val| val.generic_value)
                                                     .collect();

        let value = LLVMRunFunction(*self.execution_engine, function.as_value_ref(), args.len() as u32, args.as_mut_ptr()); // REVIEW: usize to u32 ok??

        GenericValue::new(value)
    }

    pub fn run_function_as_main(&self, function: &FunctionValue) {
        let args = vec![]; // TODO: Support argc, argv
        let env_p = vec![]; // REVIEW: No clue what this is

        unsafe {
            LLVMRunFunctionAsMain(*self.execution_engine, function.as_value_ref(), args.len() as u32, args.as_ptr(), env_p.as_ptr()); // REVIEW: usize to u32 cast ok??
        }
    }

    pub fn free_fn_machine_code(&self, function: &FunctionValue) {
        unsafe {
            LLVMFreeMachineCodeForFunction(*self.execution_engine, function.as_value_ref())
        }
    }
}

// Modules owned by the EE will be discarded by the EE so we don't
// want owned modules to drop.
impl Drop for ExecutionEngine {
    fn drop(&mut self) {
        forget(self.target_data.take().expect("TargetData should always exist until Drop"));

        if Rc::strong_count(&self.execution_engine) == 1 {
            unsafe {
                LLVMDisposeExecutionEngine(*self.execution_engine);
            }
        }
    }
}

/// A wrapper around a function pointer which ensures the symbol being pointed
/// to doesn't accidentally outlive its execution engine.
#[derive(Clone)]
pub struct Symbol<F> {
    pub(crate) execution_engine: Rc<LLVMExecutionEngineRef>,
    inner: F,
}

impl<F: UnsafeFunctionPointer> Deref for Symbol<F> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<F> Debug for Symbol<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_tuple("Symbol")
            .field(&"<unnamed>")
            .finish()
    }
}

/// Marker trait representing an unsafe function pointer (`unsafe extern "C" fn(A, B, ...) -> Output`).
pub trait UnsafeFunctionPointer: private::Sealed + Copy {}

mod private {
    /// A sealed trait which ensures nobody outside this crate can implement
    /// `UnsafeFunctionPointer`.
    /// 
    /// See https://rust-lang-nursery.github.io/api-guidelines/future-proofing.html
    pub trait Sealed {}
}

macro_rules! impl_unsafe_fn {
    ($( $param:ident ),*) => {
        impl<Output, $( $param ),*> private::Sealed for unsafe extern "C" fn($( $param ),*) -> Output {}
        impl<Output, $( $param ),*> UnsafeFunctionPointer for unsafe extern "C" fn($( $param ),*) -> Output {}
    };
}

impl_unsafe_fn!();
impl_unsafe_fn!(A);
impl_unsafe_fn!(A, B);
impl_unsafe_fn!(A, B, C);
impl_unsafe_fn!(A, B, C, D);
impl_unsafe_fn!(A, B, C, D, E);
impl_unsafe_fn!(A, B, C, D, E, F);
impl_unsafe_fn!(A, B, C, D, E, F, G);
impl_unsafe_fn!(A, B, C, D, E, F, G, H);
impl_unsafe_fn!(A, B, C, D, E, F, G, H, I);
impl_unsafe_fn!(A, B, C, D, E, F, G, H, I, J);
impl_unsafe_fn!(A, B, C, D, E, F, G, H, I, J, K);
impl_unsafe_fn!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_unsafe_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M);

