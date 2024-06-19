//! A `Module` represents a single code compilation unit.

use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyModule};
#[allow(deprecated)]
use llvm_sys::bit_reader::LLVMParseBitcodeInContext;
use llvm_sys::bit_writer::{LLVMWriteBitcodeToFile, LLVMWriteBitcodeToMemoryBuffer};
#[llvm_versions(..=14)]
use llvm_sys::core::LLVMGetTypeByName;

use llvm_sys::core::{
    LLVMAddFunction, LLVMAddGlobal, LLVMAddGlobalInAddressSpace, LLVMAddNamedMetadataOperand, LLVMCloneModule,
    LLVMDisposeModule, LLVMDumpModule, LLVMGetFirstFunction, LLVMGetFirstGlobal, LLVMGetLastFunction,
    LLVMGetLastGlobal, LLVMGetModuleContext, LLVMGetModuleIdentifier, LLVMGetNamedFunction, LLVMGetNamedGlobal,
    LLVMGetNamedMetadataNumOperands, LLVMGetNamedMetadataOperands, LLVMGetTarget, LLVMPrintModuleToFile,
    LLVMPrintModuleToString, LLVMSetDataLayout, LLVMSetModuleIdentifier, LLVMSetTarget, LLVMDisposeMessage
};
#[llvm_versions(7..)]
use llvm_sys::core::{LLVMAddModuleFlag, LLVMGetModuleFlag};
#[llvm_versions(13..)]
use llvm_sys::error::LLVMGetErrorMessage;
use llvm_sys::execution_engine::{
    LLVMCreateExecutionEngineForModule, LLVMCreateInterpreterForModule, LLVMCreateJITCompilerForModule,
};
use llvm_sys::prelude::{LLVMModuleRef, LLVMValueRef};
#[llvm_versions(13..)]
use llvm_sys::transforms::pass_builder::LLVMRunPasses;
use llvm_sys::LLVMLinkage;
#[llvm_versions(7..)]
use llvm_sys::LLVMModuleFlagBehavior;

use std::cell::{Cell, Ref, RefCell};
use std::ffi::CStr;
use std::fs::File;
use std::marker::PhantomData;
use std::mem::{forget, MaybeUninit};
use std::path::Path;
use std::ptr;
use std::rc::Rc;

#[llvm_versions(7..)]
use crate::comdat::Comdat;
use crate::context::{AsContextRef, Context, ContextRef};
use crate::data_layout::DataLayout;
#[llvm_versions(7..)]
use crate::debug_info::{DICompileUnit, DWARFEmissionKind, DWARFSourceLanguage, DebugInfoBuilder};
use crate::execution_engine::ExecutionEngine;
use crate::memory_buffer::MemoryBuffer;
#[llvm_versions(13..)]
use crate::passes::PassBuilderOptions;
use crate::support::{to_c_str, LLVMString};
#[llvm_versions(13..)]
use crate::targets::TargetMachine;
use crate::targets::{InitializationConfig, Target, TargetTriple};
use crate::types::{AsTypeRef, BasicType, FunctionType, StructType};
#[llvm_versions(7..)]
use crate::values::BasicValue;
use crate::values::{AsValueRef, FunctionValue, GlobalValue, MetadataValue};
use crate::{AddressSpace, OptimizationLevel};

#[llvm_enum(LLVMLinkage)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// This enum defines how to link a global variable or function in a module. The variant documentation is
/// mostly taken straight from LLVM's own documentation except for some minor clarification.
///
/// It is illegal for a function declaration to have any linkage type other than external or extern_weak.
///
/// All Global Variables, Functions and Aliases can have one of the following DLL storage class: `DLLImport`
/// & `DLLExport`.
// REVIEW: Maybe this should go into it's own module?
pub enum Linkage {
    /// `Appending` linkage may only be applied to global variables of pointer to array type. When two global
    /// variables with appending linkage are linked together, the two global arrays are appended together.
    /// This is the LLVM, typesafe, equivalent of having the system linker append together "sections" with
    /// identical names when .o files are linked. Unfortunately this doesn't correspond to any feature in .o
    /// files, so it can only be used for variables like llvm.global_ctors which llvm interprets specially.
    #[llvm_variant(LLVMAppendingLinkage)]
    Appending,
    /// Globals with `AvailableExternally` linkage are never emitted into the object file corresponding to
    /// the LLVM module. From the linker's perspective, an `AvailableExternally` global is equivalent to an
    /// external declaration. They exist to allow inlining and other optimizations to take place given
    /// knowledge of the definition of the global, which is known to be somewhere outside the module. Globals
    /// with `AvailableExternally` linkage are allowed to be discarded at will, and allow inlining and other
    /// optimizations. This linkage type is only allowed on definitions, not declarations.
    #[llvm_variant(LLVMAvailableExternallyLinkage)]
    AvailableExternally,
    /// `Common` linkage is most similar to "weak" linkage, but they are used for tentative definitions
    /// in C, such as "int X;" at global scope. Symbols with Common linkage are merged in the same way as
    /// weak symbols, and they may not be deleted if unreferenced. `Common` symbols may not have an explicit
    /// section, must have a zero initializer, and may not be marked 'constant'. Functions and aliases may
    /// not have `Common` linkage.
    #[llvm_variant(LLVMCommonLinkage)]
    Common,
    /// `DLLExport` causes the compiler to provide a global pointer to a pointer in a DLL, so that it can be
    /// referenced with the dllimport attribute. On Microsoft Windows targets, the pointer name is formed by
    /// combining __imp_ and the function or variable name. Since this storage class exists for defining a dll
    /// interface, the compiler, assembler and linker know it is externally referenced and must refrain from
    /// deleting the symbol.
    #[llvm_variant(LLVMDLLExportLinkage)]
    DLLExport,
    /// `DLLImport` causes the compiler to reference a function or variable via a global pointer to a pointer
    /// that is set up by the DLL exporting the symbol. On Microsoft Windows targets, the pointer name is
    /// formed by combining __imp_ and the function or variable name.
    #[llvm_variant(LLVMDLLImportLinkage)]
    DLLImport,
    /// If none of the other identifiers are used, the global is externally visible, meaning that it
    /// participates in linkage and can be used to resolve external symbol references.
    #[llvm_variant(LLVMExternalLinkage)]
    External,
    /// The semantics of this linkage follow the ELF object file model: the symbol is weak until linked,
    /// if not linked, the symbol becomes null instead of being an undefined reference.
    #[llvm_variant(LLVMExternalWeakLinkage)]
    ExternalWeak,
    /// FIXME: Unknown linkage type
    #[llvm_variant(LLVMGhostLinkage)]
    Ghost,
    /// Similar to private, but the value shows as a local symbol (STB_LOCAL in the case of ELF) in the object
    /// file. This corresponds to the notion of the 'static' keyword in C.
    #[llvm_variant(LLVMInternalLinkage)]
    Internal,
    /// FIXME: Unknown linkage type
    #[llvm_variant(LLVMLinkerPrivateLinkage)]
    LinkerPrivate,
    /// FIXME: Unknown linkage type
    #[llvm_variant(LLVMLinkerPrivateWeakLinkage)]
    LinkerPrivateWeak,
    /// Globals with `LinkOnceAny` linkage are merged with other globals of the same name when linkage occurs.
    /// This can be used to implement some forms of inline functions, templates, or other code which must be
    /// generated in each translation unit that uses it, but where the body may be overridden with a more
    /// definitive definition later. Unreferenced `LinkOnceAny` globals are allowed to be discarded. Note that
    /// `LinkOnceAny` linkage does not actually allow the optimizer to inline the body of this function into
    /// callers because it doesn’t know if this definition of the function is the definitive definition within
    /// the program or whether it will be overridden by a stronger definition. To enable inlining and other
    /// optimizations, use `LinkOnceODR` linkage.
    #[llvm_variant(LLVMLinkOnceAnyLinkage)]
    LinkOnceAny,
    /// FIXME: Unknown linkage type
    #[llvm_variant(LLVMLinkOnceODRAutoHideLinkage)]
    LinkOnceODRAutoHide,
    /// Some languages allow differing globals to be merged, such as two functions with different semantics.
    /// Other languages, such as C++, ensure that only equivalent globals are ever merged (the "one definition
    /// rule" — "ODR"). Such languages can use the `LinkOnceODR` and `WeakODR` linkage types to indicate that
    /// the global will only be merged with equivalent globals. These linkage types are otherwise the same
    /// as their non-odr versions.
    #[llvm_variant(LLVMLinkOnceODRLinkage)]
    LinkOnceODR,
    /// Global values with `Private` linkage are only directly accessible by objects in the current module.
    /// In particular, linking code into a module with a private global value may cause the private to be
    /// renamed as necessary to avoid collisions. Because the symbol is private to the module, all references
    /// can be updated. This doesn’t show up in any symbol table in the object file.
    #[llvm_variant(LLVMPrivateLinkage)]
    Private,
    /// `WeakAny` linkage has the same merging semantics as linkonce linkage, except that unreferenced globals
    /// with weak linkage may not be discarded. This is used for globals that are declared WeakAny in C source code.
    #[llvm_variant(LLVMWeakAnyLinkage)]
    WeakAny,
    /// Some languages allow differing globals to be merged, such as two functions with different semantics.
    /// Other languages, such as C++, ensure that only equivalent globals are ever merged (the "one definition
    /// rule" — "ODR"). Such languages can use the `LinkOnceODR` and `WeakODR` linkage types to indicate that
    /// the global will only be merged with equivalent globals. These linkage types are otherwise the same
    /// as their non-odr versions.
    #[llvm_variant(LLVMWeakODRLinkage)]
    WeakODR,
}

/// Represents a reference to an LLVM `Module`.
/// The underlying module will be disposed when dropping this object.
#[derive(Debug, PartialEq, Eq)]
pub struct Module<'ctx> {
    data_layout: RefCell<Option<DataLayout>>,
    pub(crate) module: Cell<LLVMModuleRef>,
    pub(crate) owned_by_ee: RefCell<Option<ExecutionEngine<'ctx>>>,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> Module<'ctx> {
    /// Get a module from an [LLVMModuleRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid.
    pub unsafe fn new(module: LLVMModuleRef) -> Self {
        debug_assert!(!module.is_null());

        Module {
            module: Cell::new(module),
            owned_by_ee: RefCell::new(None),
            data_layout: RefCell::new(Some(Module::get_borrowed_data_layout(module))),
            _marker: PhantomData,
        }
    }

    /// Acquires the underlying raw pointer belonging to this `Module` type.
    pub fn as_mut_ptr(&self) -> LLVMModuleRef {
        self.module.get()
    }

    /// Creates a function given its `name` and `ty`, adds it to the `Module`
    /// and returns it.
    ///
    /// An optional `linkage` can be specified, without which the default value
    /// `Linkage::ExternalLinkage` will be used.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::{Module, Linkage};
    /// use inkwell::types::FunctionType;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    ///
    /// let fn_type = context.f32_type().fn_type(&[], false);
    /// let fn_val = module.add_function("my_function", fn_type, None);
    ///
    /// assert_eq!(fn_val.get_name().to_str(), Ok("my_function"));
    /// assert_eq!(fn_val.get_linkage(), Linkage::External);
    /// ```
    pub fn add_function(&self, name: &str, ty: FunctionType<'ctx>, linkage: Option<Linkage>) -> FunctionValue<'ctx> {
        let c_string = to_c_str(name);
        let fn_value = unsafe {
            FunctionValue::new(LLVMAddFunction(self.module.get(), c_string.as_ptr(), ty.as_type_ref()))
                .expect("add_function should always succeed in adding a new function")
        };

        if let Some(linkage) = linkage {
            fn_value.set_linkage(linkage)
        }

        fn_value
    }

    /// Gets the `Context` from which this `Module` originates.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::{Context, ContextRef};
    /// use inkwell::module::Module;
    ///
    /// let local_context = Context::create();
    /// let local_module = local_context.create_module("my_module");
    ///
    /// assert_eq!(local_module.get_context(), local_context);
    /// ```
    pub fn get_context(&self) -> ContextRef<'ctx> {
        unsafe { ContextRef::new(LLVMGetModuleContext(self.module.get())) }
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    ///
    /// assert_eq!(fn_value, module.get_first_function().unwrap());
    /// ```
    pub fn get_first_function(&self) -> Option<FunctionValue<'ctx>> {
        unsafe { FunctionValue::new(LLVMGetFirstFunction(self.module.get())) }
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    ///
    /// assert_eq!(fn_value, module.get_last_function().unwrap());
    /// ```
    pub fn get_last_function(&self) -> Option<FunctionValue<'ctx>> {
        unsafe { FunctionValue::new(LLVMGetLastFunction(self.module.get())) }
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    ///
    /// assert_eq!(fn_value, module.get_function("my_fn").unwrap());
    /// ```
    pub fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        let c_string = to_c_str(name);

        unsafe { FunctionValue::new(LLVMGetNamedFunction(self.module.get(), c_string.as_ptr())) }
    }

    /// An iterator over the functions in this `Module`.
    ///
    /// ```
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    ///
    /// let names: Vec<String> = module
    ///     .get_functions()
    ///     .map(|f| f.get_name().to_string_lossy().to_string())
    ///     .collect();
    ///
    /// assert_eq!(vec!["my_fn".to_owned()], names);
    /// ```
    pub fn get_functions(&self) -> FunctionIterator<'ctx> {
        FunctionIterator::from_module(self)
    }

    /// Gets a named `StructType` from this `Module`'s `Context`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    ///
    /// assert!(module.get_struct_type("foo").is_none());
    ///
    /// let opaque = context.opaque_struct_type("foo");
    ///
    /// assert_eq!(module.get_struct_type("foo").unwrap(), opaque);
    /// ```
    ///
    #[llvm_versions(..=11)]
    pub fn get_struct_type(&self, name: &str) -> Option<StructType<'ctx>> {
        let c_string = to_c_str(name);

        let struct_type = unsafe { LLVMGetTypeByName(self.module.get(), c_string.as_ptr()) };

        if struct_type.is_null() {
            return None;
        }

        unsafe { Some(StructType::new(struct_type)) }
    }

    /// Gets a named `StructType` from this `Module`'s `Context`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    ///
    /// assert!(module.get_struct_type("foo").is_none());
    ///
    /// let opaque = context.opaque_struct_type("foo");
    ///
    /// assert_eq!(module.get_struct_type("foo").unwrap(), opaque);
    /// ```
    #[llvm_versions(12..)]
    pub fn get_struct_type(&self, name: &str) -> Option<StructType<'ctx>> {
        self.get_context().get_struct_type(name)
    }

    /// Assigns a `TargetTriple` to this `Module`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use inkwell::context::Context;
    /// use inkwell::targets::{Target, TargetTriple};
    ///
    /// Target::initialize_x86(&Default::default());
    /// let context = Context::create();
    /// let module = context.create_module("mod");
    /// let triple = TargetTriple::create("x86_64-pc-linux-gnu");
    ///
    /// assert_eq!(module.get_triple(), TargetTriple::create(""));
    ///
    /// module.set_triple(&triple);
    ///
    /// assert_eq!(module.get_triple(), triple);
    /// ```
    pub fn set_triple(&self, triple: &TargetTriple) {
        unsafe { LLVMSetTarget(self.module.get(), triple.as_ptr()) }
    }

    /// Gets the `TargetTriple` assigned to this `Module`. If none has been
    /// assigned, the triple will default to "".
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use inkwell::context::Context;
    /// use inkwell::targets::{Target, TargetTriple};
    ///
    /// Target::initialize_x86(&Default::default());
    /// let context = Context::create();
    /// let module = context.create_module("mod");
    /// let triple = TargetTriple::create("x86_64-pc-linux-gnu");
    ///
    /// assert_eq!(module.get_triple(), TargetTriple::create(""));
    ///
    /// module.set_triple(&triple);
    ///
    /// assert_eq!(module.get_triple(), triple);
    /// ```
    pub fn get_triple(&self) -> TargetTriple {
        // REVIEW: This isn't an owned LLVMString, is it? If so, need to deallocate.
        let target_str = unsafe { LLVMGetTarget(self.module.get()) };

        unsafe { TargetTriple::new(LLVMString::create_from_c_str(CStr::from_ptr(target_str))) }
    }

    /// Creates an `ExecutionEngine` from this `Module`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::targets::{InitializationConfig, Target};
    ///
    /// Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let execution_engine = module.create_execution_engine().unwrap();
    ///
    /// assert_eq!(module.get_context(), context);
    /// ```
    // SubType: ExecutionEngine<Basic?>
    pub fn create_execution_engine(&self) -> Result<ExecutionEngine<'ctx>, LLVMString> {
        Target::initialize_native(&InitializationConfig::default()).map_err(|mut err_string| {
            err_string.push('\0');

            LLVMString::create_from_str(&err_string)
        })?;

        if self.owned_by_ee.borrow().is_some() {
            let string = "This module is already owned by an ExecutionEngine.\0";
            return Err(LLVMString::create_from_str(string));
        }

        let mut execution_engine = MaybeUninit::uninit();
        let mut err_string = MaybeUninit::uninit();
        let code = unsafe {
            // Takes ownership of module
            LLVMCreateExecutionEngineForModule(
                execution_engine.as_mut_ptr(),
                self.module.get(),
                err_string.as_mut_ptr(),
            )
        };

        if code == 1 {
            unsafe {
                return Err(LLVMString::new(err_string.assume_init()));
            }
        }

        let execution_engine = unsafe { execution_engine.assume_init() };
        let execution_engine = unsafe { ExecutionEngine::new(Rc::new(execution_engine), false) };

        *self.owned_by_ee.borrow_mut() = Some(execution_engine.clone());

        Ok(execution_engine)
    }

    /// Creates an interpreter `ExecutionEngine` from this `Module`.
    ///
    /// # Example
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::targets::{InitializationConfig, Target};
    ///
    /// Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let execution_engine = module.create_interpreter_execution_engine().unwrap();
    ///
    /// assert_eq!(module.get_context(), context);
    /// ```
    // SubType: ExecutionEngine<Interpreter>
    pub fn create_interpreter_execution_engine(&self) -> Result<ExecutionEngine<'ctx>, LLVMString> {
        Target::initialize_native(&InitializationConfig::default()).map_err(|mut err_string| {
            err_string.push('\0');

            LLVMString::create_from_str(&err_string)
        })?;

        if self.owned_by_ee.borrow().is_some() {
            let string = "This module is already owned by an ExecutionEngine.\0";
            return Err(LLVMString::create_from_str(string));
        }

        let mut execution_engine = MaybeUninit::uninit();
        let mut err_string = MaybeUninit::uninit();

        let code = unsafe {
            // Takes ownership of module
            LLVMCreateInterpreterForModule(
                execution_engine.as_mut_ptr(),
                self.module.get(),
                err_string.as_mut_ptr(),
            )
        };

        if code == 1 {
            unsafe {
                return Err(LLVMString::new(err_string.assume_init()));
            }
        }

        let execution_engine = unsafe { execution_engine.assume_init() };
        let execution_engine = unsafe { ExecutionEngine::new(Rc::new(execution_engine), false) };

        *self.owned_by_ee.borrow_mut() = Some(execution_engine.clone());

        Ok(execution_engine)
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
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    ///
    /// assert_eq!(module.get_context(), context);
    /// ```
    // SubType: ExecutionEngine<Jit>
    pub fn create_jit_execution_engine(
        &self,
        opt_level: OptimizationLevel,
    ) -> Result<ExecutionEngine<'ctx>, LLVMString> {
        Target::initialize_native(&InitializationConfig::default()).map_err(|mut err_string| {
            err_string.push('\0');

            LLVMString::create_from_str(&err_string)
        })?;

        if self.owned_by_ee.borrow().is_some() {
            let string = "This module is already owned by an ExecutionEngine.\0";
            return Err(LLVMString::create_from_str(string));
        }

        let mut execution_engine = MaybeUninit::uninit();
        let mut err_string = MaybeUninit::uninit();

        let code = unsafe {
            // Takes ownership of module
            LLVMCreateJITCompilerForModule(
                execution_engine.as_mut_ptr(),
                self.module.get(),
                opt_level as u32,
                err_string.as_mut_ptr(),
            )
        };

        if code == 1 {
            unsafe {
                return Err(LLVMString::new(err_string.assume_init()));
            }
        }

        let execution_engine = unsafe { execution_engine.assume_init() };
        let execution_engine = unsafe { ExecutionEngine::new(Rc::new(execution_engine), true) };

        *self.owned_by_ee.borrow_mut() = Some(execution_engine.clone());

        Ok(execution_engine)
    }

    /// Creates a `GlobalValue` based on a type in an address space.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("mod");
    /// let i8_type = context.i8_type();
    /// let global = module.add_global(i8_type, Some(AddressSpace::from(1u16)), "my_global");
    ///
    /// assert_eq!(module.get_first_global().unwrap(), global);
    /// assert_eq!(module.get_last_global().unwrap(), global);
    /// ```
    pub fn add_global<T: BasicType<'ctx>>(
        &self,
        type_: T,
        address_space: Option<AddressSpace>,
        name: &str,
    ) -> GlobalValue<'ctx> {
        let c_string = to_c_str(name);

        let value = unsafe {
            match address_space {
                Some(address_space) => LLVMAddGlobalInAddressSpace(
                    self.module.get(),
                    type_.as_type_ref(),
                    c_string.as_ptr(),
                    address_space.0,
                ),
                None => LLVMAddGlobal(self.module.get(), type_.as_type_ref(), c_string.as_ptr()),
            }
        };

        unsafe { GlobalValue::new(value) }
    }

    /// Writes a `Module` to a `Path`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// use std::path::Path;
    ///
    /// let mut path = Path::new("module.bc");
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    ///
    /// module.add_function("my_fn", fn_type, None);
    /// module.write_bitcode_to_path(&path);
    /// ```
    pub fn write_bitcode_to_path(&self, path: &Path) -> bool {
        let path_str = path.to_str().expect("Did not find a valid Unicode path string");
        let c_string = to_c_str(path_str);

        unsafe { LLVMWriteBitcodeToFile(self.module.get(), c_string.as_ptr()) == 0 }
    }

    // See GH issue #6
    /// `write_bitcode_to_path` should be preferred over this method, as it does not work on all operating systems.
    pub fn write_bitcode_to_file(&self, file: &File, should_close: bool, unbuffered: bool) -> bool {
        #[cfg(unix)]
        {
            use llvm_sys::bit_writer::LLVMWriteBitcodeToFD;
            use std::os::unix::io::AsRawFd;

            // REVIEW: as_raw_fd docs suggest it only works in *nix
            // Also, should_close should maybe be hardcoded to true?
            unsafe {
                LLVMWriteBitcodeToFD(
                    self.module.get(),
                    file.as_raw_fd(),
                    should_close as i32,
                    unbuffered as i32,
                ) == 0
            }
        }
        #[cfg(not(unix))]
        return false;
    }

    /// Writes this `Module` to a `MemoryBuffer`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let f = module.add_function("f", fn_type, None);
    /// let basic_block = context.append_basic_block(f, "entry");
    /// let builder = context.create_builder();
    ///
    /// builder.position_at_end(basic_block);
    /// builder.build_return(None);
    ///
    /// let buffer = module.write_bitcode_to_memory();
    /// ```
    pub fn write_bitcode_to_memory(&self) -> MemoryBuffer {
        let memory_buffer = unsafe { LLVMWriteBitcodeToMemoryBuffer(self.module.get()) };

        unsafe { MemoryBuffer::new(memory_buffer) }
    }

    /// Ensures that the current `Module` is valid, and returns a `Result`
    /// that describes whether or not it is, returning a LLVM allocated string on error.
    ///
    /// # Remarks
    /// See also: http://llvm.org/doxygen/Analysis_2Analysis_8cpp_source.html
    pub fn verify(&self) -> Result<(), LLVMString> {
        let mut err_str = MaybeUninit::uninit();

        let action = LLVMVerifierFailureAction::LLVMReturnStatusAction;

        let code = unsafe { LLVMVerifyModule(self.module.get(), action, err_str.as_mut_ptr()) };

        let err_str = unsafe { err_str.assume_init() };
        if code == 1 && !err_str.is_null() {
            return unsafe { Err(LLVMString::new(err_str)) };
        }

        unsafe { LLVMDisposeMessage(err_str) };

        Ok(())
    }

    fn get_borrowed_data_layout(module: LLVMModuleRef) -> DataLayout {
        let data_layout = unsafe {
            use llvm_sys::core::LLVMGetDataLayoutStr;

            LLVMGetDataLayoutStr(module)
        };

        unsafe { DataLayout::new_borrowed(data_layout) }
    }

    /// Gets a smart pointer to the `DataLayout` belonging to a particular `Module`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::OptimizationLevel;
    /// use inkwell::context::Context;
    /// use inkwell::targets::{InitializationConfig, Target};
    ///
    /// Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");
    ///
    /// let context = Context::create();
    /// let module = context.create_module("sum");
    /// let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    /// let target_data = execution_engine.get_target_data();
    /// let data_layout = target_data.get_data_layout();
    ///
    /// module.set_data_layout(&data_layout);
    ///
    /// assert_eq!(*module.get_data_layout(), data_layout);
    /// ```
    pub fn get_data_layout(&self) -> Ref<DataLayout> {
        Ref::map(self.data_layout.borrow(), |l| {
            l.as_ref().expect("DataLayout should always exist until Drop")
        })
    }

    // REVIEW: Ensure the replaced string ptr still gets cleaned up by the module (I think it does)
    // valgrind might come in handy once non jemalloc allocators stabilize
    /// Sets the `DataLayout` for a particular `Module`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::OptimizationLevel;
    /// use inkwell::context::Context;
    /// use inkwell::targets::{InitializationConfig, Target};
    ///
    /// Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");
    ///
    /// let context = Context::create();
    /// let module = context.create_module("sum");
    /// let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    /// let target_data = execution_engine.get_target_data();
    /// let data_layout = target_data.get_data_layout();
    ///
    /// module.set_data_layout(&data_layout);
    ///
    /// assert_eq!(*module.get_data_layout(), data_layout);
    /// ```
    pub fn set_data_layout(&self, data_layout: &DataLayout) {
        unsafe {
            LLVMSetDataLayout(self.module.get(), data_layout.as_ptr());
        }

        *self.data_layout.borrow_mut() = Some(Module::get_borrowed_data_layout(self.module.get()));
    }

    /// Prints the content of the `Module` to stderr.
    pub fn print_to_stderr(&self) {
        unsafe {
            LLVMDumpModule(self.module.get());
        }
    }

    /// Prints the content of the `Module` to an `LLVMString`.
    pub fn print_to_string(&self) -> LLVMString {
        unsafe { LLVMString::new(LLVMPrintModuleToString(self.module.get())) }
    }

    /// Prints the content of the `Module` to a file.
    pub fn print_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), LLVMString> {
        let path_str = path
            .as_ref()
            .to_str()
            .expect("Did not find a valid Unicode path string");
        let path = to_c_str(path_str);
        let mut err_string = MaybeUninit::uninit();
        let return_code = unsafe {
            LLVMPrintModuleToFile(
                self.module.get(),
                path.as_ptr() as *const ::libc::c_char,
                err_string.as_mut_ptr(),
            )
        };

        if return_code == 1 {
            unsafe {
                return Err(LLVMString::new(err_string.assume_init()));
            }
        }

        Ok(())
    }

    /// Prints the content of the `Module` to a `String`.
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.print_to_string().to_string()
    }

    /// Sets the inline assembly for the `Module`.
    pub fn set_inline_assembly(&self, asm: &str) {
        #[cfg(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0"))]
        {
            use llvm_sys::core::LLVMSetModuleInlineAsm;

            let c_string = to_c_str(asm);

            unsafe { LLVMSetModuleInlineAsm(self.module.get(), c_string.as_ptr()) }
        }
        #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
        {
            use llvm_sys::core::LLVMSetModuleInlineAsm2;

            unsafe { LLVMSetModuleInlineAsm2(self.module.get(), asm.as_ptr() as *const ::libc::c_char, asm.len()) }
        }
    }

    // REVIEW: Should module take ownership of metadata?
    // REVIEW: Should we return a MetadataValue for the global since it's its own value?
    // it would be the last item in get_global_metadata I believe
    // TODOC: Appends your metadata to a global MetadataValue<Node> indexed by key
    /// Appends a `MetaDataValue` to a global list indexed by a particular key.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let bool_type = context.bool_type();
    /// let f32_type = context.f32_type();
    /// let bool_val = bool_type.const_int(0, false);
    /// let f32_val = f32_type.const_float(0.0);
    ///
    /// assert_eq!(module.get_global_metadata_size("my_md"), 0);
    ///
    /// let md_string = context.metadata_string("lots of metadata here");
    /// let md_node = context.metadata_node(&[bool_val.into(), f32_val.into()]);
    ///
    /// module.add_global_metadata("my_md", &md_string).unwrap();
    /// module.add_global_metadata("my_md", &md_node).unwrap();
    ///
    /// assert_eq!(module.get_global_metadata_size("my_md"), 2);
    ///
    /// let global_md = module.get_global_metadata("my_md");
    ///
    /// assert_eq!(global_md.len(), 2);
    ///
    /// let (md_0, md_1) = (global_md[0].get_node_values(), global_md[1].get_node_values());
    ///
    /// assert_eq!(md_0.len(), 1);
    /// assert_eq!(md_1.len(), 2);
    /// assert_eq!(md_0[0].into_metadata_value().get_string_value(), md_string.get_string_value());
    /// assert_eq!(md_1[0].into_int_value(), bool_val);
    /// assert_eq!(md_1[1].into_float_value(), f32_val);
    /// ```
    pub fn add_global_metadata(&self, key: &str, metadata: &MetadataValue<'ctx>) -> Result<(), &'static str> {
        if !metadata.is_node() {
            return Err("metadata is expected to be a node.");
        }

        let c_string = to_c_str(key);
        unsafe {
            LLVMAddNamedMetadataOperand(self.module.get(), c_string.as_ptr(), metadata.as_value_ref());
        }

        Ok(())
    }

    // REVIEW: Better name? get_global_metadata_len or _count?
    /// Obtains the number of `MetaDataValue`s indexed by a particular key.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let bool_type = context.bool_type();
    /// let f32_type = context.f32_type();
    /// let bool_val = bool_type.const_int(0, false);
    /// let f32_val = f32_type.const_float(0.0);
    ///
    /// assert_eq!(module.get_global_metadata_size("my_md"), 0);
    ///
    /// let md_string = context.metadata_string("lots of metadata here");
    /// let md_node = context.metadata_node(&[bool_val.into(), f32_val.into()]);
    ///
    /// module.add_global_metadata("my_md", &md_string).unwrap();
    /// module.add_global_metadata("my_md", &md_node).unwrap();
    ///
    /// assert_eq!(module.get_global_metadata_size("my_md"), 2);
    ///
    /// let global_md = module.get_global_metadata("my_md");
    ///
    /// assert_eq!(global_md.len(), 2);
    ///
    /// let (md_0, md_1) = (global_md[0].get_node_values(), global_md[1].get_node_values());
    ///
    /// assert_eq!(md_0.len(), 1);
    /// assert_eq!(md_1.len(), 2);
    /// assert_eq!(md_0[0].into_metadata_value().get_string_value(), md_string.get_string_value());
    /// assert_eq!(md_1[0].into_int_value(), bool_val);
    /// assert_eq!(md_1[1].into_float_value(), f32_val);
    /// ```
    pub fn get_global_metadata_size(&self, key: &str) -> u32 {
        let c_string = to_c_str(key);

        unsafe { LLVMGetNamedMetadataNumOperands(self.module.get(), c_string.as_ptr()) }
    }

    // SubTypes: -> Vec<MetadataValue<Node>>
    /// Obtains the global `MetaDataValue` node indexed by key, which may contain 1 string or multiple values as its `get_node_values()`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let bool_type = context.bool_type();
    /// let f32_type = context.f32_type();
    /// let bool_val = bool_type.const_int(0, false);
    /// let f32_val = f32_type.const_float(0.0);
    ///
    /// assert_eq!(module.get_global_metadata_size("my_md"), 0);
    ///
    /// let md_string = context.metadata_string("lots of metadata here");
    /// let md_node = context.metadata_node(&[bool_val.into(), f32_val.into()]);
    ///
    /// module.add_global_metadata("my_md", &md_string).unwrap();
    /// module.add_global_metadata("my_md", &md_node).unwrap();
    ///
    /// assert_eq!(module.get_global_metadata_size("my_md"), 2);
    ///
    /// let global_md = module.get_global_metadata("my_md");
    ///
    /// assert_eq!(global_md.len(), 2);
    ///
    /// let (md_0, md_1) = (global_md[0].get_node_values(), global_md[1].get_node_values());
    ///
    /// assert_eq!(md_0.len(), 1);
    /// assert_eq!(md_1.len(), 2);
    /// assert_eq!(md_0[0].into_metadata_value().get_string_value(), md_string.get_string_value());
    /// assert_eq!(md_1[0].into_int_value(), bool_val);
    /// assert_eq!(md_1[1].into_float_value(), f32_val);
    /// ```
    pub fn get_global_metadata(&self, key: &str) -> Vec<MetadataValue<'ctx>> {
        let c_string = to_c_str(key);
        let count = self.get_global_metadata_size(key) as usize;

        let mut vec: Vec<LLVMValueRef> = Vec::with_capacity(count);
        let ptr = vec.as_mut_ptr();

        unsafe {
            LLVMGetNamedMetadataOperands(self.module.get(), c_string.as_ptr(), ptr);

            vec.set_len(count);
        };

        vec.iter().map(|val| unsafe { MetadataValue::new(*val) }).collect()
    }

    /// Gets the first `GlobalValue` in a module.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let module = context.create_module("mod");
    ///
    /// assert!(module.get_first_global().is_none());
    ///
    /// let global = module.add_global(i8_type, Some(AddressSpace::from(4u16)), "my_global");
    ///
    /// assert_eq!(module.get_first_global().unwrap(), global);
    /// ```
    pub fn get_first_global(&self) -> Option<GlobalValue<'ctx>> {
        let value = unsafe { LLVMGetFirstGlobal(self.module.get()) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(GlobalValue::new(value)) }
    }

    /// Gets the last `GlobalValue` in a module.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("mod");
    /// let i8_type = context.i8_type();
    ///
    /// assert!(module.get_last_global().is_none());
    ///
    /// let global = module.add_global(i8_type, Some(AddressSpace::from(4u16)), "my_global");
    ///
    /// assert_eq!(module.get_last_global().unwrap(), global);
    /// ```
    pub fn get_last_global(&self) -> Option<GlobalValue<'ctx>> {
        let value = unsafe { LLVMGetLastGlobal(self.module.get()) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(GlobalValue::new(value)) }
    }

    /// Gets a named `GlobalValue` in a module.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::AddressSpace;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("mod");
    /// let i8_type = context.i8_type();
    ///
    /// assert!(module.get_global("my_global").is_none());
    ///
    /// let global = module.add_global(i8_type, Some(AddressSpace::from(4u16)), "my_global");
    ///
    /// assert_eq!(module.get_global("my_global").unwrap(), global);
    /// ```
    pub fn get_global(&self, name: &str) -> Option<GlobalValue<'ctx>> {
        let c_string = to_c_str(name);
        let value = unsafe { LLVMGetNamedGlobal(self.module.get(), c_string.as_ptr()) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(GlobalValue::new(value)) }
    }

    /// An iterator over the globals in this `Module`.
    pub fn get_globals(&self) -> GlobalIterator<'ctx> {
        GlobalIterator::from_module(self)
    }

    /// Creates a new `Module` from a `MemoryBuffer` with bitcode.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use inkwell::memory_buffer::MemoryBuffer;
    /// use std::path::Path;
    ///
    /// let path = Path::new("foo/bar.bc");
    /// let context = Context::create();
    /// let buffer = MemoryBuffer::create_from_file(&path).unwrap();
    /// let module = Module::parse_bitcode_from_buffer(&buffer, &context);
    ///
    /// assert_eq!(module.unwrap().get_context(), context);
    ///
    /// ```
    pub fn parse_bitcode_from_buffer(
        buffer: &MemoryBuffer,
        context: impl AsContextRef<'ctx>,
    ) -> Result<Self, LLVMString> {
        let mut module = MaybeUninit::uninit();
        let mut err_string = MaybeUninit::uninit();

        // LLVM has a newer version of this function w/o the error result since 3.8 but this deprecated function
        // hasen't yet been removed even in LLVM 8. Seems fine to use instead of switching to their
        // error diagnostics handler for now.
        #[allow(deprecated)]
        let success = unsafe {
            LLVMParseBitcodeInContext(
                context.as_ctx_ref(),
                buffer.memory_buffer,
                module.as_mut_ptr(),
                err_string.as_mut_ptr(),
            )
        };

        if success != 0 {
            unsafe {
                return Err(LLVMString::new(err_string.assume_init()));
            }
        }

        unsafe { Ok(Module::new(module.assume_init())) }
    }

    /// A convenience function for creating a `Module` from a bitcode file for a given context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::module::Module;
    /// use std::path::Path;
    ///
    /// let path = Path::new("foo/bar.bc");
    /// let context = Context::create();
    /// let module = Module::parse_bitcode_from_path(&path, &context);
    ///
    /// assert_eq!(module.unwrap().get_context(), context);
    ///
    /// ```
    // LLVMGetBitcodeModuleInContext was a pain to use, so I seem to be able to achieve the same effect
    // by reusing create_from_file instead. This is basically just a convenience function.
    pub fn parse_bitcode_from_path<P: AsRef<Path>>(
        path: P,
        context: impl AsContextRef<'ctx>,
    ) -> Result<Self, LLVMString> {
        let buffer = MemoryBuffer::create_from_file(path.as_ref())?;

        Self::parse_bitcode_from_buffer(&buffer, context)
    }

    /// Gets the name of this `Module`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    ///
    /// assert_eq!(module.get_name().to_str(), Ok("my_mdoule"));
    /// ```
    pub fn get_name(&self) -> &CStr {
        let mut length = 0;
        let cstr_ptr = unsafe { LLVMGetModuleIdentifier(self.module.get(), &mut length) };

        unsafe { CStr::from_ptr(cstr_ptr) }
    }

    /// Assigns the name of this `Module`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    ///
    /// module.set_name("my_module2");
    ///
    /// assert_eq!(module.get_name().to_str(), Ok("my_module2"));
    /// ```
    pub fn set_name(&self, name: &str) {
        unsafe { LLVMSetModuleIdentifier(self.module.get(), name.as_ptr() as *const ::libc::c_char, name.len()) }
    }

    /// Gets the source file name. It defaults to the module identifier but is separate from it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    ///
    /// assert_eq!(module.get_source_file_name().to_str(), Ok("my_mod"));
    ///
    /// module.set_source_file_name("my_mod.rs");
    ///
    /// assert_eq!(module.get_name().to_str(), Ok("my_mod"));
    /// assert_eq!(module.get_source_file_name().to_str(), Ok("my_mod.rs"));
    /// ```
    #[llvm_versions(7..)]
    pub fn get_source_file_name(&self) -> &CStr {
        use llvm_sys::core::LLVMGetSourceFileName;

        let mut len = 0;
        let ptr = unsafe { LLVMGetSourceFileName(self.module.get(), &mut len) };

        unsafe { CStr::from_ptr(ptr) }
    }

    /// Sets the source file name. It defaults to the module identifier but is separate from it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    ///
    /// assert_eq!(module.get_source_file_name().to_str(), Ok("my_mod"));
    ///
    /// module.set_source_file_name("my_mod.rs");
    ///
    /// assert_eq!(module.get_name().to_str(), Ok("my_mod"));
    /// assert_eq!(module.get_source_file_name().to_str(), Ok("my_mod.rs"));
    /// ```
    #[llvm_versions(7..)]
    pub fn set_source_file_name(&self, file_name: &str) {
        use llvm_sys::core::LLVMSetSourceFileName;

        unsafe {
            LLVMSetSourceFileName(
                self.module.get(),
                file_name.as_ptr() as *const ::libc::c_char,
                file_name.len(),
            )
        }
    }

    /// Links one module into another. This will merge two `Module`s into one.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("mod");
    /// let module2 = context.create_module("mod2");
    ///
    /// assert!(module.link_in_module(module2).is_ok());
    /// ```
    pub fn link_in_module(&self, other: Self) -> Result<(), LLVMString> {
        if other.owned_by_ee.borrow().is_some() {
            let string = "Cannot link a module which is already owned by an ExecutionEngine.\0";
            return Err(LLVMString::create_from_str(string));
        }

        use crate::support::error_handling::get_error_str_diagnostic_handler;
        use libc::c_void;
        use llvm_sys::linker::LLVMLinkModules2;

        let context = self.get_context();

        let mut char_ptr: *mut ::libc::c_char = ptr::null_mut();
        let char_ptr_ptr = &mut char_ptr as *mut *mut ::libc::c_char as *mut *mut c_void as *mut c_void;

        // Newer LLVM versions don't use an out ptr anymore which was really straightforward...
        // Here we assign an error handler to extract the error message, if any, for us.
        context.set_diagnostic_handler(get_error_str_diagnostic_handler, char_ptr_ptr);

        let code = unsafe { LLVMLinkModules2(self.module.get(), other.module.get()) };

        forget(other);

        if code == 1 {
            debug_assert!(!char_ptr.is_null());

            unsafe { Err(LLVMString::new(char_ptr)) }
        } else {
            Ok(())
        }
    }

    /// Gets the `Comdat` associated with a particular name. If it does not exist, it will be created.
    /// A new `Comdat` defaults to a kind of `ComdatSelectionKind::Any`.
    #[llvm_versions(7..)]
    pub fn get_or_insert_comdat(&self, name: &str) -> Comdat {
        use llvm_sys::comdat::LLVMGetOrInsertComdat;

        let c_string = to_c_str(name);
        let comdat_ptr = unsafe { LLVMGetOrInsertComdat(self.module.get(), c_string.as_ptr()) };

        unsafe { Comdat::new(comdat_ptr) }
    }

    /// Gets the `MetadataValue` flag associated with the key in this module, if any.
    /// If a `BasicValue` was used to create this flag, it will be wrapped in a `MetadataValue`
    /// when returned from this function.
    // SubTypes: Might need to return Option<BVE, MV<Enum>, or MV<String>>
    #[llvm_versions(7..)]
    pub fn get_flag(&self, key: &str) -> Option<MetadataValue<'ctx>> {
        use llvm_sys::core::LLVMMetadataAsValue;

        let flag = unsafe { LLVMGetModuleFlag(self.module.get(), key.as_ptr() as *const ::libc::c_char, key.len()) };

        if flag.is_null() {
            return None;
        }

        let flag_value = unsafe { LLVMMetadataAsValue(LLVMGetModuleContext(self.module.get()), flag) };

        unsafe { Some(MetadataValue::new(flag_value)) }
    }

    /// Append a `MetadataValue` as a module wide flag. Note that using the same key twice
    /// will likely invalidate the module.
    #[llvm_versions(7..)]
    pub fn add_metadata_flag(&self, key: &str, behavior: FlagBehavior, flag: MetadataValue<'ctx>) {
        let md = flag.as_metadata_ref();

        unsafe {
            LLVMAddModuleFlag(
                self.module.get(),
                behavior.into(),
                key.as_ptr() as *mut ::libc::c_char,
                key.len(),
                md,
            )
        }
    }

    /// Append a `BasicValue` as a module wide flag. Note that using the same key twice
    /// will likely invalidate the module.
    // REVIEW: What happens if value is not const?
    #[llvm_versions(7..)]
    pub fn add_basic_value_flag<BV: BasicValue<'ctx>>(&self, key: &str, behavior: FlagBehavior, flag: BV) {
        use llvm_sys::core::LLVMValueAsMetadata;

        let md = unsafe { LLVMValueAsMetadata(flag.as_value_ref()) };

        unsafe {
            LLVMAddModuleFlag(
                self.module.get(),
                behavior.into(),
                key.as_ptr() as *mut ::libc::c_char,
                key.len(),
                md,
            )
        }
    }

    /// Strips and debug info from the module, if it exists.
    #[llvm_versions(6..)]
    pub fn strip_debug_info(&self) -> bool {
        use llvm_sys::debuginfo::LLVMStripModuleDebugInfo;

        unsafe { LLVMStripModuleDebugInfo(self.module.get()) == 1 }
    }

    /// Gets the version of debug metadata contained in this `Module`.
    #[llvm_versions(6..)]
    pub fn get_debug_metadata_version(&self) -> libc::c_uint {
        use llvm_sys::debuginfo::LLVMGetModuleDebugMetadataVersion;

        unsafe { LLVMGetModuleDebugMetadataVersion(self.module.get()) }
    }

    /// Creates a `DebugInfoBuilder` for this `Module`.
    #[llvm_versions(7..)]
    pub fn create_debug_info_builder(
        &self,
        allow_unresolved: bool,
        language: DWARFSourceLanguage,
        filename: &str,
        directory: &str,
        producer: &str,
        is_optimized: bool,
        flags: &str,
        runtime_ver: libc::c_uint,
        split_name: &str,
        kind: DWARFEmissionKind,
        dwo_id: libc::c_uint,
        split_debug_inlining: bool,
        debug_info_for_profiling: bool,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        sysroot: &str,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        sdk: &str,
    ) -> (DebugInfoBuilder<'ctx>, DICompileUnit<'ctx>) {
        DebugInfoBuilder::new(
            self,
            allow_unresolved,
            language,
            filename,
            directory,
            producer,
            is_optimized,
            flags,
            runtime_ver,
            split_name,
            kind,
            dwo_id,
            split_debug_inlining,
            debug_info_for_profiling,
            #[cfg(any(
                feature = "llvm11-0",
                feature = "llvm12-0",
                feature = "llvm13-0",
                feature = "llvm14-0",
                feature = "llvm15-0",
                feature = "llvm16-0",
                feature = "llvm17-0",
                feature = "llvm18-0"
            ))]
            sysroot,
            #[cfg(any(
                feature = "llvm11-0",
                feature = "llvm12-0",
                feature = "llvm13-0",
                feature = "llvm14-0",
                feature = "llvm15-0",
                feature = "llvm16-0",
                feature = "llvm17-0",
                feature = "llvm18-0"
            ))]
            sdk,
        )
    }

    /// Construct and run a set of passes over a module.
    /// This function takes a string with the passes that should be used.
    /// The format of this string is the same as opt's -passes argument for the new pass manager.
    /// Individual passes may be specified, separated by commas.
    /// Full pipelines may also be invoked using default<O3> and friends.
    /// See opt for full reference of the Passes format.
    #[llvm_versions(13..)]
    pub fn run_passes(
        &self,
        passes: &str,
        machine: &TargetMachine,
        options: PassBuilderOptions,
    ) -> Result<(), LLVMString> {
        unsafe {
            let error = LLVMRunPasses(
                self.module.get(),
                to_c_str(passes).as_ptr(),
                machine.target_machine,
                options.options_ref,
            );
            if error.is_null() {
                Ok(())
            } else {
                let message = LLVMGetErrorMessage(error);
                Err(LLVMString::new(message as *const libc::c_char))
            }
        }
    }
}

impl Clone for Module<'_> {
    fn clone(&self) -> Self {
        // REVIEW: Is this just a LLVM 6 bug? We could conditionally compile this assertion for affected versions
        let verify = self.verify();

        assert!(
            verify.is_ok(),
            "Cloning a Module seems to segfault when module is not valid. We are preventing that here. Error: {}",
            verify.unwrap_err()
        );

        unsafe { Module::new(LLVMCloneModule(self.module.get())) }
    }
}

// Module owns the data layout string, so LLVMDisposeModule will deallocate it for us.
// which is why DataLayout must be called with `new_borrowed`
impl Drop for Module<'_> {
    fn drop(&mut self) {
        if self.owned_by_ee.borrow_mut().take().is_none() {
            unsafe {
                LLVMDisposeModule(self.module.get());
            }
        }

        // Context & EE will drop naturally if they are unique references at this point
    }
}

#[llvm_versions(7..)]
#[llvm_enum(LLVMModuleFlagBehavior)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Defines the operational behavior for a module wide flag. This documentation comes directly
/// from the LLVM docs
pub enum FlagBehavior {
    /// Emits an error if two values disagree, otherwise the resulting value is that of the operands.
    #[llvm_variant(LLVMModuleFlagBehaviorError)]
    Error,
    /// Emits a warning if two values disagree. The result value will be the operand for the
    /// flag from the first module being linked.
    #[llvm_variant(LLVMModuleFlagBehaviorWarning)]
    Warning,
    /// Adds a requirement that another module flag be present and have a specified value after
    /// linking is performed. The value must be a metadata pair, where the first element of the
    /// pair is the ID of the module flag to be restricted, and the second element of the pair
    /// is the value the module flag should be restricted to. This behavior can be used to
    /// restrict the allowable results (via triggering of an error) of linking IDs with the
    /// **Override** behavior.
    #[llvm_variant(LLVMModuleFlagBehaviorRequire)]
    Require,
    /// Uses the specified value, regardless of the behavior or value of the other module. If
    /// both modules specify **Override**, but the values differ, an error will be emitted.
    #[llvm_variant(LLVMModuleFlagBehaviorOverride)]
    Override,
    /// Appends the two values, which are required to be metadata nodes.
    #[llvm_variant(LLVMModuleFlagBehaviorAppend)]
    Append,
    /// Appends the two values, which are required to be metadata nodes. However, duplicate
    /// entries in the second list are dropped during the append operation.
    #[llvm_variant(LLVMModuleFlagBehaviorAppendUnique)]
    AppendUnique,
}

/// Iterate over all `FunctionValue`s in an llvm module
#[derive(Debug)]
pub struct FunctionIterator<'ctx>(Option<FunctionValue<'ctx>>);

impl<'ctx> FunctionIterator<'ctx> {
    fn from_module(module: &Module<'ctx>) -> Self {
        Self(module.get_first_function())
    }
}

impl<'ctx> Iterator for FunctionIterator<'ctx> {
    type Item = FunctionValue<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(func) = self.0 {
            self.0 = func.get_next_function();
            Some(func)
        } else {
            None
        }
    }
}

/// Iterate over all `GlobalValue`s in an llvm module
#[derive(Debug)]
pub struct GlobalIterator<'ctx>(Option<GlobalValue<'ctx>>);

impl<'ctx> GlobalIterator<'ctx> {
    fn from_module(module: &Module<'ctx>) -> Self {
        Self(module.get_first_global())
    }
}

impl<'ctx> Iterator for GlobalIterator<'ctx> {
    type Item = GlobalValue<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(global) = self.0 {
            self.0 = global.get_next_global();
            Some(global)
        } else {
            None
        }
    }
}
