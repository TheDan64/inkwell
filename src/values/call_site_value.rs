use either::Either;
use llvm_sys::LLVMTypeKind;
use llvm_sys::core::{LLVMIsTailCall, LLVMSetTailCall, LLVMGetTypeKind, LLVMTypeOf, LLVMSetInstructionCallConv, LLVMGetInstructionCallConv, LLVMSetInstrParamAlignment};
use llvm_sys::prelude::LLVMValueRef;

#[llvm_versions(3.9 => latest)]
use attributes::Attribute;
use support::LLVMString;
use values::{AsValueRef, BasicValueEnum, InstructionValue, Value};
#[llvm_versions(3.9 => latest)]
use values::FunctionValue;

/// A value resulting from a function call. It may have function attributes applied to it.
///
/// This struct may be removed in the future in favor of an `InstructionValue<CallSite>` type.
#[derive(Debug, PartialEq)]
pub struct CallSiteValue(Value);

impl CallSiteValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        CallSiteValue(Value::new(value))
    }

    /// Sets whether or not this call is a tail call.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// call_site_value.set_tail_call(true);
    /// ```
    pub fn set_tail_call(&self, tail_call: bool) {
        unsafe {
            LLVMSetTailCall(self.as_value_ref(), tail_call as i32)
        }
    }

    /// Determines whether or not this call is a tail call.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// call_site_value.set_tail_call(true);
    ///
    /// assert!(call_site_value.is_tail_call());
    /// ```
    pub fn is_tail_call(&self) -> bool {
        unsafe {
            LLVMIsTailCall(self.as_value_ref()) == 1
        }
    }

    /// Try to convert this `CallSiteValue` to a `BasicValueEnum` if not a void return type.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// assert!(call_site_value.try_as_basic_value().is_right());
    /// ```
    pub fn try_as_basic_value(&self) -> Either<BasicValueEnum, InstructionValue> {
        unsafe {
            match LLVMGetTypeKind(LLVMTypeOf(self.as_value_ref())) {
                LLVMTypeKind::LLVMVoidTypeKind => Either::Right(InstructionValue::new(self.as_value_ref())),
                _ => Either::Left(BasicValueEnum::new(self.as_value_ref())),
            }
        }
    }

    /// Adds an `Attribute` to this `CallSiteValue`.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// call_site_value.add_attribute(0, string_attribute);
    /// call_site_value.add_attribute(0, enum_attribute);
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn add_attribute(&self, index: u32, attribute: Attribute) {
        use llvm_sys::core::LLVMAddCallSiteAttribute;

        unsafe {
            LLVMAddCallSiteAttribute(self.as_value_ref(), index, attribute.attribute)
        }
    }

    /// Gets the `FunctionValue` this `CallSiteValue` is based on.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// assert_eq!(call_site_value.get_called_fn_value(), fn_value);
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn get_called_fn_value(&self) -> FunctionValue {
        use llvm_sys::core::LLVMGetCalledValue;

        let ptr = unsafe {
            LLVMGetCalledValue(self.as_value_ref())
        };

        FunctionValue::new(ptr).expect("This should never be null?")
    }

    /// Counts the number of `Attribute`s on this `CallSiteValue` at an index.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// call_site_value.add_attribute(0, string_attribute);
    /// call_site_value.add_attribute(0, enum_attribute);
    ///
    /// assert_eq!(call_site_value.count_attributes(0), 2);
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn count_attributes(&self, index: u32) -> u32 {
        use llvm_sys::core::LLVMGetCallSiteAttributeCount;

        unsafe {
            LLVMGetCallSiteAttributeCount(self.as_value_ref(), index)
        }
    }

    /// Gets an enum `Attribute` on this `CallSiteValue` at an index and kind id.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// call_site_value.add_attribute(0, string_attribute);
    /// call_site_value.add_attribute(0, enum_attribute);
    ///
    /// assert_eq!(call_site_value.get_enum_attribute(0, 1).unwrap(), enum_attribute);
    /// ```
    // SubTypes: -> Attribute<Enum>
    #[llvm_versions(3.9 => latest)]
    pub fn get_enum_attribute(&self, index: u32, kind_id: u32) -> Option<Attribute> {
        use llvm_sys::core::LLVMGetCallSiteEnumAttribute;

        let ptr = unsafe {
            LLVMGetCallSiteEnumAttribute(self.as_value_ref(), index, kind_id)
        };

        if ptr.is_null() {
            return None;
        }

        Some(Attribute::new(ptr))
    }

    /// Gets a string `Attribute` on this `CallSiteValue` at an index and key.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// call_site_value.add_attribute(0, string_attribute);
    /// call_site_value.add_attribute(0, enum_attribute);
    ///
    /// assert_eq!(call_site_value.get_string_attribute(0, "my_key").unwrap(), string_attribute);
    /// ```
    // SubTypes: -> Attribute<String>
    #[llvm_versions(3.9 => latest)]
    pub fn get_string_attribute(&self, index: u32, key: &str) -> Option<Attribute> {
        use llvm_sys::core::LLVMGetCallSiteStringAttribute;

        let ptr = unsafe {
            LLVMGetCallSiteStringAttribute(self.as_value_ref(), index, key.as_ptr() as *const i8, key.len() as u32)
        };

        if ptr.is_null() {
            return None;
        }

        Some(Attribute::new(ptr))
    }

    /// Removes an enum `Attribute` on this `CallSiteValue` at an index and kind id.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// call_site_value.add_attribute(0, string_attribute);
    /// call_site_value.add_attribute(0, enum_attribute);
    /// call_site_value.remove_enum_attribute(0, 1);
    ///
    /// assert_eq!(call_site_value.get_enum_attribute(0, 1), None);
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn remove_enum_attribute(&self, index: u32, kind_id: u32) {
        use llvm_sys::core::LLVMRemoveCallSiteEnumAttribute;

        unsafe {
            LLVMRemoveCallSiteEnumAttribute(self.as_value_ref(), index, kind_id)
        }
    }

    /// Removes a string `Attribute` on this `CallSiteValue` at an index and key.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// call_site_value.add_attribute(0, string_attribute);
    /// call_site_value.add_attribute(0, enum_attribute);
    /// call_site_value.remove_string_attribute(0, "my_key");
    ///
    /// assert_eq!(call_site_value.get_string_attribute(0, "my_key"), None);
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn remove_string_attribute(&self, index: u32, key: &str) {
        use llvm_sys::core::LLVMRemoveCallSiteStringAttribute;

        unsafe {
            LLVMRemoveCallSiteStringAttribute(self.as_value_ref(), index, key.as_ptr() as *const i8, key.len() as u32)
        }
    }

    /// Counts the number of arguments this `CallSiteValue` was called with.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let string_attribute = context.create_string_attribute("my_key", "my_val");
    /// let enum_attribute = context.create_enum_attribute(1, 1);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// assert_eq!(call_site_value.count_arguments(), 0);
    /// ```
    #[llvm_versions(3.9 => latest)]
    pub fn count_arguments(&self) -> u32 {
        use llvm_sys::core::LLVMGetNumArgOperands;

        unsafe {
            LLVMGetNumArgOperands(self.as_value_ref())
        }
    }

    /// Gets the calling convention for this `CallSiteValue`.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// assert_eq!(call_site_value.get_call_convention(), 0);
    /// ```
    pub fn get_call_convention(&self) -> u32 {
        unsafe {
            LLVMGetInstructionCallConv(self.as_value_ref())
        }
    }

    /// Sets the calling convention for this `CallSiteValue`.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// call_site_value.set_call_convention(2);
    ///
    /// assert_eq!(call_site_value.get_call_convention(), 2);
    /// ```
    pub fn set_call_convention(&self, conv: u32) {
        unsafe {
            LLVMSetInstructionCallConv(self.as_value_ref(), conv)
        }
    }

    /// Shortcut for setting the alignment `Attribute` for this `CallSiteValue`.
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
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let entry_bb = fn_value.append_basic_block("entry");
    ///
    /// builder.position_at_end(&entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn");
    ///
    /// call_site_value.set_param_alignment_attribute(0, 2);
    /// ```
    pub fn set_param_alignment_attribute(&self, index: u32, alignment: u32) {
        unsafe {
            LLVMSetInstrParamAlignment(self.as_value_ref(), index, alignment)
        }
    }

    /// Prints the definition of a `CallSiteValue` to a `LLVMString`.
    pub fn print_to_string(&self) -> LLVMString {
        self.0.print_to_string()
    }
}

impl AsValueRef for CallSiteValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.0.value
    }
}
