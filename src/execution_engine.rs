use llvm_sys::core::LLVMDisposeMessage;
use llvm_sys::execution_engine::{LLVMGetExecutionEngineTargetData, LLVMExecutionEngineRef, LLVMRunFunction, LLVMRunFunctionAsMain, LLVMDisposeExecutionEngine, LLVMGetFunctionAddress, LLVMAddModule, LLVMFindFunction, LLVMLinkInMCJIT, LLVMLinkInInterpreter, LLVMRemoveModule, LLVMGenericValueRef, LLVMFreeMachineCodeForFunction, LLVMGetPointerToGlobal, LLVMAddGlobalMapping};

use module::Module;
use targets::TargetData;
use values::{AnyValue, AsValueRef, FunctionValue, GenericValue, GlobalValue};

use std::ffi::{CStr, CString};
use std::mem::{forget, uninitialized, zeroed};

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionLookupError {
    JITNotEnabled,
    FunctionNotFound, // 404!
}

#[derive(Debug)]
pub struct ExecutionEngine {
    execution_engine: LLVMExecutionEngineRef,
    pub(crate) modules: Vec<Module>,
    target_data: Option<TargetData>,
    jit_mode: bool,
}

impl ExecutionEngine {
    pub(crate) fn new(execution_engine: LLVMExecutionEngineRef, jit_mode: bool) -> ExecutionEngine {
        assert!(!execution_engine.is_null());

        // REVIEW: Will we have to do this for LLVMGetExecutionEngineTargetMachine too?
        let target_data = unsafe {
            LLVMGetExecutionEngineTargetData(execution_engine)
        };

        ExecutionEngine {
            execution_engine: execution_engine,
            modules: vec![],
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
    /// ```
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
    /// ee.add_global_mapping(&extf, sumf as *mut _);
    ///
    /// let result = unsafe { ee.run_function(&f, &[]) }.as_float(&ft);
    ///
    /// assert_eq!(result, 128.);
    /// ```
    pub fn add_global_mapping(&mut self, value: &AnyValue, addr: *const ()) {
        unsafe {
            LLVMAddGlobalMapping(self.execution_engine, value.as_value_ref(), addr as *mut _)
        }
    }

    // TODOC: EE must *own* modules and deal out references
    pub fn add_module(&mut self, module: Module) -> &Module {
        unsafe {
            LLVMAddModule(self.execution_engine, module.module)
        }

        self.modules.push(module);

        &self.modules[self.modules.len() - 1]
    }

    pub fn remove_module(&mut self, module: &Module) -> Result<Module, String> {
        let mut new_module = unsafe { uninitialized() };
        let mut err_str = unsafe { zeroed() };

        let code = unsafe {
            LLVMRemoveModule(self.execution_engine, module.module, &mut new_module, &mut err_str)
        };

        if code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(err_str);

                rust_str
            };

            return Err(rust_str);
        }

        // REVIEW: This might end up a hashtable for better performance
        let mut index = None;

        for (i, owned_module) in self.modules.iter().enumerate() {
            if module == owned_module {
                index = Some(i);
            }
        }

        match index {
            Some(idx) => self.modules.remove(idx),
            None => return Err("Module does not belong to this ExecutionEngine".into()),
        };

        Ok(Module::new(new_module))
    }

    // FIXME: Workaround until we can think of a better API
    // Ideally, we'd provide this as a hashmap from module name to module, but this may not be
    // possible since you can create modules, ie via create_module_from_ir, which either do not
    // have a name, or at least LLVM provides no way to obtain its name. Also, possible collisions
    // depending on how loose, if at all, LLVM is with module names belonging to execution engines
    pub fn get_module_at(&self, index: usize) -> &Module {
        &self.modules[index]
    }

    /// WARNING: The returned address *will* be invalid if the EE drops first
    /// Do not attempt to transmute it to a function if the ExecutionEngine is gone
    // TODOC: Initializing a target MUST occur before creating the EE or else it will not count
    // TODOC: Can still add functions after EE has been created
    pub fn get_function_address(&self, fn_name: &str) -> Result<u64, FunctionLookupError> {
        if !self.jit_mode {
            return Err(FunctionLookupError::JITNotEnabled);
        }

        let c_string = CString::new(fn_name).expect("Conversion to CString failed unexpectedly");

        let address = unsafe {
            LLVMGetFunctionAddress(self.execution_engine, c_string.as_ptr())
        };

        // REVIEW: Can also return 0 if no targets are initialized.
        // One option might be to set a global to true if any at all of the targets have been
        // initialized (maybe we could figure out which config in particular is the trigger)
        // and if not return an "NoTargetsInitialized" error, instead of not found.
        if address == 0 {
            return Err(FunctionLookupError::FunctionNotFound);
        }

        Ok(address)
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
            LLVMFindFunction(self.execution_engine, c_string.as_ptr(), &mut function)
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

        let value = LLVMRunFunction(self.execution_engine, function.as_value_ref(), args.len() as u32, args.as_mut_ptr()); // REVIEW: usize to u32 ok??

        GenericValue::new(value)
    }

    pub fn run_function_as_main(&self, function: &FunctionValue) {
        let args = vec![]; // TODO: Support argc, argv
        let env_p = vec![]; // REVIEW: No clue what this is

        unsafe {
            LLVMRunFunctionAsMain(self.execution_engine, function.as_value_ref(), args.len() as u32, args.as_ptr(), env_p.as_ptr()); // REVIEW: usize to u32 cast ok??
        }
    }

    pub fn free_fn_machine_code(&self, function: &FunctionValue) {
        unsafe {
            LLVMFreeMachineCodeForFunction(self.execution_engine, function.as_value_ref())
        }
    }
}

// Modules owned by the EE will be discarded by the EE so we don't
// want owned modules to drop.
impl Drop for ExecutionEngine {
    fn drop(&mut self) {
        for module in self.modules.drain(..) {
            forget(module);
        }

        forget(self.target_data.take().expect("TargetData should always exist until Drop"));

        unsafe {
            LLVMDisposeExecutionEngine(self.execution_engine);
        }
    }
}
