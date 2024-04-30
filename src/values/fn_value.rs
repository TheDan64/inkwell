use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMViewFunctionCFG, LLVMViewFunctionCFGOnly};
use llvm_sys::core::{
    LLVMAddAttributeAtIndex, LLVMGetAttributeCountAtIndex, LLVMGetEnumAttributeAtIndex, LLVMGetStringAttributeAtIndex,
    LLVMRemoveEnumAttributeAtIndex, LLVMRemoveStringAttributeAtIndex,
};
use llvm_sys::core::{
    LLVMCountBasicBlocks, LLVMCountParams, LLVMDeleteFunction, LLVMGetBasicBlocks, LLVMGetFirstBasicBlock,
    LLVMGetFirstParam, LLVMGetFunctionCallConv, LLVMGetGC, LLVMGetIntrinsicID, LLVMGetLastBasicBlock, LLVMGetLastParam,
    LLVMGetLinkage, LLVMGetNextFunction, LLVMGetNextParam, LLVMGetParam, LLVMGetParams, LLVMGetPreviousFunction,
    LLVMIsAFunction, LLVMIsConstant, LLVMSetFunctionCallConv, LLVMSetGC, LLVMSetLinkage, LLVMSetParamAlignment,
};
use llvm_sys::core::{LLVMGetPersonalityFn, LLVMSetPersonalityFn};
#[llvm_versions(7..)]
use llvm_sys::debuginfo::{LLVMGetSubprogram, LLVMSetSubprogram};
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMValueRef};

use std::ffi::CStr;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::mem::forget;

use crate::attributes::{Attribute, AttributeLoc};
use crate::basic_block::BasicBlock;
#[llvm_versions(7..)]
use crate::debug_info::DISubprogram;
use crate::module::Linkage;
use crate::support::to_c_str;
use crate::types::FunctionType;
use crate::values::traits::{AnyValue, AsValueRef};
use crate::values::{BasicValueEnum, GlobalValue, Value};

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct FunctionValue<'ctx> {
    fn_value: Value<'ctx>,
}

impl<'ctx> FunctionValue<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of type function.
    pub unsafe fn new(value: LLVMValueRef) -> Option<Self> {
        if value.is_null() {
            return None;
        }

        assert!(!LLVMIsAFunction(value).is_null());

        Some(FunctionValue {
            fn_value: Value::new(value),
        })
    }

    pub fn get_linkage(self) -> Linkage {
        unsafe { LLVMGetLinkage(self.as_value_ref()).into() }
    }

    pub fn set_linkage(self, linkage: Linkage) {
        unsafe { LLVMSetLinkage(self.as_value_ref(), linkage.into()) }
    }

    pub fn is_null(self) -> bool {
        self.fn_value.is_null()
    }

    pub fn is_undef(self) -> bool {
        self.fn_value.is_undef()
    }

    pub fn print_to_stderr(self) {
        self.fn_value.print_to_stderr()
    }

    // FIXME: Better error returns, code 1 is error
    pub fn verify(self, print: bool) -> bool {
        let action = if print {
            LLVMVerifierFailureAction::LLVMPrintMessageAction
        } else {
            LLVMVerifierFailureAction::LLVMReturnStatusAction
        };

        let code = unsafe { LLVMVerifyFunction(self.fn_value.value, action) };

        code != 1
    }

    // REVIEW: If there's a demand, could easily create a module.get_functions() -> Iterator
    pub fn get_next_function(self) -> Option<Self> {
        unsafe { FunctionValue::new(LLVMGetNextFunction(self.as_value_ref())) }
    }

    pub fn get_previous_function(self) -> Option<Self> {
        unsafe { FunctionValue::new(LLVMGetPreviousFunction(self.as_value_ref())) }
    }

    pub fn get_first_param(self) -> Option<BasicValueEnum<'ctx>> {
        let param = unsafe { LLVMGetFirstParam(self.as_value_ref()) };

        if param.is_null() {
            return None;
        }

        unsafe { Some(BasicValueEnum::new(param)) }
    }

    pub fn get_last_param(self) -> Option<BasicValueEnum<'ctx>> {
        let param = unsafe { LLVMGetLastParam(self.as_value_ref()) };

        if param.is_null() {
            return None;
        }

        unsafe { Some(BasicValueEnum::new(param)) }
    }

    pub fn get_first_basic_block(self) -> Option<BasicBlock<'ctx>> {
        unsafe { BasicBlock::new(LLVMGetFirstBasicBlock(self.as_value_ref())) }
    }

    pub fn get_nth_param(self, nth: u32) -> Option<BasicValueEnum<'ctx>> {
        let count = self.count_params();

        if nth + 1 > count {
            return None;
        }

        unsafe { Some(BasicValueEnum::new(LLVMGetParam(self.as_value_ref(), nth))) }
    }

    pub fn count_params(self) -> u32 {
        unsafe { LLVMCountParams(self.fn_value.value) }
    }

    pub fn count_basic_blocks(self) -> u32 {
        unsafe { LLVMCountBasicBlocks(self.as_value_ref()) }
    }

    pub fn get_basic_block_iter(self) -> BasicBlockIter<'ctx> {
        BasicBlockIter(self.get_first_basic_block())
    }

    pub fn get_basic_blocks(self) -> Vec<BasicBlock<'ctx>> {
        let count = self.count_basic_blocks();
        let mut raw_vec: Vec<LLVMBasicBlockRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetBasicBlocks(self.as_value_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec
            .iter()
            .map(|val| unsafe { BasicBlock::new(*val).unwrap() })
            .collect()
    }

    pub fn get_param_iter(self) -> ParamValueIter<'ctx> {
        ParamValueIter {
            param_iter_value: self.fn_value.value,
            start: true,
            _marker: PhantomData,
        }
    }

    pub fn get_params(self) -> Vec<BasicValueEnum<'ctx>> {
        let count = self.count_params();
        let mut raw_vec: Vec<LLVMValueRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetParams(self.as_value_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| unsafe { BasicValueEnum::new(*val) }).collect()
    }

    pub fn get_last_basic_block(self) -> Option<BasicBlock<'ctx>> {
        unsafe { BasicBlock::new(LLVMGetLastBasicBlock(self.fn_value.value)) }
    }

    /// Gets the name of a `FunctionValue`.
    pub fn get_name(&self) -> &CStr {
        self.fn_value.get_name()
    }

    /// View the control flow graph and produce a .dot file
    pub fn view_function_cfg(self) {
        unsafe { LLVMViewFunctionCFG(self.as_value_ref()) }
    }

    /// Only view the control flow graph
    pub fn view_function_cfg_only(self) {
        unsafe { LLVMViewFunctionCFGOnly(self.as_value_ref()) }
    }

    // TODO: Look for ways to prevent use after delete but maybe not possible
    pub unsafe fn delete(self) {
        LLVMDeleteFunction(self.as_value_ref())
    }

    #[llvm_versions(..=7)]
    pub fn get_type(self) -> FunctionType<'ctx> {
        use crate::types::PointerType;

        let ptr_type = unsafe { PointerType::new(self.fn_value.get_type()) };

        ptr_type.get_element_type().into_function_type()
    }

    #[llvm_versions(8..)]
    pub fn get_type(self) -> FunctionType<'ctx> {
        unsafe { FunctionType::new(llvm_sys::core::LLVMGlobalGetValueType(self.as_value_ref())) }
    }

    // TODOC: How this works as an exception handler
    pub fn has_personality_function(self) -> bool {
        use llvm_sys::core::LLVMHasPersonalityFn;

        unsafe { LLVMHasPersonalityFn(self.as_value_ref()) == 1 }
    }

    pub fn get_personality_function(self) -> Option<FunctionValue<'ctx>> {
        // This prevents a segfault when not having a pfn
        if !self.has_personality_function() {
            return None;
        }

        unsafe { FunctionValue::new(LLVMGetPersonalityFn(self.as_value_ref())) }
    }

    pub fn set_personality_function(self, personality_fn: FunctionValue<'ctx>) {
        unsafe { LLVMSetPersonalityFn(self.as_value_ref(), personality_fn.as_value_ref()) }
    }

    pub fn get_intrinsic_id(self) -> u32 {
        unsafe { LLVMGetIntrinsicID(self.as_value_ref()) }
    }

    pub fn get_call_conventions(self) -> u32 {
        unsafe { LLVMGetFunctionCallConv(self.as_value_ref()) }
    }

    pub fn set_call_conventions(self, call_conventions: u32) {
        unsafe { LLVMSetFunctionCallConv(self.as_value_ref(), call_conventions) }
    }

    pub fn get_gc(&self) -> &CStr {
        unsafe { CStr::from_ptr(LLVMGetGC(self.as_value_ref())) }
    }

    pub fn set_gc(self, gc: &str) {
        let c_string = to_c_str(gc);

        unsafe { LLVMSetGC(self.as_value_ref(), c_string.as_ptr()) }
    }

    pub fn replace_all_uses_with(self, other: FunctionValue<'ctx>) {
        self.fn_value.replace_all_uses_with(other.as_value_ref())
    }

    /// Adds an `Attribute` to a particular location in this `FunctionValue`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    ///
    /// fn_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// fn_value.add_attribute(AttributeLoc::Return, enum_attribute);
    /// ```
    pub fn add_attribute(self, loc: AttributeLoc, attribute: Attribute) {
        unsafe { LLVMAddAttributeAtIndex(self.as_value_ref(), loc.get_index(), attribute.attribute) }
    }

    /// Counts the number of `Attribute`s belonging to the specified location in this `FunctionValue`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    ///
    /// fn_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// fn_value.add_attribute(AttributeLoc::Return, enum_attribute);
    ///
    /// assert_eq!(fn_value.count_attributes(AttributeLoc::Return), 2);
    /// ```
    pub fn count_attributes(self, loc: AttributeLoc) -> u32 {
        unsafe { LLVMGetAttributeCountAtIndex(self.as_value_ref(), loc.get_index()) }
    }

    /// Get all `Attribute`s belonging to the specified location in this `FunctionValue`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    ///
    /// fn_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// fn_value.add_attribute(AttributeLoc::Return, enum_attribute);
    ///
    /// assert_eq!(fn_value.attributes(AttributeLoc::Return), vec![string_attribute, enum_attribute]);
    /// ```
    pub fn attributes(self, loc: AttributeLoc) -> Vec<Attribute> {
        use llvm_sys::core::LLVMGetAttributesAtIndex;
        use std::mem::{ManuallyDrop, MaybeUninit};

        let count = self.count_attributes(loc) as usize;

        // initialize a vector, but leave each element uninitialized
        let mut attribute_refs: Vec<MaybeUninit<Attribute>> = vec![MaybeUninit::uninit(); count];

        // Safety: relies on `Attribute` having the same in-memory representation as `LLVMAttributeRef`
        unsafe {
            LLVMGetAttributesAtIndex(
                self.as_value_ref(),
                loc.get_index(),
                attribute_refs.as_mut_ptr() as *mut _,
            )
        }

        // Safety: all elements are initialized
        unsafe {
            // ensure the vector is not dropped
            let mut attribute_refs = ManuallyDrop::new(attribute_refs);

            Vec::from_raw_parts(
                attribute_refs.as_mut_ptr() as *mut Attribute,
                attribute_refs.len(),
                attribute_refs.capacity(),
            )
        }
    }

    /// Removes a string `Attribute` belonging to the specified location in this `FunctionValue`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    ///
    /// fn_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// fn_value.remove_string_attribute(AttributeLoc::Return, "my_key");
    /// ```
    pub fn remove_string_attribute(self, loc: AttributeLoc, key: &str) {
        unsafe {
            LLVMRemoveStringAttributeAtIndex(
                self.as_value_ref(),
                loc.get_index(),
                key.as_ptr() as *const ::libc::c_char,
                key.len() as u32,
            )
        }
    }

    /// Removes an enum `Attribute` belonging to the specified location in this `FunctionValue`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    ///
    /// fn_value.add_attribute(AttributeLoc::Return, enum_attribute);
    /// fn_value.remove_enum_attribute(AttributeLoc::Return, 1);
    /// ```
    pub fn remove_enum_attribute(self, loc: AttributeLoc, kind_id: u32) {
        unsafe { LLVMRemoveEnumAttributeAtIndex(self.as_value_ref(), loc.get_index(), kind_id) }
    }

    /// Gets an enum `Attribute` belonging to the specified location in this `FunctionValue`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    ///
    /// fn_value.add_attribute(AttributeLoc::Return, enum_attribute);
    ///
    /// assert_eq!(fn_value.get_enum_attribute(AttributeLoc::Return, 1), Some(enum_attribute));
    /// ```
    // SubTypes: -> Option<Attribute<Enum>>
    pub fn get_enum_attribute(self, loc: AttributeLoc, kind_id: u32) -> Option<Attribute> {
        let ptr = unsafe { LLVMGetEnumAttributeAtIndex(self.as_value_ref(), loc.get_index(), kind_id) };

        if ptr.is_null() {
            return None;
        }

        unsafe { Some(Attribute::new(ptr)) }
    }

    /// Gets a string `Attribute` belonging to the specified location in this `FunctionValue`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    ///
    /// fn_value.add_attribute(AttributeLoc::Return, string_attribute);
    ///
    /// assert_eq!(fn_value.get_string_attribute(AttributeLoc::Return, "my_key"), Some(string_attribute));
    /// ```
    // SubTypes: -> Option<Attribute<String>>
    pub fn get_string_attribute(self, loc: AttributeLoc, key: &str) -> Option<Attribute> {
        let ptr = unsafe {
            LLVMGetStringAttributeAtIndex(
                self.as_value_ref(),
                loc.get_index(),
                key.as_ptr() as *const ::libc::c_char,
                key.len() as u32,
            )
        };

        if ptr.is_null() {
            return None;
        }

        unsafe { Some(Attribute::new(ptr)) }
    }

    pub fn set_param_alignment(self, param_index: u32, alignment: u32) {
        if let Some(param) = self.get_nth_param(param_index) {
            unsafe { LLVMSetParamAlignment(param.as_value_ref(), alignment) }
        }
    }

    /// Gets the `GlobalValue` version of this `FunctionValue`. This allows
    /// you to further inspect its global properties or even convert it to
    /// a `PointerValue`.
    pub fn as_global_value(self) -> GlobalValue<'ctx> {
        unsafe { GlobalValue::new(self.as_value_ref()) }
    }

    /// Set the debug info descriptor
    #[llvm_versions(7..)]
    pub fn set_subprogram(self, subprogram: DISubprogram<'ctx>) {
        unsafe { LLVMSetSubprogram(self.as_value_ref(), subprogram.metadata_ref) }
    }

    /// Get the debug info descriptor
    #[llvm_versions(7..)]
    pub fn get_subprogram(self) -> Option<DISubprogram<'ctx>> {
        let metadata_ref = unsafe { LLVMGetSubprogram(self.as_value_ref()) };

        if metadata_ref.is_null() {
            None
        } else {
            Some(DISubprogram {
                metadata_ref,
                _marker: PhantomData,
            })
        }
    }

    /// Get the section to which this function belongs
    pub fn get_section(&self) -> Option<&CStr> {
        self.fn_value.get_section()
    }

    /// Set the section to which this function should belong
    pub fn set_section(self, section: Option<&str>) {
        self.fn_value.set_section(section)
    }
}

unsafe impl AsValueRef for FunctionValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.fn_value.value
    }
}

impl Display for FunctionValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl fmt::Debug for FunctionValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = self.print_to_string();
        let llvm_type = self.get_type();
        let name = self.get_name();
        let is_const = unsafe { LLVMIsConstant(self.fn_value.value) == 1 };
        let is_null = self.is_null();

        f.debug_struct("FunctionValue")
            .field("name", &name)
            .field("address", &self.as_value_ref())
            .field("is_const", &is_const)
            .field("is_null", &is_null)
            .field("llvm_value", &llvm_value)
            .field("llvm_type", &llvm_type.print_to_string())
            .finish()
    }
}

/// Iterate over all `BasicBlock`s in a function.
#[derive(Debug)]
pub struct BasicBlockIter<'ctx>(Option<BasicBlock<'ctx>>);

impl<'ctx> Iterator for BasicBlockIter<'ctx> {
    type Item = BasicBlock<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(bb) = self.0 {
            self.0 = bb.get_next_basic_block();
            Some(bb)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ParamValueIter<'ctx> {
    param_iter_value: LLVMValueRef,
    start: bool,
    _marker: PhantomData<&'ctx ()>,
}

impl<'ctx> Iterator for ParamValueIter<'ctx> {
    type Item = BasicValueEnum<'ctx>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start {
            let first_value = unsafe { LLVMGetFirstParam(self.param_iter_value) };

            if first_value.is_null() {
                return None;
            }

            self.start = false;

            self.param_iter_value = first_value;

            return unsafe { Some(Self::Item::new(first_value)) };
        }

        let next_value = unsafe { LLVMGetNextParam(self.param_iter_value) };

        if next_value.is_null() {
            return None;
        }

        self.param_iter_value = next_value;

        unsafe { Some(Self::Item::new(next_value)) }
    }
}
