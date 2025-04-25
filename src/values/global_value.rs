use llvm_sys::core::LLVMGlobalSetMetadata;

use llvm_sys::core::{
    LLVMDeleteGlobal, LLVMGetAlignment, LLVMGetDLLStorageClass, LLVMGetInitializer, LLVMGetLinkage, LLVMGetNextGlobal,
    LLVMGetPreviousGlobal, LLVMGetThreadLocalMode, LLVMGetVisibility, LLVMIsDeclaration, LLVMIsExternallyInitialized,
    LLVMIsGlobalConstant, LLVMIsThreadLocal, LLVMSetAlignment, LLVMSetDLLStorageClass, LLVMSetExternallyInitialized,
    LLVMSetGlobalConstant, LLVMSetInitializer, LLVMSetLinkage, LLVMSetThreadLocal, LLVMSetThreadLocalMode,
    LLVMSetVisibility,
};

use llvm_sys::core::{LLVMGetUnnamedAddress, LLVMSetUnnamedAddress};
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMThreadLocalMode;

use llvm_sys::LLVMUnnamedAddr;

use std::ffi::CStr;
use std::fmt::{self, Display};

use crate::comdat::Comdat;
use crate::module::Linkage;
use crate::types::AnyTypeEnum;
use crate::values::traits::AsValueRef;

use crate::values::MetadataValue;
use crate::values::{BasicValue, BasicValueEnum, PointerValue, Value};
use crate::{DLLStorageClass, GlobalVisibility, ThreadLocalMode};

use super::AnyValue;

// REVIEW: GlobalValues are always PointerValues. With SubTypes, we should
// compress this into a PointerValue<Global> type
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct GlobalValue<'ctx> {
    global_value: Value<'ctx>,
}

impl<'ctx> GlobalValue<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of type global.
    pub unsafe fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        GlobalValue {
            global_value: Value::new(value),
        }
    }

    /// Get name of the `GlobalValue`.
    pub fn get_name(&self) -> &CStr {
        self.global_value.get_name()
    }

    /// Set name of the `GlobalValue`.
    pub fn set_name(&self, name: &str) {
        self.global_value.set_name(name)
    }

    pub fn get_previous_global(self) -> Option<GlobalValue<'ctx>> {
        let value = unsafe { LLVMGetPreviousGlobal(self.as_value_ref()) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(GlobalValue::new(value)) }
    }

    pub fn get_next_global(self) -> Option<GlobalValue<'ctx>> {
        let value = unsafe { LLVMGetNextGlobal(self.as_value_ref()) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(GlobalValue::new(value)) }
    }

    pub fn get_dll_storage_class(self) -> DLLStorageClass {
        let dll_storage_class = unsafe { LLVMGetDLLStorageClass(self.as_value_ref()) };

        DLLStorageClass::new(dll_storage_class)
    }

    pub fn set_dll_storage_class(self, dll_storage_class: DLLStorageClass) {
        unsafe { LLVMSetDLLStorageClass(self.as_value_ref(), dll_storage_class.into()) }
    }

    pub fn get_initializer(self) -> Option<BasicValueEnum<'ctx>> {
        let value = unsafe { LLVMGetInitializer(self.as_value_ref()) };

        if value.is_null() {
            return None;
        }

        unsafe { Some(BasicValueEnum::new(value)) }
    }

    // SubType: This input type should be tied to the BasicType
    pub fn set_initializer(self, value: &dyn BasicValue<'ctx>) {
        unsafe { LLVMSetInitializer(self.as_value_ref(), value.as_value_ref()) }
    }

    pub fn is_thread_local(self) -> bool {
        unsafe { LLVMIsThreadLocal(self.as_value_ref()) == 1 }
    }

    // TODOC: Setting this to true is the same as setting GeneralDynamicTLSModel
    pub fn set_thread_local(self, is_thread_local: bool) {
        unsafe { LLVMSetThreadLocal(self.as_value_ref(), is_thread_local as i32) }
    }

    pub fn get_thread_local_mode(self) -> Option<ThreadLocalMode> {
        let thread_local_mode = unsafe { LLVMGetThreadLocalMode(self.as_value_ref()) };

        ThreadLocalMode::new(thread_local_mode)
    }

    // REVIEW: Does this have any bad behavior if it isn't thread local or just a noop?
    // or should it call self.set_thread_local(true)?
    pub fn set_thread_local_mode(self, thread_local_mode: Option<ThreadLocalMode>) {
        let thread_local_mode = match thread_local_mode {
            Some(mode) => mode.as_llvm_mode(),
            None => LLVMThreadLocalMode::LLVMNotThreadLocal,
        };

        unsafe { LLVMSetThreadLocalMode(self.as_value_ref(), thread_local_mode) }
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
        unsafe { LLVMIsDeclaration(self.as_value_ref()) == 1 }
    }

    pub fn has_unnamed_addr(self) -> bool {
        unsafe { LLVMGetUnnamedAddress(self.as_value_ref()) == LLVMUnnamedAddr::LLVMGlobalUnnamedAddr }
    }

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
        unsafe { LLVMIsGlobalConstant(self.as_value_ref()) == 1 }
    }

    pub fn set_constant(self, is_constant: bool) {
        unsafe { LLVMSetGlobalConstant(self.as_value_ref(), is_constant as i32) }
    }

    pub fn is_externally_initialized(self) -> bool {
        unsafe { LLVMIsExternallyInitialized(self.as_value_ref()) == 1 }
    }

    pub fn set_externally_initialized(self, externally_initialized: bool) {
        unsafe { LLVMSetExternallyInitialized(self.as_value_ref(), externally_initialized as i32) }
    }

    pub fn set_visibility(self, visibility: GlobalVisibility) {
        unsafe { LLVMSetVisibility(self.as_value_ref(), visibility.into()) }
    }

    pub fn get_visibility(self) -> GlobalVisibility {
        let visibility = unsafe { LLVMGetVisibility(self.as_value_ref()) };

        GlobalVisibility::new(visibility)
    }

    /// Get section, this global value belongs to
    pub fn get_section(&self) -> Option<&CStr> {
        self.global_value.get_section()
    }

    /// Set section, this global value belongs to
    pub fn set_section(self, section: Option<&str>) {
        self.global_value.set_section(section)
    }

    pub unsafe fn delete(self) {
        LLVMDeleteGlobal(self.as_value_ref())
    }

    pub fn as_pointer_value(self) -> PointerValue<'ctx> {
        unsafe { PointerValue::new(self.as_value_ref()) }
    }

    pub fn get_alignment(self) -> u32 {
        unsafe { LLVMGetAlignment(self.as_value_ref()) }
    }

    pub fn set_alignment(self, alignment: u32) {
        unsafe { LLVMSetAlignment(self.as_value_ref(), alignment) }
    }

    /// Sets a metadata of the given type on the GlobalValue
    pub fn set_metadata(self, metadata: MetadataValue<'ctx>, kind_id: u32) {
        unsafe { LLVMGlobalSetMetadata(self.as_value_ref(), kind_id, metadata.as_metadata_ref()) }
    }

    /// Gets a `Comdat` assigned to this `GlobalValue`, if any.
    pub fn get_comdat(self) -> Option<Comdat> {
        use llvm_sys::comdat::LLVMGetComdat;

        let comdat_ptr = unsafe { LLVMGetComdat(self.as_value_ref()) };

        if comdat_ptr.is_null() {
            return None;
        }

        unsafe { Some(Comdat::new(comdat_ptr)) }
    }

    /// Assigns a `Comdat` to this `GlobalValue`.
    pub fn set_comdat(self, comdat: Comdat) {
        use llvm_sys::comdat::LLVMSetComdat;

        unsafe { LLVMSetComdat(self.as_value_ref(), comdat.0) }
    }

    pub fn get_unnamed_address(self) -> UnnamedAddress {
        use llvm_sys::core::LLVMGetUnnamedAddress;

        let unnamed_address = unsafe { LLVMGetUnnamedAddress(self.as_value_ref()) };

        UnnamedAddress::new(unnamed_address)
    }

    pub fn set_unnamed_address(self, address: UnnamedAddress) {
        use llvm_sys::core::LLVMSetUnnamedAddress;

        unsafe { LLVMSetUnnamedAddress(self.as_value_ref(), address.into()) }
    }

    pub fn get_linkage(self) -> Linkage {
        unsafe { LLVMGetLinkage(self.as_value_ref()).into() }
    }

    pub fn set_linkage(self, linkage: Linkage) {
        unsafe { LLVMSetLinkage(self.as_value_ref(), linkage.into()) }
    }

    pub fn get_value_type(self) -> AnyTypeEnum<'ctx> {
        unsafe { AnyTypeEnum::new(llvm_sys::core::LLVMGlobalGetValueType(self.as_value_ref())) }
    }
}

unsafe impl AsValueRef for GlobalValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.global_value.value
    }
}

impl Display for GlobalValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

/// This enum determines the significance of a `GlobalValue`'s address.

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
