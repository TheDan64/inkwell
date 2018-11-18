//! A `Context` is an opaque owner and manager of core global data.

use llvm_sys::core::{LLVMAppendBasicBlockInContext, LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDoubleTypeInContext, LLVMFloatTypeInContext, LLVMFP128TypeInContext, LLVMInsertBasicBlockInContext, LLVMInt16TypeInContext, LLVMInt1TypeInContext, LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMStructCreateNamed, LLVMStructTypeInContext, LLVMVoidTypeInContext, LLVMHalfTypeInContext, LLVMGetGlobalContext, LLVMPPCFP128TypeInContext, LLVMConstStructInContext, LLVMMDNodeInContext, LLVMMDStringInContext, LLVMGetMDKindIDInContext, LLVMX86FP80TypeInContext, LLVMConstStringInContext, LLVMContextSetDiagnosticHandler};
#[llvm_versions(4.0 => latest)]
use llvm_sys::core::{LLVMCreateEnumAttribute, LLVMCreateStringAttribute};
use llvm_sys::prelude::{LLVMContextRef, LLVMTypeRef, LLVMValueRef, LLVMDiagnosticInfoRef};
use llvm_sys::ir_reader::LLVMParseIRInContext;
use libc::c_void;

#[llvm_versions(4.0 => latest)]
use attributes::Attribute;
use basic_block::BasicBlock;
use builder::Builder;
use memory_buffer::MemoryBuffer;
use module::Module;
use support::LLVMString;
use types::{BasicTypeEnum, FloatType, IntType, StructType, VoidType, AsTypeRef};
use values::{AsValueRef, FunctionValue, StructValue, MetadataValue, BasicValueEnum, VectorValue};

use std::ffi::CString;
use std::mem::forget;
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

/// A `Context` is a container for all LLVM entities including `Module`s.
///
/// A `Context` is not thread safe and cannot be shared across threads. Multiple `Context`s
/// can, however, execute on different threads simultaneously according to the LLVM docs.
///
/// # Note
///
/// Cloning this object is essentially just a case of copying a couple pointers
/// and incrementing one or two atomics, so this should be quite cheap to create
/// copies. The underlying LLVM object will be automatically deallocated when
/// there are no more references to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    pub(crate) context: Rc<LLVMContextRef>,
}

impl Context {
    pub(crate) fn new(context: Rc<LLVMContextRef>) -> Self {
        assert!(!context.is_null());

        Context {
            context,
        }
    }

    /// Creates a new `Context`.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// ```
    pub fn create() -> Self {
        let context = unsafe {
            LLVMContextCreate()
        };

        Context::new(Rc::new(context))
    }

    /// Creates a `ContextRef` which references the global context singleton.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    ///
    /// let context = Context::get_global();
    /// ```
    pub fn get_global() -> ContextRef {
        let context = unsafe {
            LLVMGetGlobalContext()
        };

        ContextRef::new(Context::new(Rc::new(context)))
    }

    /// Creates a new `Builder` for a `Context`.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// ```
    pub fn create_builder(&self) -> Builder {
        let builder = unsafe {
            LLVMCreateBuilderInContext(*self.context)
        };

        Builder::new(builder)
    }

    /// Creates a new `Module` for a `Context`.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// ```
    pub fn create_module(&self, name: &str) -> Module {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let module = unsafe {
            LLVMModuleCreateWithNameInContext(c_string.as_ptr(), *self.context)
        };

        Module::new(module, Some(&self))
    }

    /// Creates a new `Module` for the current `Context` from a `MemoryBuffer`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_val = module.add_function("my_fn", fn_type, None);
    /// let basic_block = fn_val.append_basic_block("entry");
    ///
    /// builder.position_at_end(&basic_block);
    /// builder.build_return(None);
    ///
    /// let memory_buffer = module.write_bitcode_to_memory();
    ///
    /// let module2 = context.create_module_from_ir(memory_buffer).unwrap();
    /// ```
    // REVIEW: I haven't yet been able to find docs or other wrappers that confirm, but my suspicion
    // is that the method needs to take ownership of the MemoryBuffer... otherwise I see what looks like
    // a double free in valgrind when the MemoryBuffer drops so we are `forget`ting MemoryBuffer here
    // for now until we can confirm this is the correct thing to do
    pub fn create_module_from_ir(&self, memory_buffer: MemoryBuffer) -> Result<Module, LLVMString> {
        let mut module = ptr::null_mut();
        let mut err_str = ptr::null_mut();

        let code = unsafe {
            LLVMParseIRInContext(*self.context, memory_buffer.memory_buffer, &mut module, &mut err_str)
        };

        forget(memory_buffer);

        if code == 0 {
            return Ok(Module::new(module, Some(&self)));
        }

        Err(LLVMString::new(err_str))
    }

    /// Gets the `VoidType`. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let void_type = context.void_type();
    ///
    /// assert_eq!(*void_type.get_context(), context);
    /// ```
    pub fn void_type(&self) -> VoidType {
        let void_type = unsafe {
            LLVMVoidTypeInContext(*self.context)
        };

        VoidType::new(void_type)
    }

    /// Gets the `IntType` representing 1 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let bool_type = context.bool_type();
    ///
    /// assert_eq!(bool_type.get_bit_width(), 1);
    /// assert_eq!(*bool_type.get_context(), context);
    /// ```
    pub fn bool_type(&self) -> IntType {
        let bool_type = unsafe {
            LLVMInt1TypeInContext(*self.context)
        };

        IntType::new(bool_type)
    }

    /// Gets the `IntType` representing 8 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    ///
    /// assert_eq!(i8_type.get_bit_width(), 8);
    /// assert_eq!(*i8_type.get_context(), context);
    /// ```
    pub fn i8_type(&self) -> IntType {
        let i8_type = unsafe {
            LLVMInt8TypeInContext(*self.context)
        };

        IntType::new(i8_type)
    }

    /// Gets the `IntType` representing 16 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i16_type = context.i16_type();
    ///
    /// assert_eq!(i16_type.get_bit_width(), 16);
    /// assert_eq!(*i16_type.get_context(), context);
    /// ```
    pub fn i16_type(&self) -> IntType {
        let i16_type = unsafe {
            LLVMInt16TypeInContext(*self.context)
        };

        IntType::new(i16_type)
    }

    /// Gets the `IntType` representing 32 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i32_type = context.i32_type();
    ///
    /// assert_eq!(i32_type.get_bit_width(), 32);
    /// assert_eq!(*i32_type.get_context(), context);
    /// ```
    pub fn i32_type(&self) -> IntType {
        let i32_type = unsafe {
            LLVMInt32TypeInContext(*self.context)
        };

        IntType::new(i32_type)
    }

    /// Gets the `IntType` representing 64 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i64_type = context.i64_type();
    ///
    /// assert_eq!(i64_type.get_bit_width(), 64);
    /// assert_eq!(*i64_type.get_context(), context);
    /// ```
    pub fn i64_type(&self) -> IntType {
        let i64_type = unsafe {
            LLVMInt64TypeInContext(*self.context)
        };

        IntType::new(i64_type)
    }

    /// Gets the `IntType` representing 128 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i128_type = context.i128_type();
    ///
    /// assert_eq!(i128_type.get_bit_width(), 128);
    /// assert_eq!(*i128_type.get_context(), context);
    /// ```
    pub fn i128_type(&self) -> IntType {
        // REVIEW: The docs says there's a LLVMInt128TypeInContext, but
        // it might only be in a newer version

        self.custom_width_int_type(128)
    }

    /// Gets the `IntType` representing a custom bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i42_type = context.custom_width_int_type(42);
    ///
    /// assert_eq!(i42_type.get_bit_width(), 42);
    /// assert_eq!(*i42_type.get_context(), context);
    /// ```
    pub fn custom_width_int_type(&self, bits: u32) -> IntType {
        let int_type = unsafe {
            LLVMIntTypeInContext(*self.context, bits)
        };

        IntType::new(int_type)
    }

    /// Gets the `FloatType` representing a 16 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    ///
    /// let f16_type = context.f16_type();
    ///
    /// assert_eq!(*f16_type.get_context(), context);
    /// ```
    pub fn f16_type(&self) -> FloatType {
        let f16_type = unsafe {
            LLVMHalfTypeInContext(*self.context)
        };

        FloatType::new(f16_type)
    }

    /// Gets the `FloatType` representing a 32 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    ///
    /// let f32_type = context.f32_type();
    ///
    /// assert_eq!(*f32_type.get_context(), context);
    /// ```
    pub fn f32_type(&self) -> FloatType {
        let f32_type = unsafe {
            LLVMFloatTypeInContext(*self.context)
        };

        FloatType::new(f32_type)
    }

    /// Gets the `FloatType` representing a 64 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    ///
    /// let f64_type = context.f64_type();
    ///
    /// assert_eq!(*f64_type.get_context(), context);
    /// ```
    pub fn f64_type(&self) -> FloatType {
        let f64_type = unsafe {
            LLVMDoubleTypeInContext(*self.context)
        };

        FloatType::new(f64_type)
    }

    /// Gets the `FloatType` representing a 80 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    ///
    /// let x86_f80_type = context.x86_f80_type();
    ///
    /// assert_eq!(*x86_f80_type.get_context(), context);
    /// ```
    pub fn x86_f80_type(&self) -> FloatType {
        let f128_type = unsafe {
            LLVMX86FP80TypeInContext(*self.context)
        };

        FloatType::new(f128_type)
    }

    /// Gets the `FloatType` representing a 128 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    ///
    /// let f128_type = context.f128_type();
    ///
    /// assert_eq!(*f128_type.get_context(), context);
    /// ```
    pub fn f128_type(&self) -> FloatType {
        let f128_type = unsafe {
            LLVMFP128TypeInContext(*self.context)
        };

        FloatType::new(f128_type)
    }

    /// Gets the `FloatType` representing a 128 bit width. It will be assigned the current context.
    ///
    /// PPC is two 64 bits side by side rather than one single 128 bit float.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    ///
    /// let f128_type = context.ppc_f128_type();
    ///
    /// assert_eq!(*f128_type.get_context(), context);
    /// ```
    pub fn ppc_f128_type(&self) -> FloatType {
        let f128_type = unsafe {
            LLVMPPCFP128TypeInContext(*self.context)
        };

        FloatType::new(f128_type)
    }

    /// Creates a `StructType` definiton from heterogeneous types in the current `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let i16_type = context.i16_type();
    /// let struct_type = context.struct_type(&[i16_type.into(), f32_type.into()], false);
    ///
    /// assert_eq!(struct_type.get_field_types(), &[i16_type.into(), f32_type.into()]);
    /// ```
    // REVIEW: AnyType but VoidType? FunctionType?
    pub fn struct_type(&self, field_types: &[BasicTypeEnum], packed: bool) -> StructType {
        let mut field_types: Vec<LLVMTypeRef> = field_types.iter()
                                                           .map(|val| val.as_type_ref())
                                                           .collect();
        let struct_type = unsafe {
            LLVMStructTypeInContext(*self.context, field_types.as_mut_ptr(), field_types.len() as u32, packed as i32)
        };

        StructType::new(struct_type)
    }

    /// Creates an opaque `StructType` with no type definition yet defined.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let i16_type = context.i16_type();
    /// let struct_type = context.opaque_struct_type("my_struct");
    ///
    /// assert_eq!(struct_type.get_field_types(), &[]);
    /// ```
    pub fn opaque_struct_type(&self, name: &str) -> StructType {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let struct_type = unsafe {
            LLVMStructCreateNamed(*self.context, c_string.as_ptr())
        };

        StructType::new(struct_type)
    }

    /// Creates a constant `StructValue` from constant values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let f32_type = context.f32_type();
    /// let i16_type = context.i16_type();
    /// let f32_one = f32_type.const_float(1.);
    /// let i16_two = i16_type.const_int(2, false);
    /// let const_struct = context.const_struct(&[i16_two.into(), f32_one.into()], false);
    ///
    /// assert_eq!(const_struct.get_type().get_field_types(), &[i16_type.into(), f32_type.into()]);
    /// ```
    pub fn const_struct(&self, values: &[BasicValueEnum], packed: bool) -> StructValue {
        let mut args: Vec<LLVMValueRef> = values.iter()
                                                .map(|val| val.as_value_ref())
                                                .collect();
        let value = unsafe {
            LLVMConstStructInContext(*self.context, args.as_mut_ptr(), args.len() as u32, packed as i32)
        };

        StructValue::new(value)
    }

    /// Append a named `BasicBlock` at the end of the referenced `FunctionValue`.
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
    /// let entry_basic_block = context.append_basic_block(&fn_value, "entry");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 1);
    ///
    /// let last_basic_block = context.append_basic_block(&fn_value, "last");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 2);
    /// assert_eq!(fn_value.get_first_basic_block().unwrap(), entry_basic_block);
    /// assert_eq!(fn_value.get_last_basic_block().unwrap(), last_basic_block);
    /// ```
    pub fn append_basic_block(&self, function: &FunctionValue, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMAppendBasicBlockInContext(*self.context, function.as_value_ref(), c_string.as_ptr())
        };

        BasicBlock::new(bb).expect("Appending basic block should never fail")
    }

    /// Append a named `BasicBlock` after the referenced `BasicBlock`.
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
    /// let entry_basic_block = context.append_basic_block(&fn_value, "entry");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 1);
    ///
    /// let last_basic_block = context.insert_basic_block_after(&entry_basic_block, "last");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 2);
    /// assert_eq!(fn_value.get_first_basic_block().unwrap(), entry_basic_block);
    /// assert_eq!(fn_value.get_last_basic_block().unwrap(), last_basic_block);
    /// ```
    // REVIEW: What happens when using these methods and the BasicBlock doesn't have a parent?
    // Should they be callable at all? Needs testing to see what LLVM will do, I suppose. See below unwrap.
    // Maybe need SubTypes: BasicBlock<HasParent>, BasicBlock<Orphan>?
    pub fn insert_basic_block_after(&self, basic_block: &BasicBlock, name: &str) -> BasicBlock {
        match basic_block.get_next_basic_block() {
            Some(next_basic_block) => self.prepend_basic_block(&next_basic_block, name),
            None => {
                let parent_fn = basic_block.get_parent().unwrap();

                self.append_basic_block(&parent_fn, name)
            },
        }
    }

    /// Prepend a named `BasicBlock` before the referenced `BasicBlock`.
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
    /// let entry_basic_block = context.append_basic_block(&fn_value, "entry");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 1);
    ///
    /// let first_basic_block = context.prepend_basic_block(&entry_basic_block, "first");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 2);
    /// assert_eq!(fn_value.get_first_basic_block().unwrap(), first_basic_block);
    /// assert_eq!(fn_value.get_last_basic_block().unwrap(), entry_basic_block);
    /// ```
    pub fn prepend_basic_block(&self, basic_block: &BasicBlock, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMInsertBasicBlockInContext(*self.context, basic_block.basic_block, c_string.as_ptr())
        };

        BasicBlock::new(bb).expect("Prepending basic block should never fail")
    }

    /// Creates a `MetadataValue` tuple of heterogeneous types (a "Node") for the current context. It can be assigned to a value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let i8_type = context.i8_type();
    /// let i8_two = i8_type.const_int(2, false);
    /// let f32_type = context.f32_type();
    /// let f32_zero = f32_type.const_float(0.);
    /// let md_node = context.metadata_node(&[i8_two.into(), f32_zero.into()]);
    /// let f32_one = f32_type.const_float(1.);
    ///
    /// assert!(md_node.is_node());
    ///
    /// f32_one.set_metadata(md_node, 0);
    /// ```
    // REVIEW: Maybe more helpful to beginners to call this metadata_tuple?
    // REVIEW: Seems to be unassgned to anything
    // REVIEW: Should maybe make this take &[BasicValueEnum]?
    pub fn metadata_node(&self, values: &[BasicValueEnum]) -> MetadataValue {
        let mut tuple_values: Vec<LLVMValueRef> = values.iter()
                                                        .map(|val| val.as_value_ref())
                                                        .collect();
        let metadata_value = unsafe {
            LLVMMDNodeInContext(*self.context, tuple_values.as_mut_ptr(), tuple_values.len() as u32)
        };

        MetadataValue::new(metadata_value)
    }

    /// Creates a `MetadataValue` string for the current context. It can be assigned to a value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let md_string = context.metadata_string("Floats are awesome!");
    /// let f32_type = context.f32_type();
    /// let f32_one = f32_type.const_float(1.);
    ///
    /// assert!(md_string.is_string());
    ///
    /// f32_one.set_metadata(md_string, 0);
    /// ```
    // REVIEW: Seems to be unassgned to anything
    pub fn metadata_string(&self, string: &str) -> MetadataValue {
        let c_string = CString::new(string).expect("Conversion to CString failed unexpectedly");

        let metadata_value = unsafe {
            LLVMMDStringInContext(*self.context, c_string.as_ptr(), string.len() as u32)
        };

        MetadataValue::new(metadata_value)
    }

    /// Obtains the index of a metadata kind id. If the string doesn't exist, LLVM will add it at index `FIRST_CUSTOM_METADATA_KIND_ID` onward.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::values::FIRST_CUSTOM_METADATA_KIND_ID;
    ///
    /// let context = Context::create();
    ///
    /// assert_eq!(context.get_kind_id("dbg"), 0);
    /// assert_eq!(context.get_kind_id("tbaa"), 1);
    /// assert_eq!(context.get_kind_id("prof"), 2);
    ///
    /// // Custom kind id doesn't exist in LLVM until now:
    /// assert_eq!(context.get_kind_id("foo"), FIRST_CUSTOM_METADATA_KIND_ID);
    /// ```
    pub fn get_kind_id(&self, key: &str) -> u32 {
        unsafe {
            LLVMGetMDKindIDInContext(*self.context, key.as_ptr() as *const i8, key.len() as u32)
        }
    }

    // LLVM 3.9+
    // pub fn get_diagnostic_handler(&self) -> DiagnosticHandler {
    //     let handler = unsafe {
    //         LLVMContextGetDiagnosticHandler(self.context)
    //     };

    //     // REVIEW: Can this be null?

    //     DiagnosticHandler::new(handler)
    // }

    /// Creates an enum `Attribute` in this `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let enum_attribute = context.create_enum_attribute(0, 10);
    ///
    /// assert!(enum_attribute.is_enum());
    /// ```
    #[llvm_versions(4.0 => latest)]
    pub fn create_enum_attribute(&self, kind_id: u32, val: u64) -> Attribute {
        let attribute = unsafe {
            LLVMCreateEnumAttribute(*self.context, kind_id, val)
        };

        Attribute::new(attribute)
    }

    /// Creates a string `Attribute` in this `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let string_attribute = context.create_string_attribute("my_key_123", "my_val");
    ///
    /// assert!(string_attribute.is_string());
    /// ```
    #[llvm_versions(4.0 => latest)]
    pub fn create_string_attribute(&self, key: &str, val: &str) -> Attribute {
        let attribute = unsafe {
            LLVMCreateStringAttribute(*self.context, key.as_ptr() as *const _, key.len() as u32, val.as_ptr() as *const _, val.len() as u32)
        };

        Attribute::new(attribute)
    }

    /// Creates a const string which may be null terminated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let string = context.const_string("my_string", false);
    ///
    /// assert_eq!(string.print_to_string().to_string(), "[9 x i8] c\"my_string\"");
    /// ```
    // SubTypes: Should return VectorValue<IntValue<i8>>
    pub fn const_string(&self, string: &str, null_terminated: bool) -> VectorValue {
        let ptr = unsafe {
            LLVMConstStringInContext(*self.context, string.as_ptr() as *const i8, string.len() as u32, !null_terminated as i32)
        };

        VectorValue::new(ptr)
    }

    pub(crate) fn set_diagnostic_handler(&self, handler: extern "C" fn (LLVMDiagnosticInfoRef, *mut c_void), void_ptr: *mut c_void) {
        unsafe {
            LLVMContextSetDiagnosticHandler(*self.context, Some(handler), void_ptr)
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if Rc::strong_count(&self.context) == 1 {
            unsafe {
                LLVMContextDispose(*self.context);
            }
        }
    }
}

// Alternate strategy would be to just define ownership parameter
// on Context, and only call destructor if true. Not sure of pros/cons
// compared to this approach other than not needing Deref trait's ugly syntax
// REVIEW: Now that Contexts are ref counted, it may not be necessary to
// have this ContextRef type, however Global Contexts throw a wrench in that
// a bit as it is a special case. get_context() methods as well since they do
// not have access to the original Rc. I suppose Context could be Option<Rc<LLVMContextRef>>
// where None is global context
/// A `ContextRef` is a smart pointer allowing borrowed access to a a type's `Context`.
#[derive(Debug, PartialEq, Eq)]
pub struct ContextRef {
    context: Option<Context>,
}

impl ContextRef {
    pub(crate) fn new(context: Context) -> Self {
        ContextRef {
            context: Some(context),
        }
    }
}

impl Deref for ContextRef {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.context.as_ref().expect("ContextRef should never be deref'd after being dropped")
    }
}

impl Drop for ContextRef {
    fn drop(&mut self) {
        // REVIEW: If you forget an Rc type like Context, does that mean it never gets decremented?
        forget(self.context.take());
    }
}
