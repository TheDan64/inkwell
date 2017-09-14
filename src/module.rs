use llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction};
use llvm_sys::bit_writer::{LLVMWriteBitcodeToFile, LLVMWriteBitcodeToMemoryBuffer, LLVMWriteBitcodeToFD};
use llvm_sys::core::{LLVMAddFunction, LLVMAddGlobal, LLVMDisposeMessage, LLVMDumpModule, LLVMGetNamedFunction, LLVMGetTypeByName, LLVMSetDataLayout, LLVMSetInitializer, LLVMSetTarget, LLVMCloneModule, LLVMDisposeModule, LLVMGetTarget, LLVMGetDataLayout, LLVMModuleCreateWithName, LLVMGetModuleContext, LLVMGetFirstFunction, LLVMGetLastFunction, LLVMSetLinkage, LLVMAddGlobalInAddressSpace, LLVMPrintModuleToString, LLVMGetNamedMetadataNumOperands, LLVMAddNamedMetadataOperand, LLVMGetNamedMetadataOperands};
use llvm_sys::execution_engine::{LLVMCreateJITCompilerForModule, LLVMCreateMCJITCompilerForModule};
use llvm_sys::prelude::{LLVMValueRef, LLVMModuleRef};
use llvm_sys::LLVMLinkage;

use std::ffi::{CString, CStr};
use std::fs::File;
use std::mem::{forget, uninitialized, zeroed};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::slice::from_raw_parts;

use context::{Context, ContextRef};
use data_layout::DataLayout;
use execution_engine::ExecutionEngine;
use memory_buffer::MemoryBuffer;
use types::{AsTypeRef, BasicType, FunctionType, BasicTypeEnum};
use values::{AsValueRef, BasicValue, FunctionValue, PointerValue, MetadataValue, BasicMetadataValueEnum};

// REVIEW: Maybe this should go into it's own module?
#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub(crate) module: LLVMModuleRef,
    data_layout: Option<DataLayout>,
}

impl Module {
    pub(crate) fn new(module: LLVMModuleRef) -> Self {
        assert!(!module.is_null());

        let mut module = Module {
            module: module,
            data_layout: None,
        };

        module.data_layout = Some(DataLayout::new(module.get_raw_data_layout()));
        module
    }

    pub fn create(name: &str) -> Self {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let module = unsafe {
            LLVMModuleCreateWithName(c_string.as_ptr())
        };

        Module::new(module)
    }

    // TODOC: LLVM will default linkage to ExternalLinkage (at least in 3.7)
    pub fn add_function(&self, name: &str, return_type: &FunctionType, linkage: Option<&Linkage>) -> FunctionValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMAddFunction(self.module, c_string.as_ptr(), return_type.as_type_ref())
        };

        if let Some(linkage) = linkage {
            unsafe {
                LLVMSetLinkage(value, linkage.as_llvm_linkage());
            }
        }

        FunctionValue::new(value).expect("add_function should always succeed in adding a new function")
    }

    pub fn get_context(&self) -> ContextRef {
        let context = unsafe {
            LLVMGetModuleContext(self.module)
        };

        ContextRef::new(Context::new(context))
    }

    pub fn get_first_function(&self) -> Option<FunctionValue> {
        let function = unsafe {
            LLVMGetFirstFunction(self.module)
        };

        FunctionValue::new(function)
    }

    pub fn get_last_function(&self) -> Option<FunctionValue> {
        let function = unsafe {
            LLVMGetLastFunction(self.module)
        };

        FunctionValue::new(function)
    }

    pub fn get_function(&self, name: &str) -> Option<FunctionValue> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMGetNamedFunction(self.module, c_string.as_ptr())
        };

        FunctionValue::new(value)
    }

    pub fn get_type(&self, name: &str) -> Option<BasicTypeEnum> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let type_ = unsafe {
            LLVMGetTypeByName(self.module, c_string.as_ptr())
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
            LLVMSetTarget(self.module, c_string.as_ptr())
        }
    }

    // REVIEW: Can we link this to Target.from_name or Target.from_triple / etc?
    pub fn get_target(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTarget(self.module))
        }
    }

    // TODOC: EE must *own* modules and deal out references
    // REVIEW: Looking at LLVM source, opt_level seems to be casted into a targets::CodeGenOptLevel
    // or LLVM equivalent anyway
    pub fn create_jit_execution_engine(self, opt_level: u32) -> Result<ExecutionEngine, String> {
        let mut execution_engine = unsafe { uninitialized() };
        let mut err_str = unsafe { zeroed() };

        let code = unsafe {
            LLVMCreateJITCompilerForModule(&mut execution_engine, self.module, opt_level, &mut err_str) // Should take ownership of module
        };

        if code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(err_str);

                rust_str
            };

            return Err(rust_str);
        }

        let mut execution_engine = ExecutionEngine::new(execution_engine, true);

        execution_engine.modules.push(self);

        Ok(execution_engine)
    }

    // REVIEW: Is this really always a pointer? It would make sense...
    pub fn add_global(&self, type_: &BasicType, initial_value: Option<&BasicValue>, address_space: Option<u32>, name: &str) -> PointerValue {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            match address_space {
                Some(address_space) => LLVMAddGlobalInAddressSpace(self.module, type_.as_type_ref(), c_string.as_ptr(), address_space),
                None => LLVMAddGlobal(self.module, type_.as_type_ref(), c_string.as_ptr()),
            }
        };

        if let Some(init_val) = initial_value {
            unsafe {
                LLVMSetInitializer(value, init_val.as_value_ref())
            }
        }

        PointerValue::new(value)
    }

    pub fn write_bitcode_to_path(&self, path: &Path) -> bool {
        let path_str = path.to_str().expect("Did not find a valid Unicode path string");
        let c_string = CString::new(path_str).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMWriteBitcodeToFile(self.module, c_string.as_ptr()) == 0
        }
    }

    // See GH issue #6
    fn write_bitcode_to_file(&self, file: &File, should_close: bool, unbuffered: bool) -> bool {
        // REVIEW: as_raw_fd docs suggest it only works in *nix
        // Also, should_close should maybe be hardcoded to true?
        unsafe {
            LLVMWriteBitcodeToFD(self.module, file.as_raw_fd(), should_close as i32, unbuffered as i32) == 0
        }
    }

    pub fn write_bitcode_to_memory(&self) -> MemoryBuffer {
        let memory_buffer = unsafe {
            LLVMWriteBitcodeToMemoryBuffer(self.module)
        };

        MemoryBuffer::new(memory_buffer)
    }

    pub fn verify(&self, print: bool) -> bool {
        let mut err_str = unsafe { zeroed() };

        let action = if print {
            LLVMVerifierFailureAction::LLVMPrintMessageAction
        } else {
            LLVMVerifierFailureAction::LLVMReturnStatusAction
        };

        let code = unsafe {
            LLVMVerifyModule(self.module, action, &mut err_str)
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
            LLVMGetDataLayout(self.module) as *mut _
        }
    }

    pub fn get_data_layout(&self) -> &DataLayout {
        self.data_layout.as_ref().expect("DataLayout should always exist until Drop")
    }

    // REVIEW: Ensure the replaced string ptr still gets cleaned up by the module (I think it does)
    // valgrind might come in handy once non jemalloc allocators stabilize
    pub fn set_data_layout(&self, data_layout: &DataLayout) {
        unsafe {
            LLVMSetDataLayout(self.module, data_layout.data_layout.get());
        }

        self.data_layout.as_ref()
                        .expect("DataLayout should always exist until Drop")
                        .data_layout
                        .set(self.get_raw_data_layout());
    }

    pub fn print_to_stderr(&self) {
        unsafe {
            LLVMDumpModule(self.module);
        }
    }

    pub fn print_to_string(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMPrintModuleToString(self.module))
        }
    }

    // REVIEW: Should module take ownership of metadata?
    // REVIEW: Should we return a MetadataValue for the global since it's its own value?
    // it would be the last item in get_global_metadata I believe
    // TODOC: Appends your metadata to a global MetadataValue<Node> indexed by key
    pub fn add_global_metadata(&self, key: &str, metadata: &MetadataValue) {
        let c_string = CString::new(key).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMAddNamedMetadataOperand(self.module, c_string.as_ptr(), metadata.as_value_ref())
        }
    }
    // REVIEW: Better name?
    // TODOC: Gets the size of the metadata node indexed by key
    pub fn get_global_metadata_size(&self, key: &str) -> u32 {
        let c_string = CString::new(key).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMGetNamedMetadataNumOperands(self.module, c_string.as_ptr())
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
            LLVMGetNamedMetadataOperands(self.module, c_string.as_ptr(), ptr);

            from_raw_parts(ptr, count as usize)
        };

        slice.iter().map(|val| MetadataValue::new(*val)).collect()
    }
}

impl Clone for Module {
    fn clone(&self) -> Self {
        let module = unsafe {
            LLVMCloneModule(self.module)
        };

        Module::new(module)
    }
}

// Module owns the data layout string, so LLVMDisposeModule will deallocate it for us,
// so we must call forget to avoid dropping it ourselves
impl Drop for Module {
    fn drop(&mut self) {
        forget(self.data_layout.take().expect("DataLayout should always exist until Drop"));

        unsafe {
            LLVMDisposeModule(self.module);
        }
    }
}
