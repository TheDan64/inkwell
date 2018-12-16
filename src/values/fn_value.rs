use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMViewFunctionCFG, LLVMViewFunctionCFGOnly};
use llvm_sys::core::{LLVMIsAFunction, LLVMIsConstant, LLVMGetLinkage, LLVMTypeOf, LLVMGetPreviousFunction, LLVMGetNextFunction, LLVMGetParam, LLVMCountParams, LLVMGetLastParam, LLVMCountBasicBlocks, LLVMGetFirstParam, LLVMGetNextParam, LLVMGetBasicBlocks, LLVMGetReturnType, LLVMAppendBasicBlock, LLVMDeleteFunction, LLVMGetElementType, LLVMGetLastBasicBlock, LLVMGetFirstBasicBlock, LLVMGetEntryBasicBlock, LLVMGetIntrinsicID, LLVMGetFunctionCallConv, LLVMSetFunctionCallConv, LLVMGetGC, LLVMSetGC, LLVMSetLinkage, LLVMSetParamAlignment, LLVMGetParams};
#[llvm_versions(3.7 => latest)]
use llvm_sys::core::{LLVMGetPersonalityFn, LLVMSetPersonalityFn};
#[llvm_versions(3.9 => latest)]
use llvm_sys::core::{LLVMAddAttributeAtIndex, LLVMGetAttributeCountAtIndex, LLVMGetEnumAttributeAtIndex, LLVMGetStringAttributeAtIndex, LLVMRemoveEnumAttributeAtIndex, LLVMRemoveStringAttributeAtIndex};
use llvm_sys::prelude::{LLVMValueRef, LLVMBasicBlockRef};

use std::ffi::{CStr, CString};
use std::mem::forget;
use std::fmt;

#[llvm_versions(3.9 => latest)]
use attributes::Attribute;
use basic_block::BasicBlock;
use module::Linkage;
use support::LLVMString;
use types::{BasicTypeEnum, FunctionType};
use values::traits::AsValueRef;
use values::{BasicValueEnum, GlobalValue, Value, MetadataValue};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FunctionValue {
    fn_value: Value,
}

impl FunctionValue {
    pub(crate) fn new(value: LLVMValueRef) -> Option<Self> {
        if value.is_null() {
            return None;
        }

        unsafe {
            assert!(!LLVMIsAFunction(value).is_null())
        }

        Some(FunctionValue { fn_value: Value::new(value) })
    }

    pub fn get_linkage(&self) -> Linkage {
        let linkage = unsafe {
            LLVMGetLinkage(self.as_value_ref())
        };

        Linkage::new(linkage)
    }

    pub fn set_linkage(&self, linkage: Linkage) {
        unsafe {
            LLVMSetLinkage(self.as_value_ref(), linkage.as_llvm_enum())
        }
    }

    pub fn is_null(&self) -> bool {
        self.fn_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.fn_value.is_undef()
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.fn_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.fn_value.print_to_stderr()
    }

    // FIXME: Better error returns, code 1 is error
    pub fn verify(&self, print: bool) -> bool {
        let action = if print {
            LLVMVerifierFailureAction::LLVMPrintMessageAction
        } else {
            LLVMVerifierFailureAction::LLVMReturnStatusAction
        };

        let code = unsafe {
            LLVMVerifyFunction(self.fn_value.value, action)
        };

        code != 1
    }

    // REVIEW: If there's a demand, could easily create a module.get_functions() -> Iterator
    pub fn get_next_function(&self) -> Option<Self> {
        let function = unsafe {
            LLVMGetNextFunction(self.as_value_ref())
        };

        FunctionValue::new(function)
    }

    pub fn get_previous_function(&self) -> Option<Self> {
        let function = unsafe {
            LLVMGetPreviousFunction(self.as_value_ref())
        };

        FunctionValue::new(function)
    }

    pub fn get_first_param(&self) -> Option<BasicValueEnum> {
        let param = unsafe {
            LLVMGetFirstParam(self.as_value_ref())
        };

        if param.is_null() {
            return None;
        }

        Some(BasicValueEnum::new(param))
    }

    pub fn get_last_param(&self) -> Option<BasicValueEnum> {
        let param = unsafe {
            LLVMGetLastParam(self.as_value_ref())
        };

        if param.is_null() {
            return None;
        }

        Some(BasicValueEnum::new(param))
    }

    // REVIEW: Odd behavior, a function with no BBs returns a ptr
    // that isn't actually a basic_block and seems to get corrupted
    // Should check filed LLVM bugs - maybe just return None since
    // we can catch it with "LLVMIsABasicBlock"
    pub fn get_entry_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetEntryBasicBlock(self.as_value_ref())
        };

        BasicBlock::new(bb)
    }

    // REVIEW: Odd behavior, a function with no BBs returns a ptr
    // that isn't actually a basic_block and seems to get corrupted
    // Should check filed LLVM bugs - maybe just return None since
    // we can catch it with "LLVMIsABasicBlock"
    pub fn get_first_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetFirstBasicBlock(self.as_value_ref())
        };

        BasicBlock::new(bb)
    }

    // TODOC: This applies the global context to the basic block. To keep the existing context
    // prefer context.append_basic_block()
    pub fn append_basic_block(&self, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMAppendBasicBlock(self.as_value_ref(), c_string.as_ptr())
        };

        BasicBlock::new(bb).expect("Appending basic block should never fail")
    }

    pub fn get_nth_param(&self, nth: u32) -> Option<BasicValueEnum> {
        let count = self.count_params();

        if nth + 1 > count {
            return None;
        }

        let param = unsafe {
            LLVMGetParam(self.as_value_ref(), nth)
        };

        Some(BasicValueEnum::new(param))
    }

    pub fn count_params(&self) -> u32 {
        unsafe {
            LLVMCountParams(self.fn_value.value)
        }
    }

    pub fn count_basic_blocks(&self) -> u32 {
        unsafe {
            LLVMCountBasicBlocks(self.as_value_ref())
        }
    }

    pub fn get_basic_blocks(&self) -> Vec<BasicBlock> {
        let count = self.count_basic_blocks();
        let mut raw_vec: Vec<LLVMBasicBlockRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetBasicBlocks(self.as_value_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| BasicBlock::new(*val).unwrap()).collect()
    }

    pub fn get_return_type(&self) -> BasicTypeEnum {
        let type_ = unsafe {
            LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(self.fn_value.value)))
        };

        BasicTypeEnum::new(type_)
    }

    pub fn get_param_iter(&self) -> ParamValueIter {
        ParamValueIter {
            param_iter_value: self.fn_value.value,
            start: true,
        }
    }

    pub fn get_params(&self) -> Vec<BasicValueEnum> {
        let count = self.count_params();
        let mut raw_vec: Vec<LLVMValueRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetParams(self.as_value_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| BasicValueEnum::new(*val)).collect()
    }

    pub fn get_last_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetLastBasicBlock(self.fn_value.value)
        };

        BasicBlock::new(bb)
    }

    pub fn get_name(&self) -> &CStr {
        self.fn_value.get_name()
    }

    pub fn view_function_config(&self) {
        unsafe {
            LLVMViewFunctionCFG(self.as_value_ref())
        }
    }

    pub fn view_function_config_only(&self) {
        unsafe {
            LLVMViewFunctionCFGOnly(self.as_value_ref())
        }
    }

    // TODO: Look for ways to prevent use after delete but maybe not possible
    pub unsafe fn delete(self) {
        LLVMDeleteFunction(self.as_value_ref())
    }

    pub fn get_type(&self) -> FunctionType {
        FunctionType::new(self.fn_value.get_type())
    }

    pub fn has_metadata(&self) -> bool {
        self.fn_value.has_metadata()
    }

    pub fn get_metadata(&self, kind_id: u32) -> Option<MetadataValue> {
        self.fn_value.get_metadata(kind_id)
    }

    pub fn set_metadata(&self, metadata: MetadataValue, kind_id: u32) {
        self.fn_value.set_metadata(metadata, kind_id)
    }

    // TODOC: How this works as an exception handler
    #[llvm_versions(3.9 => latest)]
    pub fn has_personality_function(&self) -> bool {
        use llvm_sys::core::LLVMHasPersonalityFn;

        unsafe {
            LLVMHasPersonalityFn(self.as_value_ref()) == 1
        }
    }

    #[cfg(not(any(feature = "llvm3-6", feature = "llvm3-8")))]
    pub fn get_personality_function(&self) -> Option<FunctionValue> {
        // This prevents a segfault in 3.9+ when not having a pfn
        // however that segfault will unforuntately still happen in 3.8
        // because LLVMHasPersonalityFn doesn't exist yet :(
        #[cfg(not(any(feature = "llvm3-7", feature = "llvm3-8")))]
        {
            if !self.has_personality_function() {
                return None;
            }
        }

        let value = unsafe {
            LLVMGetPersonalityFn(self.as_value_ref())
        };

        FunctionValue::new(value)
    }

    // TODOC: This function will segfault in 3.8 due to a LLVM bug when
    // there is no personality fn. This segfault is worked around and
    // avoided in later LLVM versions. Therefore this fn is unsafe in 3.8
    // but not in all other versions
    #[cfg(feature = "llvm3-8")]
    pub unsafe fn get_personality_function(&self) -> Option<FunctionValue> {
        let value = unsafe {
            LLVMGetPersonalityFn(self.as_value_ref())
        };

        FunctionValue::new(value)
    }

    #[llvm_versions(3.7 => latest)]
    pub fn set_personality_function(&self, personality_fn: FunctionValue) {
        unsafe {
            LLVMSetPersonalityFn(self.as_value_ref(), personality_fn.as_value_ref())
        }
    }

    pub fn get_intrinsic_id(&self) -> u32 {
        unsafe {
            LLVMGetIntrinsicID(self.as_value_ref())
        }
    }

    pub fn get_call_conventions(&self) -> u32 {
        unsafe {
            LLVMGetFunctionCallConv(self.as_value_ref())
        }
    }

    pub fn set_call_conventions(&self, call_conventions: u32) {
        unsafe {
            LLVMSetFunctionCallConv(self.as_value_ref(), call_conventions)
        }
    }

    pub fn get_gc(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetGC(self.as_value_ref()))
        }
    }

    pub fn set_gc(&self, gc: &str) {
        let c_string = CString::new(gc).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetGC(self.as_value_ref(), c_string.as_ptr())
        }
    }

    pub fn replace_all_uses_with(&self, other: FunctionValue) {
        self.fn_value.replace_all_uses_with(other.as_value_ref())
    }

    /// Adds an `Attribute` to this `FunctionValue` if index is 0, otherwise to its parameter
    /// in the 1 - Nth position.
    ///
    /// # Example
    ///
    /// ```no_run
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
    /// fn_value.add_attribute(0, string_attribute);
    /// fn_value.add_attribute(0, enum_attribute);
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn add_attribute(&self, index: u32, attribute: Attribute) {
        unsafe {
            LLVMAddAttributeAtIndex(self.as_value_ref(), index, attribute.attribute)
        }
    }

    /// Counts the number of `Attribute`s belonging to this `FunctionValue` if index is 0,
    /// otherwise to its parameter in the 1 - Nth position.
    ///
    /// # Example
    ///
    /// ```no_run
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
    /// fn_value.add_attribute(0, string_attribute);
    /// fn_value.add_attribute(0, enum_attribute);
    ///
    /// assert_eq!(fn_value.count_attributes(0), 2);
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn count_attributes(&self, index: u32) -> u32 {
        unsafe {
            LLVMGetAttributeCountAtIndex(self.as_value_ref(), index)
        }
    }

    /// Removes a string `Attribute` belonging this `FunctionValue` if index is 0,
    /// otherwise to its parameter in the 1 - Nth position.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    ///
    /// fn_value.add_attribute(0, string_attribute);
    /// fn_value.remove_string_attribute(0, "my_key");
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn remove_string_attribute(&self, index: u32, key: &str) {
        unsafe {
            LLVMRemoveStringAttributeAtIndex(self.as_value_ref(), index, key.as_ptr() as *const i8, key.len() as u32)
        }
    }

    /// Removes an enum `Attribute` belonging to this `FunctionValue` if index is 0,
    /// otherwise to its parameter in the 1 - Nth position.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    ///
    /// fn_value.add_attribute(0, enum_attribute);
    /// fn_value.remove_enum_attribute(0, 1);
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn remove_enum_attribute(&self, index: u32, kind_id: u32) {
        unsafe {
            LLVMRemoveEnumAttributeAtIndex(self.as_value_ref(), index, kind_id)
        }
    }

    /// Gets an enum `Attribute` belonging to this `FunctionValue` if index is 0,
    /// otherwise to its parameter in the 1 - Nth position.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    ///
    /// fn_value.add_attribute(0, enum_attribute);
    ///
    /// assert_eq!(fn_value.get_enum_attribute(0, 1), Some(enum_attribute));
    /// ```
    // SubTypes: -> Option<Attribute<Enum>>
    #[llvm_versions(3.9 => latest)]
    pub fn get_enum_attribute(&self, index: u32, kind_id: u32) -> Option<Attribute> {
        let ptr = unsafe {
            LLVMGetEnumAttributeAtIndex(self.as_value_ref(), index, kind_id)
        };

        if ptr.is_null() {
            return None;
        }

        Some(Attribute::new(ptr))
    }

    /// Gets a string `Attribute` belonging this `FunctionValue` if index is 0,
    /// otherwise to its parameter in the 1 - Nth position.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    ///
    /// fn_value.add_attribute(0, string_attribute);
    ///
    /// assert_eq!(fn_value.get_string_attribute(0, "my_key"), Some(string_attribute));
    /// ```
    // SubTypes: -> Option<Attribute<String>>
    #[llvm_versions(3.9 => latest)]
    pub fn get_string_attribute(&self, index: u32, key: &str) -> Option<Attribute> {
        let ptr = unsafe {
            LLVMGetStringAttributeAtIndex(self.as_value_ref(), index, key.as_ptr() as *const i8, key.len() as u32)
        };

        if ptr.is_null() {
            return None;
        }

        Some(Attribute::new(ptr))
    }

    pub fn set_param_alignment(&self, param_index: u32, alignment: u32) {
        if let Some(param) = self.get_nth_param(param_index) {
            unsafe {
                LLVMSetParamAlignment(param.as_value_ref(), alignment)
            }
        }
    }

    /// Gets the `GlobalValue` version of this `FunctionValue`. This allows
    /// you to further inspect its global properties or even convert it to
    /// a `PointerValue`.
    pub fn as_global_value(&self) -> GlobalValue {
        GlobalValue::new(self.as_value_ref())
    }
}

impl AsValueRef for FunctionValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.fn_value.value
    }
}

impl fmt::Debug for FunctionValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = self.print_to_string();
        let llvm_type = self.get_type();
        let name = self.get_name();
        let is_const = unsafe {
            LLVMIsConstant(self.fn_value.value) == 1
        };
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

#[derive(Debug)]
pub struct ParamValueIter {
    param_iter_value: LLVMValueRef,
    start: bool,
}

impl Iterator for ParamValueIter {
    type Item = BasicValueEnum;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start {
            let first_value = unsafe {
                LLVMGetFirstParam(self.param_iter_value)
            };

            if first_value.is_null() {
                return None;
            }

            self.start = false;

            self.param_iter_value = first_value;

            return Some(Self::Item::new(first_value));
        }

        let next_value = unsafe {
            LLVMGetNextParam(self.param_iter_value)
        };

        if next_value.is_null() {
            return None;
        }

        self.param_iter_value = next_value;

        Some(Self::Item::new(next_value))
    }
}
