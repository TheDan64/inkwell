use llvm_sys::LLVMThreadLocalMode;
#[llvm_versions(3.6..8.0)]
use llvm_sys::core::{LLVMGetVisibility, LLVMSetVisibility, LLVMGetSection, LLVMSetSection, LLVMIsExternallyInitialized, LLVMSetExternallyInitialized, LLVMDeleteGlobal, LLVMIsGlobalConstant, LLVMSetGlobalConstant, LLVMGetPreviousGlobal, LLVMGetNextGlobal, LLVMIsThreadLocal, LLVMSetThreadLocal, LLVMGetThreadLocalMode, LLVMSetThreadLocalMode, LLVMGetInitializer, LLVMSetInitializer, LLVMIsDeclaration, LLVMGetDLLStorageClass, LLVMSetDLLStorageClass, LLVMGetAlignment, LLVMSetAlignment, LLVMGetLinkage, LLVMSetLinkage};
#[llvm_versions(8.0..=latest)]
use llvm_sys::core::{LLVMGetVisibility, LLVMSetVisibility, LLVMGetSection, LLVMSetSection, LLVMIsExternallyInitialized, LLVMSetExternallyInitialized, LLVMDeleteGlobal, LLVMIsGlobalConstant, LLVMSetGlobalConstant, LLVMGetPreviousGlobal, LLVMGetNextGlobal, LLVMIsThreadLocal, LLVMSetThreadLocal, LLVMGetThreadLocalMode, LLVMSetThreadLocalMode, LLVMGetInitializer, LLVMSetInitializer, LLVMIsDeclaration, LLVMGetDLLStorageClass, LLVMSetDLLStorageClass, LLVMGetAlignment, LLVMSetAlignment, LLVMGetLinkage, LLVMSetLinkage};
#[llvm_versions(3.6..=6.0)]
use llvm_sys::core::{LLVMHasUnnamedAddr, LLVMSetUnnamedAddr};
#[llvm_versions(7.0..=latest)]
use llvm_sys::core::{LLVMGetUnnamedAddress, LLVMSetUnnamedAddress};
#[llvm_versions(7.0..=latest)]
use llvm_sys::LLVMUnnamedAddr;
#[llvm_versions(8.0..=latest)]
use llvm_sys::core::LLVMGlobalSetMetadata;
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::CStr;

use crate::{GlobalVisibility, ThreadLocalMode, DLLStorageClass};
use crate::module::Linkage;
use crate::support::{to_c_str, LLVMString};
#[llvm_versions(7.0..=latest)]
use crate::comdat::Comdat;
use crate::values::traits::AsValueRef;
use crate::values::{BasicValueEnum, BasicValue, PointerValue, Value};
#[llvm_versions(8.0..=latest)]
use crate::values::MetadataValue;

// REVIEW: GlobalValues are always PointerValues. With SubTypes, we should
// compress this into a PointerValue<Global> type
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct GlobalValue<'ctx> {
    global_value: Value<'ctx>,
}

impl<'ctx> GlobalValue<'ctx> {
    pub(crate) unsafe fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        GlobalValue {
            global_value: Value::new(value),
        }
    }

    /// Gets the name of a `GlobalValue`.
    pub fn get_name(&self) -> &CStr {
        self.global_value.get_name()
    }

    pub fn get_previous_global(self) -> Option<GlobalValue<'ctx>> {
        let value = unsafe {
            LLVMGetPreviousGlobal(self.as_value_ref())
        };

        if value.is_null() {
            return None;
        }

        unsafe {
            Some(GlobalValue::new(value))
        }
    }

    pub fn get_next_global(self) -> Option<GlobalValue<'ctx>> {
        let value = unsafe {
            LLVMGetNextGlobal(self.as_value_ref())
        };

        if value.is_null() {
            return None;
        }

        unsafe {
            Some(GlobalValue::new(value))
        }
    }

    pub fn get_dll_storage_class(self) -> DLLStorageClass {
        let dll_storage_class = unsafe {
            LLVMGetDLLStorageClass(self.as_value_ref())
        };

        DLLStorageClass::new(dll_storage_class)
    }

    pub fn set_dll_storage_class(self, dll_storage_class: DLLStorageClass) {
        unsafe {
            LLVMSetDLLStorageClass(self.as_value_ref(), dll_storage_class.into())
        }
    }

    pub fn get_initializer(self) -> Option<BasicValueEnum<'ctx>> {
        let value = unsafe {
            LLVMGetInitializer(self.as_value_ref())
        };

        if value.is_null() {
            return None;
        }

        unsafe {
            Some(BasicValueEnum::new(value))
        }
    }

    // SubType: This input type should be tied to the BasicType
    pub fn set_initializer(self, value: &dyn BasicValue<'ctx>) {
        unsafe {
            LLVMSetInitializer(self.as_value_ref(), value.as_value_ref())
        }
    }

    pub fn is_thread_local(self) -> bool {
        unsafe {
            LLVMIsThreadLocal(self.as_value_ref()) == 1
        }
    }

    // TODOC: Setting this to true is the same as setting GeneralDynamicTLSModel
    pub fn set_thread_local(self, is_thread_local: bool) {
        unsafe {
            LLVMSetThreadLocal(self.as_value_ref(), is_thread_local as i32)
        }
    }

    pub fn get_thread_local_mode(self) -> Option<ThreadLocalMode> {
        let thread_local_mode = unsafe {
            LLVMGetThreadLocalMode(self.as_value_ref())
        };

        ThreadLocalMode::new(thread_local_mode)
    }

    // REVIEW: Does this have any bad behavior if it isn't thread local or just a noop?
    // or should it call self.set_thread_local(true)?
    pub fn set_thread_local_mode(self, thread_local_mode: Option<ThreadLocalMode>) {
        let thread_local_mode = match thread_local_mode {
            Some(mode) => mode.as_llvm_mode(),
            None => LLVMThreadLocalMode::LLVMNotThreadLocal,
        };

        unsafe {
            LLVMSetThreadLocalMode(self.as_value_ref(), thread_local_mode)
        }
    }

    // SubType: This should be moved into the type. GlobalValue<Initialized/Uninitialized>
    /// Determines whether or not a `GlobalValue` is a declaration or a definition.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_func", fn_type, None);
    ///
    /// assert!(fn_value.as_global_value().is_declaration());
    ///
    /// context.append_basic_block(fn_value, "entry");
    ///
    /// assert!(!fn_value.as_global_value().is_declaration());
    /// ```
    pub fn is_declaration(self) -> bool {
        unsafe {
            LLVMIsDeclaration(self.as_value_ref()) == 1
        }
    }

    #[llvm_versions(3.6..7.0)]
    pub fn has_unnamed_addr(self) -> bool {
        unsafe {
            LLVMHasUnnamedAddr(self.as_value_ref()) == 1
        }
    }

    #[llvm_versions(7.0..=latest)]
    pub fn has_unnamed_addr(self) -> bool {
        unsafe {
            LLVMGetUnnamedAddress(self.as_value_ref()) == LLVMUnnamedAddr::LLVMGlobalUnnamedAddr
        }
    }


    #[llvm_versions(3.6..7.0)]
    pub fn set_unnamed_addr(self, has_unnamed_addr: bool) {
        unsafe {
            LLVMSetUnnamedAddr(self.as_value_ref(), has_unnamed_addr as i32)
        }
    }

    #[llvm_versions(7.0..=latest)]
    pub fn set_unnamed_addr(self, has_unnamed_addr: bool) {
        unsafe {
            if has_unnamed_addr {
                LLVMSetUnnamedAddress(self.as_value_ref(), UnnamedAddress::Global.into())
            } else {
                LLVMSetUnnamedAddress(self.as_value_ref(), UnnamedAddress::None.into())
            }
        }
    }

    pub fn is_constant(self) -> bool {
        unsafe {
            LLVMIsGlobalConstant(self.as_value_ref()) == 1
        }
    }

    pub fn set_constant(self, is_constant: bool) {
        unsafe {
            LLVMSetGlobalConstant(self.as_value_ref(), is_constant as i32)
        }
    }

    pub fn is_externally_initialized(self) -> bool {
        unsafe {
            LLVMIsExternallyInitialized(self.as_value_ref()) == 1
        }
    }

    pub fn set_externally_initialized(self, externally_initialized: bool) {
        unsafe {
            LLVMSetExternallyInitialized(self.as_value_ref(), externally_initialized as i32)
        }
    }

    pub fn set_visibility(self, visibility: GlobalVisibility) {
        unsafe {
            LLVMSetVisibility(self.as_value_ref(), visibility.into())
        }
    }

    pub fn get_visibility(self) -> GlobalVisibility {
        let visibility = unsafe {
            LLVMGetVisibility(self.as_value_ref())
        };

        GlobalVisibility::new(visibility)
    }

    pub fn get_section(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetSection(self.as_value_ref()))
        }
    }

    pub fn set_section(self, section: &str) {
        let c_string = to_c_str(section);

        unsafe {
            LLVMSetSection(self.as_value_ref(), c_string.as_ptr())
        }
    }

    pub unsafe fn delete(self) {
        LLVMDeleteGlobal(self.as_value_ref())
    }

    pub fn as_pointer_value(self) -> PointerValue<'ctx> {
        unsafe {
            PointerValue::new(self.as_value_ref())
        }
    }

    pub fn get_alignment(self) -> u32 {
        unsafe {
            LLVMGetAlignment(self.as_value_ref())
        }
    }

    pub fn set_alignment(self, alignment: u32) {
        unsafe {
            LLVMSetAlignment(self.as_value_ref(), alignment)
        }
    }

    /// Sets a metadata of the given type on the GlobalValue
    #[llvm_versions(8.0..=latest)]
    pub fn set_metadata(self, metadata: MetadataValue<'ctx>, kind_id: u32) {
        unsafe {
            LLVMGlobalSetMetadata(self.as_value_ref(), kind_id, metadata.as_metadata_ref())
        }
    }

    /// Gets a `Comdat` assigned to this `GlobalValue`, if any.
    #[llvm_versions(7.0..=latest)]
    pub fn get_comdat(self) -> Option<Comdat> {
        use llvm_sys::comdat::LLVMGetComdat;

        let comdat_ptr = unsafe {
            LLVMGetComdat(self.as_value_ref())
        };

        if comdat_ptr.is_null() {
            return None;
        }

        Some(Comdat::new(comdat_ptr))
    }

    /// Assigns a `Comdat` to this `GlobalValue`.
    #[llvm_versions(7.0..=latest)]
    pub fn set_comdat(self, comdat: Comdat) {
        use llvm_sys::comdat::LLVMSetComdat;

        unsafe {
            LLVMSetComdat(self.as_value_ref(), comdat.0)
        }
    }

    #[llvm_versions(7.0..=latest)]
    pub fn get_unnamed_address(self) -> UnnamedAddress {
        use llvm_sys::core::LLVMGetUnnamedAddress;

        let unnamed_address = unsafe {
            LLVMGetUnnamedAddress(self.as_value_ref())
        };

        UnnamedAddress::new(unnamed_address)
    }

    #[llvm_versions(7.0..=latest)]
    pub fn set_unnamed_address(self, address: UnnamedAddress) {
        use llvm_sys::core::LLVMSetUnnamedAddress;

        unsafe {
            LLVMSetUnnamedAddress(self.as_value_ref(), address.into())
        }
    }

    pub fn get_linkage(self) -> Linkage {
        unsafe {
            LLVMGetLinkage(self.as_value_ref()).into()
        }
    }

    pub fn set_linkage(self, linkage: Linkage) {
        unsafe {
            LLVMSetLinkage(self.as_value_ref(), linkage.into())
        }
    }

    pub fn print_to_string(self) -> LLVMString {
        self.global_value.print_to_string()
    }
}

impl AsValueRef for GlobalValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.global_value.value
    }
}

/// This enum determines the significance of a `GlobalValue`'s address.
#[llvm_versions(7.0..=latest)]
#[llvm_enum(LLVMUnnamedAddr)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UnnamedAddress {
    /// Address of the `GlobalValue` is significant.
    #[llvm_variant(LLVMNoUnnamedAddr)]
    None,

    /// Address of the `GlobalValue` is locally insignificant.
    #[llvm_variant(LLVMLocalUnnamedAddr)]
    Local,

    /// Address of the `GlobalValue` is globally insignificant.
    #[llvm_variant(LLVMGlobalUnnamedAddr)]
    Global,
}
