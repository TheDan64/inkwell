use std::fmt::{self, Display};

use either::Either;
use llvm_sys::core::{
    LLVMGetInstructionCallConv, LLVMGetTypeKind, LLVMIsTailCall, LLVMSetInstrParamAlignment,
    LLVMSetInstructionCallConv, LLVMSetTailCall, LLVMTypeOf,
};
#[llvm_versions(18..)]
use llvm_sys::core::{LLVMGetTailCallKind, LLVMSetTailCallKind};
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMTypeKind;

use crate::attributes::{Attribute, AttributeLoc};
use crate::values::{AsValueRef, BasicValueEnum, FunctionValue, InstructionValue, Value};

use super::{AnyValue, InstructionOpcode};

/// A value resulting from a function call. It may have function attributes applied to it.
///
/// This struct may be removed in the future in favor of an `InstructionValue<CallSite>` type.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct CallSiteValue<'ctx>(Value<'ctx>);

impl<'ctx> CallSiteValue<'ctx> {
    /// Get a value from an [LLVMValueRef].
    ///
    /// # Safety
    ///
    /// The ref must be valid and of type call site.
    pub unsafe fn new(value: LLVMValueRef) -> Self {
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.set_tail_call(true);
    /// ```
    pub fn set_tail_call(self, tail_call: bool) {
        unsafe { LLVMSetTailCall(self.as_value_ref(), tail_call as i32) }
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.set_tail_call(true);
    ///
    /// assert!(call_site_value.is_tail_call());
    /// ```
    pub fn is_tail_call(self) -> bool {
        unsafe { LLVMIsTailCall(self.as_value_ref()) == 1 }
    }

    /// Returns tail, musttail, and notail attributes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::values::LLVMTailCallKind::*;
    ///
    /// let context = inkwell::context::Context::create();
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// assert_eq!(call_site.get_tail_call_kind(), LLVMTailCallKindNone);
    /// ```
    #[llvm_versions(18..)]
    pub fn get_tail_call_kind(self) -> super::LLVMTailCallKind {
        unsafe { LLVMGetTailCallKind(self.as_value_ref()) }
    }

    /// Sets tail, musttail, and notail attributes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::values::LLVMTailCallKind::*;
    ///
    /// let context = inkwell::context::Context::create();
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site.set_tail_call_kind(LLVMTailCallKindTail);
    /// assert_eq!(call_site.get_tail_call_kind(), LLVMTailCallKindTail);
    /// ```
    #[llvm_versions(18..)]
    pub fn set_tail_call_kind(self, kind: super::LLVMTailCallKind) {
        unsafe { LLVMSetTailCallKind(self.as_value_ref(), kind) };
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// assert!(call_site_value.try_as_basic_value().is_right());
    /// ```
    pub fn try_as_basic_value(self) -> Either<BasicValueEnum<'ctx>, InstructionValue<'ctx>> {
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
    /// use inkwell::attributes::AttributeLoc;
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// call_site_value.add_attribute(AttributeLoc::Return, enum_attribute);
    /// ```
    pub fn add_attribute(self, loc: AttributeLoc, attribute: Attribute) {
        use llvm_sys::core::LLVMAddCallSiteAttribute;

        unsafe { LLVMAddCallSiteAttribute(self.as_value_ref(), loc.get_index(), attribute.attribute) }
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// assert_eq!(call_site_value.get_called_fn_value(), fn_value);
    /// ```
    pub fn get_called_fn_value(self) -> FunctionValue<'ctx> {
        use llvm_sys::core::LLVMGetCalledValue;

        unsafe { FunctionValue::new(LLVMGetCalledValue(self.as_value_ref())).expect("This should never be null?") }
    }

    /// Counts the number of `Attribute`s on this `CallSiteValue` at an index.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// call_site_value.add_attribute(AttributeLoc::Return, enum_attribute);
    ///
    /// assert_eq!(call_site_value.count_attributes(AttributeLoc::Return), 2);
    /// ```
    pub fn count_attributes(self, loc: AttributeLoc) -> u32 {
        use llvm_sys::core::LLVMGetCallSiteAttributeCount;

        unsafe { LLVMGetCallSiteAttributeCount(self.as_value_ref(), loc.get_index()) }
    }

    /// Get all `Attribute`s on this `CallSiteValue` at an index.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// call_site_value.add_attribute(AttributeLoc::Return, enum_attribute);
    ///
    /// assert_eq!(call_site_value.attributes(AttributeLoc::Return), vec![ string_attribute, enum_attribute ]);
    /// ```
    pub fn attributes(self, loc: AttributeLoc) -> Vec<Attribute> {
        use llvm_sys::core::LLVMGetCallSiteAttributes;
        use std::mem::{ManuallyDrop, MaybeUninit};

        let count = self.count_attributes(loc) as usize;

        // initialize a vector, but leave each element uninitialized
        let mut attribute_refs: Vec<MaybeUninit<Attribute>> = vec![MaybeUninit::uninit(); count];

        // Safety: relies on `Attribute` having the same in-memory representation as `LLVMAttributeRef`
        unsafe {
            LLVMGetCallSiteAttributes(
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

    /// Gets an enum `Attribute` on this `CallSiteValue` at an index and kind id.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// call_site_value.add_attribute(AttributeLoc::Return, enum_attribute);
    ///
    /// assert_eq!(call_site_value.get_enum_attribute(AttributeLoc::Return, 1).unwrap(), enum_attribute);
    /// ```
    // SubTypes: -> Attribute<Enum>
    pub fn get_enum_attribute(self, loc: AttributeLoc, kind_id: u32) -> Option<Attribute> {
        use llvm_sys::core::LLVMGetCallSiteEnumAttribute;

        let ptr = unsafe { LLVMGetCallSiteEnumAttribute(self.as_value_ref(), loc.get_index(), kind_id) };

        if ptr.is_null() {
            return None;
        }

        unsafe { Some(Attribute::new(ptr)) }
    }

    /// Gets a string `Attribute` on this `CallSiteValue` at an index and key.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// call_site_value.add_attribute(AttributeLoc::Return, enum_attribute);
    ///
    /// assert_eq!(call_site_value.get_string_attribute(AttributeLoc::Return, "my_key").unwrap(), string_attribute);
    /// ```
    // SubTypes: -> Attribute<String>
    pub fn get_string_attribute(self, loc: AttributeLoc, key: &str) -> Option<Attribute> {
        use llvm_sys::core::LLVMGetCallSiteStringAttribute;

        let ptr = unsafe {
            LLVMGetCallSiteStringAttribute(
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

    /// Removes an enum `Attribute` on this `CallSiteValue` at an index and kind id.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// call_site_value.add_attribute(AttributeLoc::Return, enum_attribute);
    /// call_site_value.remove_enum_attribute(AttributeLoc::Return, 1);
    ///
    /// assert_eq!(call_site_value.get_enum_attribute(AttributeLoc::Return, 1), None);
    /// ```
    pub fn remove_enum_attribute(self, loc: AttributeLoc, kind_id: u32) {
        use llvm_sys::core::LLVMRemoveCallSiteEnumAttribute;

        unsafe { LLVMRemoveCallSiteEnumAttribute(self.as_value_ref(), loc.get_index(), kind_id) }
    }

    /// Removes a string `Attribute` on this `CallSiteValue` at an index and key.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.add_attribute(AttributeLoc::Return, string_attribute);
    /// call_site_value.add_attribute(AttributeLoc::Return, enum_attribute);
    /// call_site_value.remove_string_attribute(AttributeLoc::Return, "my_key");
    ///
    /// assert_eq!(call_site_value.get_string_attribute(AttributeLoc::Return, "my_key"), None);
    /// ```
    pub fn remove_string_attribute(self, loc: AttributeLoc, key: &str) {
        use llvm_sys::core::LLVMRemoveCallSiteStringAttribute;

        unsafe {
            LLVMRemoveCallSiteStringAttribute(
                self.as_value_ref(),
                loc.get_index(),
                key.as_ptr() as *const ::libc::c_char,
                key.len() as u32,
            )
        }
    }

    /// Counts the number of arguments this `CallSiteValue` was called with.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// assert_eq!(call_site_value.count_arguments(), 0);
    /// ```
    pub fn count_arguments(self) -> u32 {
        use llvm_sys::core::LLVMGetNumArgOperands;

        unsafe { LLVMGetNumArgOperands(self.as_value_ref()) }
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// assert_eq!(call_site_value.get_call_convention(), 0);
    /// ```
    pub fn get_call_convention(self) -> u32 {
        unsafe { LLVMGetInstructionCallConv(self.as_value_ref()) }
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
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.set_call_convention(2);
    ///
    /// assert_eq!(call_site_value.get_call_convention(), 2);
    /// ```
    pub fn set_call_convention(self, conv: u32) {
        unsafe { LLVMSetInstructionCallConv(self.as_value_ref(), conv) }
    }

    /// Shortcut for setting the alignment `Attribute` for this `CallSiteValue`.
    ///
    /// # Panics
    ///
    /// When the alignment is not a power of 2.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::attributes::AttributeLoc;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_value = module.add_function("my_fn", fn_type, None);
    /// let entry_bb = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_bb);
    ///
    /// let call_site_value = builder.build_call(fn_value, &[], "my_fn").unwrap();
    ///
    /// call_site_value.set_alignment_attribute(AttributeLoc::Param(0), 2);
    /// ```
    pub fn set_alignment_attribute(self, loc: AttributeLoc, alignment: u32) {
        assert_eq!(alignment.count_ones(), 1, "Alignment must be a power of two.");

        unsafe { LLVMSetInstrParamAlignment(self.as_value_ref(), loc.get_index(), alignment) }
    }
}

unsafe impl AsValueRef for CallSiteValue<'_> {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.0.value
    }
}

impl Display for CallSiteValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.print_to_string())
    }
}

impl<'ctx> TryFrom<InstructionValue<'ctx>> for CallSiteValue<'ctx> {
    type Error = ();

    fn try_from(value: InstructionValue<'ctx>) -> Result<Self, Self::Error> {
        if value.get_opcode() == InstructionOpcode::Call {
            unsafe { Ok(CallSiteValue::new(value.as_value_ref())) }
        } else {
            Err(())
        }
    }
}
