use llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction};
use llvm_sys::bit_writer::{LLVMWriteBitcodeToFile, LLVMWriteBitcodeToMemoryBuffer};
use llvm_sys::core::{LLVMAddFunction, LLVMAddGlobal, LLVMDisposeMessage, LLVMDumpModule, LLVMGetNamedFunction, LLVMGetTypeByName, LLVMSetDataLayout, LLVMSetTarget, LLVMCloneModule, LLVMDisposeModule, LLVMGetTarget, LLVMGetDataLayout, LLVMModuleCreateWithName, LLVMGetModuleContext, LLVMGetFirstFunction, LLVMGetLastFunction, LLVMSetLinkage, LLVMAddGlobalInAddressSpace, LLVMPrintModuleToString, LLVMGetNamedMetadataNumOperands, LLVMAddNamedMetadataOperand, LLVMGetNamedMetadataOperands, LLVMGetFirstGlobal, LLVMGetLastGlobal, LLVMGetNamedGlobal, LLVMPrintModuleToFile, LLVMSetModuleInlineAsm};
use llvm_sys::execution_engine::LLVMCreateJITCompilerForModule;
use llvm_sys::prelude::{LLVMValueRef, LLVMModuleRef};
use llvm_sys::LLVMLinkage;

use std::cell::{Cell, RefCell};
use std::ffi::{CString, CStr};
use std::fs::File;
use std::mem::{forget, uninitialized, zeroed};
use std::path::Path;
use std::rc::Rc;
use std::slice::from_raw_parts;

use {AddressSpace, OptimizationLevel};
use context::{Context, ContextRef};
use data_layout::DataLayout;
use execution_engine::ExecutionEngine;
use memory_buffer::MemoryBuffer;
use types::{AsTypeRef, BasicType, FunctionType, BasicTypeEnum};
use values::{AsValueRef, FunctionValue, GlobalValue, MetadataValue};

// REVIEW: Maybe this should go into it's own module?
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Linkage {
    AppendingLinkage,
    AvailableExternallyLinkage,
    CommonLinkage,
    DLLExportLinkage,
    DLLImportLinkage,
    ExternalLinkage,
    ExternalWeakLinkage,
    GhostLinkage,
    InternalLinkage,
    LinkerPrivateLinkage,
    LinkerPrivateWeakLinkage,
    LinkOnceAnyLinkage,
    LinkOnceODRAutoHideLinkage,
    LinkOnceODRLinkage,
    PrivateLinkage,
    WeakAnyLinkage,
    WeakODRLinkage,
}

impl Linkage {
    pub(crate) fn new(linkage: LLVMLinkage) -> Self {
        match linkage {
            LLVMLinkage::LLVMAppendingLinkage => Linkage::AppendingLinkage,
            LLVMLinkage::LLVMAvailableExternallyLinkage => Linkage::AvailableExternallyLinkage,
            LLVMLinkage::LLVMCommonLinkage => Linkage::CommonLinkage,
            LLVMLinkage::LLVMDLLExportLinkage => Linkage::DLLExportLinkage,
            LLVMLinkage::LLVMDLLImportLinkage => Linkage::DLLImportLinkage,
            LLVMLinkage::LLVMExternalLinkage => Linkage::ExternalLinkage,
            LLVMLinkage::LLVMExternalWeakLinkage => Linkage::ExternalWeakLinkage,
            LLVMLinkage::LLVMGhostLinkage => Linkage::GhostLinkage,
            LLVMLinkage::LLVMInternalLinkage => Linkage::InternalLinkage,
            LLVMLinkage::LLVMLinkerPrivateLinkage => Linkage::LinkerPrivateLinkage,
            LLVMLinkage::LLVMLinkerPrivateWeakLinkage => Linkage::LinkerPrivateWeakLinkage,
            LLVMLinkage::LLVMLinkOnceAnyLinkage => Linkage::LinkOnceAnyLinkage,
            LLVMLinkage::LLVMLinkOnceODRAutoHideLinkage => Linkage::LinkOnceODRAutoHideLinkage,
            LLVMLinkage::LLVMLinkOnceODRLinkage => Linkage::LinkOnceODRLinkage,
            LLVMLinkage::LLVMPrivateLinkage => Linkage::PrivateLinkage,
            LLVMLinkage::LLVMWeakAnyLinkage => Linkage::WeakAnyLinkage,
            LLVMLinkage::LLVMWeakODRLinkage => Linkage::WeakODRLinkage,
        }
    }

    fn as_llvm_linkage(&self) -> LLVMLinkage {
        match *self {
            Linkage::AppendingLinkage => LLVMLinkage::LLVMAppendingLinkage,
            Linkage::AvailableExternallyLinkage => LLVMLinkage::LLVMAvailableExternallyLinkage,
            Linkage::CommonLinkage => LLVMLinkage::LLVMCommonLinkage,
            Linkage::DLLExportLinkage => LLVMLinkage::LLVMDLLExportLinkage,
            Linkage::DLLImportLinkage => LLVMLinkage::LLVMDLLImportLinkage,
            Linkage::ExternalLinkage => LLVMLinkage::LLVMExternalLinkage,
            Linkage::ExternalWeakLinkage => LLVMLinkage::LLVMExternalWeakLinkage,
            Linkage::GhostLinkage => LLVMLinkage::LLVMGhostLinkage,
            Linkage::InternalLinkage => LLVMLinkage::LLVMInternalLinkage,
            Linkage::LinkerPrivateLinkage => LLVMLinkage::LLVMLinkerPrivateLinkage,
            Linkage::LinkerPrivateWeakLinkage => LLVMLinkage::LLVMLinkerPrivateWeakLinkage,
            Linkage::LinkOnceAnyLinkage => LLVMLinkage::LLVMLinkOnceAnyLinkage,
            Linkage::LinkOnceODRAutoHideLinkage => LLVMLinkage::LLVMLinkOnceODRAutoHideLinkage,
            Linkage::LinkOnceODRLinkage => LLVMLinkage::LLVMLinkOnceODRLinkage,
            Linkage::PrivateLinkage => LLVMLinkage::LLVMPrivateLinkage,
            Linkage::WeakAnyLinkage => LLVMLinkage::LLVMWeakAnyLinkage,
            Linkage::WeakODRLinkage => LLVMLinkage::LLVMWeakODRLinkage,
        }
    }
}

/// Represents a reference to an LLVM `Module`.
/// The underlying module will be disposed when dropping this object.
#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub(crate) non_global_context: Option<Context>,
    data_layout: Option<DataLayout>,
    pub(crate) module: Cell<LLVMModuleRef>,
    pub(crate) owned_by_ee: RefCell<Option<ExecutionEngine>>,
}

impl Module {
    pub(crate) fn new(module: LLVMModuleRef, context: Option<&Context>) -> Self {
        assert!(!module.is_null());

        let mut module = Module {
            module: Cell::new(module),
            non_global_context: context.map(|ctx| Context::new(ctx.context.clone())),
            owned_by_ee: RefCell::new(None),
            data_layout: None,
        };

        module.data_layout = Some(DataLayout::new(module.get_raw_data_layout()));
        module
    }

    /// Creates a named `Module`. Will be automatically assigned the global context.
    ///
    /// To use your own `Context`, see [inkwell::context::create_module()](../context/struct.Context.html#method.create_module)
    ///
    /// # Example
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    ///
    /// let context = Context::get_global();
    /// let module = Module::create("my_module");
    ///
    /// assert_eq!(module.get_context(), context);
    /// ```
    pub fn create(name: &str) -> Self {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let module = unsafe {
            LLVMModuleCreateWithName(c_string.as_ptr())
        };

        Module::new(module, None)
    }

    /// Creates a function given its `name` and `ty`, adds it to the `Module`
    /// and returns it.
    ///
    /// An optional `linkage` can be specified, without which the default value
    /// `Linkage::ExternalLinkage` will be used.
    ///
    /// # Example
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::module::{Module, Linkage};
    ///
    /// let context = Context::get_global();
    /// let module = Module::create("my_module");
    ///
    /// let fn_type = context.f32_type().fn_type(&[], false);
    /// let fn_val = module.add_function("my_function", &fn_type, None);
    ///
    /// assert_eq!(fn_val.get_name().to_str(), Ok("my_function"));
    /// assert_eq!(fn_val.get_linkage(), Linkage::ExternalLinkage);
    /// ```
    pub fn add_function(&self, name: &str, ty: &FunctionType, linkage: Option<&Linkage>) -> FunctionValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMAddFunction(self.module.get(), c_string.as_ptr(), ty.as_type_ref())
        };

        if let Some(linkage) = linkage {
            unsafe {
                LLVMSetLinkage(value, linkage.as_llvm_linkage());
            }
        }

        FunctionValue::new(value).expect("add_function should always succeed in adding a new function")
    }

    /// Gets the `Context` from which this `Module` originates.
    ///
    /// # Example
    /// ```
    /// use inkwell::context::{Context, ContextRef};
    /// use inkwell::module::Module;
    ///
    /// let global_context = Context::get_global();
    /// let global_module = Module::create("my_global_module");
    ///
    /// assert_eq!(global_module.get_context(), global_context);
    ///
    /// let local_context = Context::create();
    /// let local_module = local_context.create_module("my_module");
    ///
    /// assert_eq!(*local_module.get_context(), local_context);
    /// assert_ne!(local_context, *global_context);
    /// ```
    pub fn get_context(&self) -> ContextRef {
        let context = unsafe {
            LLVMGetModuleContext(self.module.get())
        };

        // REVIEW: This probably should be somehow using the existing context Rc
        ContextRef::new(Context::new(Rc::new(context)))
    }

    /// Gets the first `FunctionValue` defined in this `Module`.
    ///
    /// # Example
    /// ```rust,no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    ///
    /// assert!(module.get_first_function().is_none());
    ///
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", &fn_type, None);
    ///
    /// assert_eq!(fn_value, module.get_first_function().unwrap());
    /// ```
    pub fn get_first_function(&self) -> Option<FunctionValue> {
        let function = unsafe {
            LLVMGetFirstFunction(self.module.get())
        };

        FunctionValue::new(function)
    }

    /// Gets the last `FunctionValue` defined in this `Module`.
    ///
    /// # Example
    /// ```rust,no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    ///
    /// assert!(module.get_last_function().is_none());
    ///
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", &fn_type, None);
    ///
    /// assert_eq!(fn_value, module.get_last_function().unwrap());
    /// ```
    pub fn get_last_function(&self) -> Option<FunctionValue> {
        let function = unsafe {
            LLVMGetLastFunction(self.module.get())
        };

        FunctionValue::new(function)
    }

    /// Gets a `FunctionValue` defined in this `Module` by its name.
    ///
    /// # Example
    /// ```rust,no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    ///
    /// assert!(module.get_function("my_fn").is_none());
    ///
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", &fn_type, None);
    ///
    /// assert_eq!(fn_value, module.get_function("my_fn").unwrap());
    /// ```
    pub fn get_function(&self, name: &str) -> Option<FunctionValue> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMGetNamedFunction(self.module.get(), c_string.as_ptr())
        };

        FunctionValue::new(value)
    }

    pub fn get_type(&self, name: &str) -> Option<BasicTypeEnum> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let type_ = unsafe {
            LLVMGetTypeByName(self.module.get(), c_string.as_ptr())
        };

        if type_.is_null() {
            return None;
        }

        Some(BasicTypeEnum::new(type_))
    }

    // TODO: Make this take a targets::Target object by ref and call get_name
    pub fn set_target(&self, target_triple: &str) {
        let c_string = CString::new(target_triple).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetTarget(self.module.get(), c_string.as_ptr())
        }
    }

    // REVIEW: Can we link this to Target.from_name or Target.from_triple / etc?
    pub fn get_target(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTarget(self.module.get()))
        }
    }

    /// Creates a JIT `ExecutionEngine` from this `Module`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::OptimizationLevel;
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::targets::{InitializationConfig, Target};
    ///
    /// Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");
    ///
    /// let context = Context::get_global();
    /// let module = Module::create("my_module");
    /// let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    ///
    /// assert_eq!(module.get_context(), context);
    /// ```
    pub fn create_jit_execution_engine(&self, opt_level: OptimizationLevel) -> Result<ExecutionEngine, String> {
        let mut execution_engine = unsafe { uninitialized() };
        let mut err_str = unsafe { zeroed() };

        let code = unsafe {
            LLVMCreateJITCompilerForModule(&mut execution_engine, self.module.get(), opt_level as u32, &mut err_str) // Should take ownership of module
        };

        if code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(err_str);

                rust_str
            };

            return Err(rust_str);
        }

        let execution_engine = ExecutionEngine::new(Rc::new(execution_engine), true);

        *self.owned_by_ee.borrow_mut() = Some(ExecutionEngine::new(execution_engine.execution_engine.clone(), true));

        Ok(execution_engine)
    }

    pub fn add_global(&self, type_: &BasicType, address_space: Option<AddressSpace>, name: &str) -> GlobalValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            match address_space {
                Some(address_space) => LLVMAddGlobalInAddressSpace(self.module.get(), type_.as_type_ref(), c_string.as_ptr(), address_space as u32),
                None => LLVMAddGlobal(self.module.get(), type_.as_type_ref(), c_string.as_ptr()),
            }
        };

        GlobalValue::new(value)
    }

    pub fn write_bitcode_to_path(&self, path: &Path) -> bool {
        let path_str = path.to_str().expect("Did not find a valid Unicode path string");
        let c_string = CString::new(path_str).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMWriteBitcodeToFile(self.module.get(), c_string.as_ptr()) == 0
        }
    }

    // See GH issue #6
    #[cfg(unix)]
    pub fn write_bitcode_to_file(&self, file: &File, should_close: bool, unbuffered: bool) -> bool {
        use std::os::unix::io::AsRawFd;
        use llvm_sys::bit_writer::LLVMWriteBitcodeToFD;

        // REVIEW: as_raw_fd docs suggest it only works in *nix
        // Also, should_close should maybe be hardcoded to true?
        unsafe {
            LLVMWriteBitcodeToFD(self.module.get(), file.as_raw_fd(), should_close as i32, unbuffered as i32) == 0
        }
    }

    #[cfg(windows)]
    #[allow(unused_variables)]
    pub fn write_bitcode_to_file(&self, file: &File, should_close: bool, unbuffered: bool) -> bool {
        false
    }

    pub fn write_bitcode_to_memory(&self) -> MemoryBuffer {
        let memory_buffer = unsafe {
            LLVMWriteBitcodeToMemoryBuffer(self.module.get())
        };

        MemoryBuffer::new(memory_buffer)
    }

    /// Ensures that the current `Module` is valid, and returns a `bool`
    /// that describes whether or not it is.
    ///
    /// * `print` - Whether or not a message describing the error should be printed
    ///   to stderr, in case of failure.
    ///
    /// # Remarks
    /// See also: http://llvm.org/doxygen/Analysis_2Analysis_8cpp_source.html
    pub fn verify(&self, print: bool) -> bool {
        // REVIEW <6A>: Maybe, instead of printing the module to stderr with `print`,
        // we should return Result<(), String> that contains an error string?
        let mut err_str = unsafe { zeroed() };

        let action = if print {
            LLVMVerifierFailureAction::LLVMPrintMessageAction
        } else {
            LLVMVerifierFailureAction::LLVMReturnStatusAction
        };

        let code = unsafe {
            LLVMVerifyModule(self.module.get(), action, &mut err_str)
        };

        if code == 1 && !err_str.is_null() {
            unsafe {
                if print {
                    let rust_str = CStr::from_ptr(err_str).to_str().unwrap();

                    eprintln!("{}", rust_str);
                }

                LLVMDisposeMessage(err_str);
            }
        }

        code == 0
    }

    // REVIEW: LLVMGetDataLayoutStr was added in 3.9+ and might be more correct according to llvm-sys
    fn get_raw_data_layout(&self) -> *mut i8 {
        unsafe {
            LLVMGetDataLayout(self.module.get()) as *mut _
        }
    }

    pub fn get_data_layout(&self) -> &DataLayout {
        self.data_layout.as_ref().expect("DataLayout should always exist until Drop")
    }

    // REVIEW: Ensure the replaced string ptr still gets cleaned up by the module (I think it does)
    // valgrind might come in handy once non jemalloc allocators stabilize
    pub fn set_data_layout(&self, data_layout: &DataLayout) {
        unsafe {
            LLVMSetDataLayout(self.module.get(), data_layout.data_layout.get());
        }

        self.data_layout.as_ref()
                        .expect("DataLayout should always exist until Drop")
                        .data_layout
                        .set(self.get_raw_data_layout());
    }

    /// Prints the content of the `Module` to stderr.
    pub fn print_to_stderr(&self) {
        unsafe {
            LLVMDumpModule(self.module.get());
        }
    }

    /// Prints the content of the `Module` to a string.
    pub fn print_to_string(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMPrintModuleToString(self.module.get()))
        }
    }

    /// Prints the content of the `Module` to a file.
    pub fn print_to_file(&self, path: &Path) -> Result<(), String> {
        let path = path.to_str().expect("Did not find a valid Unicode path string");
        let mut err_str = unsafe { zeroed() };
        let return_code = unsafe {
            LLVMPrintModuleToFile(self.module.get(), path.as_ptr() as *const i8, &mut err_str)
        };

        // TODO: Verify 1 is error code (LLVM can be inconsistent)
        if return_code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(err_str);

                rust_str
            };

            return Err(rust_str);
        }

        Ok(())
    }

    pub fn set_inline_assembly(&self, asm: &str) {
        let c_string = CString::new(asm).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetModuleInlineAsm(self.module.get(), c_string.as_ptr())
        }
    }

    // REVIEW: Should module take ownership of metadata?
    // REVIEW: Should we return a MetadataValue for the global since it's its own value?
    // it would be the last item in get_global_metadata I believe
    // TODOC: Appends your metadata to a global MetadataValue<Node> indexed by key
    pub fn add_global_metadata(&self, key: &str, metadata: &MetadataValue) {
        let c_string = CString::new(key).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMAddNamedMetadataOperand(self.module.get(), c_string.as_ptr(), metadata.as_value_ref())
        }
    }
    // REVIEW: Better name?
    // TODOC: Gets the size of the metadata node indexed by key
    pub fn get_global_metadata_size(&self, key: &str) -> u32 {
        let c_string = CString::new(key).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMGetNamedMetadataNumOperands(self.module.get(), c_string.as_ptr())
        }
    }

    // TODOC: Always returns a metadata node indexed by key, which may contain 1 string or multiple values as its get_node_values()
    // SubTypes: -> Vec<MetadataValue<Node>>
    pub fn get_global_metadata(&self, key: &str) -> Vec<MetadataValue> {
        let c_string = CString::new(key).expect("Conversion to CString failed unexpectedly");
        let count = self.get_global_metadata_size(key);

        let mut raw_vec: Vec<LLVMValueRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let slice = unsafe {
            LLVMGetNamedMetadataOperands(self.module.get(), c_string.as_ptr(), ptr);

            from_raw_parts(ptr, count as usize)
        };

        slice.iter().map(|val| MetadataValue::new(*val)).collect()
    }

    pub fn get_first_global(&self) -> Option<GlobalValue> {
        let value = unsafe {
            LLVMGetFirstGlobal(self.module.get())
        };

        if value.is_null() {
            return None;
        }

        Some(GlobalValue::new(value))
    }

    pub fn get_last_global(&self) -> Option<GlobalValue> {
        let value = unsafe {
            LLVMGetLastGlobal(self.module.get())
        };

        if value.is_null() {
            return None;
        }

        Some(GlobalValue::new(value))
    }

    pub fn get_global(&self, name: &str) -> Option<GlobalValue> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let value = unsafe {
            LLVMGetNamedGlobal(self.module.get(), c_string.as_ptr())
        };

        if value.is_null() {
            return None;
        }

        Some(GlobalValue::new(value))
    }
}

impl Clone for Module {
    fn clone(&self) -> Self {
        let module = unsafe {
            LLVMCloneModule(self.module.get())
        };

        Module::new(module, self.non_global_context.as_ref())
    }
}

// Module owns the data layout string, so LLVMDisposeModule will deallocate it for us,
// so we must call forget to avoid dropping it ourselves
impl Drop for Module {
    fn drop(&mut self) {
        forget(self.data_layout.take().expect("DataLayout should always exist until Drop"));

        if self.owned_by_ee.borrow_mut().take().is_none() {
            unsafe {
                LLVMDisposeModule(self.module.get());
            }
        }

        // Context & EE will drop naturally if they are unique references at this point
    }
}
