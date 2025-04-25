//! A `Context` is an opaque owner and manager of core global data.

use crate::InlineAsmDialect;
use libc::c_void;
#[cfg(all(any(feature = "llvm15-0", feature = "llvm16-0"), feature = "typed-pointers"))]
use llvm_sys::core::LLVMContextSetOpaquePointers;
#[llvm_versions(12..)]
use llvm_sys::core::LLVMCreateTypeAttribute;

use llvm_sys::core::LLVMGetInlineAsm;
#[llvm_versions(12..)]
use llvm_sys::core::LLVMGetTypeByName2;
use llvm_sys::core::LLVMMetadataTypeInContext;
#[cfg(not(feature = "typed-pointers"))]
use llvm_sys::core::LLVMPointerTypeInContext;
use llvm_sys::core::{
    LLVMAppendBasicBlockInContext, LLVMConstStringInContext, LLVMConstStructInContext, LLVMContextCreate,
    LLVMContextDispose, LLVMContextSetDiagnosticHandler, LLVMCreateBuilderInContext, LLVMCreateEnumAttribute,
    LLVMCreateStringAttribute, LLVMDoubleTypeInContext, LLVMFP128TypeInContext, LLVMFloatTypeInContext,
    LLVMGetGlobalContext, LLVMGetMDKindIDInContext, LLVMHalfTypeInContext, LLVMInsertBasicBlockInContext,
    LLVMInt16TypeInContext, LLVMInt1TypeInContext, LLVMInt32TypeInContext, LLVMInt64TypeInContext,
    LLVMInt8TypeInContext, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMPPCFP128TypeInContext,
    LLVMStructCreateNamed, LLVMStructTypeInContext, LLVMVoidTypeInContext, LLVMX86FP80TypeInContext,
};
#[allow(deprecated)]
use llvm_sys::core::{LLVMMDNodeInContext, LLVMMDStringInContext};
use llvm_sys::ir_reader::LLVMParseIRInContext;
use llvm_sys::prelude::{LLVMContextRef, LLVMDiagnosticInfoRef, LLVMTypeRef, LLVMValueRef};
use llvm_sys::target::{LLVMIntPtrTypeForASInContext, LLVMIntPtrTypeInContext};
use once_cell::sync::Lazy;
use std::sync::{Mutex, MutexGuard};

use crate::attributes::Attribute;
use crate::basic_block::BasicBlock;
use crate::builder::Builder;
use crate::memory_buffer::MemoryBuffer;
use crate::module::Module;
use crate::support::{to_c_str, LLVMString};
use crate::targets::TargetData;
#[llvm_versions(12..)]
use crate::types::AnyTypeEnum;
use crate::types::MetadataType;
#[cfg(not(feature = "typed-pointers"))]
use crate::types::PointerType;
use crate::types::{AsTypeRef, BasicTypeEnum, FloatType, FunctionType, IntType, StructType, VoidType};
use crate::values::{
    ArrayValue, AsValueRef, BasicMetadataValueEnum, BasicValueEnum, FunctionValue, MetadataValue, PointerValue,
    StructValue,
};
use crate::AddressSpace;

use std::marker::PhantomData;
use std::mem::forget;
use std::ptr;
use std::thread_local;

// The idea of using a Mutex<Context> here and a thread local'd MutexGuard<Context> in
// GLOBAL_CTX_LOCK is to ensure two things:
// 1) Only one thread has access to the global context at a time.
// 2) The thread has shared access across different points in the thread.
// This is still technically unsafe because another program in the same process
// could also be accessing the global context via the C API. `get_global` has been
// marked unsafe for this reason. Iff this isn't the case then this should be fully safe.
static GLOBAL_CTX: Lazy<Mutex<Context>> = Lazy::new(|| unsafe { Mutex::new(Context::new(LLVMGetGlobalContext())) });

thread_local! {
    pub(crate) static GLOBAL_CTX_LOCK: Lazy<MutexGuard<'static, Context>> = Lazy::new(|| {
        GLOBAL_CTX.lock().unwrap_or_else(|e| e.into_inner())
    });
}

/// This struct allows us to share method impls across Context and ContextRef types
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct ContextImpl(pub(crate) LLVMContextRef);

impl ContextImpl {
    pub(crate) unsafe fn new(context: LLVMContextRef) -> Self {
        assert!(!context.is_null());

        #[cfg(all(any(feature = "llvm15-0", feature = "llvm16-0"), feature = "typed-pointers"))]
        unsafe {
            LLVMContextSetOpaquePointers(context, 0)
        };

        ContextImpl(context)
    }

    fn create_builder<'ctx>(&self) -> Builder<'ctx> {
        unsafe { Builder::new(LLVMCreateBuilderInContext(self.0)) }
    }

    fn create_module<'ctx>(&self, name: &str) -> Module<'ctx> {
        let c_string = to_c_str(name);

        unsafe { Module::new(LLVMModuleCreateWithNameInContext(c_string.as_ptr(), self.0)) }
    }

    fn create_module_from_ir<'ctx>(&self, memory_buffer: MemoryBuffer) -> Result<Module<'ctx>, LLVMString> {
        let mut module = ptr::null_mut();
        let mut err_str = ptr::null_mut();

        let code = unsafe { LLVMParseIRInContext(self.0, memory_buffer.memory_buffer, &mut module, &mut err_str) };

        forget(memory_buffer);

        if code == 0 {
            unsafe {
                return Ok(Module::new(module));
            }
        }

        unsafe { Err(LLVMString::new(err_str)) }
    }

    fn create_inline_asm<'ctx>(
        &self,
        ty: FunctionType<'ctx>,
        mut assembly: String,
        mut constraints: String,
        sideeffects: bool,
        alignstack: bool,
        dialect: Option<InlineAsmDialect>,
        #[cfg(not(any(
            feature = "llvm8-0",
            feature = "llvm9-0",
            feature = "llvm10-0",
            feature = "llvm11-0",
            feature = "llvm12-0"
        )))]
        can_throw: bool,
    ) -> PointerValue<'ctx> {
        let value = unsafe {
            LLVMGetInlineAsm(
                ty.as_type_ref(),
                assembly.as_mut_ptr() as *mut ::libc::c_char,
                assembly.len(),
                constraints.as_mut_ptr() as *mut ::libc::c_char,
                constraints.len(),
                sideeffects as i32,
                alignstack as i32,
                dialect.unwrap_or(InlineAsmDialect::ATT).into(),
                #[cfg(not(any(
                    feature = "llvm8-0",
                    feature = "llvm9-0",
                    feature = "llvm10-0",
                    feature = "llvm11-0",
                    feature = "llvm12-0"
                )))]
                {
                    can_throw as i32
                },
            )
        };

        unsafe { PointerValue::new(value) }
    }

    fn void_type<'ctx>(&self) -> VoidType<'ctx> {
        unsafe { VoidType::new(LLVMVoidTypeInContext(self.0)) }
    }

    fn bool_type<'ctx>(&self) -> IntType<'ctx> {
        unsafe { IntType::new(LLVMInt1TypeInContext(self.0)) }
    }

    fn i8_type<'ctx>(&self) -> IntType<'ctx> {
        unsafe { IntType::new(LLVMInt8TypeInContext(self.0)) }
    }

    fn i16_type<'ctx>(&self) -> IntType<'ctx> {
        unsafe { IntType::new(LLVMInt16TypeInContext(self.0)) }
    }

    fn i32_type<'ctx>(&self) -> IntType<'ctx> {
        unsafe { IntType::new(LLVMInt32TypeInContext(self.0)) }
    }

    fn i64_type<'ctx>(&self) -> IntType<'ctx> {
        unsafe { IntType::new(LLVMInt64TypeInContext(self.0)) }
    }

    // TODO: Call LLVMInt128TypeInContext in applicable versions
    fn i128_type<'ctx>(&self) -> IntType<'ctx> {
        self.custom_width_int_type(128)
    }

    fn custom_width_int_type<'ctx>(&self, bits: u32) -> IntType<'ctx> {
        unsafe { IntType::new(LLVMIntTypeInContext(self.0, bits)) }
    }

    fn metadata_type<'ctx>(&self) -> MetadataType<'ctx> {
        unsafe { MetadataType::new(LLVMMetadataTypeInContext(self.0)) }
    }

    fn ptr_sized_int_type<'ctx>(&self, target_data: &TargetData, address_space: Option<AddressSpace>) -> IntType<'ctx> {
        let int_type_ptr = match address_space {
            Some(address_space) => unsafe {
                LLVMIntPtrTypeForASInContext(self.0, target_data.target_data, address_space.0)
            },
            None => unsafe { LLVMIntPtrTypeInContext(self.0, target_data.target_data) },
        };

        unsafe { IntType::new(int_type_ptr) }
    }

    fn f16_type<'ctx>(&self) -> FloatType<'ctx> {
        unsafe { FloatType::new(LLVMHalfTypeInContext(self.0)) }
    }

    fn f32_type<'ctx>(&self) -> FloatType<'ctx> {
        unsafe { FloatType::new(LLVMFloatTypeInContext(self.0)) }
    }

    fn f64_type<'ctx>(&self) -> FloatType<'ctx> {
        unsafe { FloatType::new(LLVMDoubleTypeInContext(self.0)) }
    }

    fn x86_f80_type<'ctx>(&self) -> FloatType<'ctx> {
        unsafe { FloatType::new(LLVMX86FP80TypeInContext(self.0)) }
    }

    fn f128_type<'ctx>(&self) -> FloatType<'ctx> {
        unsafe { FloatType::new(LLVMFP128TypeInContext(self.0)) }
    }

    fn ppc_f128_type<'ctx>(&self) -> FloatType<'ctx> {
        unsafe { FloatType::new(LLVMPPCFP128TypeInContext(self.0)) }
    }

    #[cfg(not(feature = "typed-pointers"))]
    fn ptr_type<'ctx>(&self, address_space: AddressSpace) -> PointerType<'ctx> {
        unsafe { PointerType::new(LLVMPointerTypeInContext(self.0, address_space.0)) }
    }

    fn struct_type<'ctx>(&self, field_types: &[BasicTypeEnum], packed: bool) -> StructType<'ctx> {
        let mut field_types: Vec<LLVMTypeRef> = field_types.iter().map(|val| val.as_type_ref()).collect();
        unsafe {
            StructType::new(LLVMStructTypeInContext(
                self.0,
                field_types.as_mut_ptr(),
                field_types.len() as u32,
                packed as i32,
            ))
        }
    }

    fn opaque_struct_type<'ctx>(&self, name: &str) -> StructType<'ctx> {
        let c_string = to_c_str(name);

        unsafe { StructType::new(LLVMStructCreateNamed(self.0, c_string.as_ptr())) }
    }

    #[llvm_versions(12..)]
    fn get_struct_type<'ctx>(&self, name: &str) -> Option<StructType<'ctx>> {
        let c_string = to_c_str(name);

        let ty = unsafe { LLVMGetTypeByName2(self.0, c_string.as_ptr()) };
        if ty.is_null() {
            return None;
        }

        unsafe { Some(StructType::new(ty)) }
    }

    fn const_struct<'ctx>(&self, values: &[BasicValueEnum], packed: bool) -> StructValue<'ctx> {
        let mut args: Vec<LLVMValueRef> = values.iter().map(|val| val.as_value_ref()).collect();
        unsafe {
            StructValue::new(LLVMConstStructInContext(
                self.0,
                args.as_mut_ptr(),
                args.len() as u32,
                packed as i32,
            ))
        }
    }

    fn append_basic_block<'ctx>(&self, function: FunctionValue<'ctx>, name: &str) -> BasicBlock<'ctx> {
        let c_string = to_c_str(name);

        unsafe {
            BasicBlock::new(LLVMAppendBasicBlockInContext(
                self.0,
                function.as_value_ref(),
                c_string.as_ptr(),
            ))
            .expect("Appending basic block should never fail")
        }
    }

    fn insert_basic_block_after<'ctx>(&self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx> {
        match basic_block.get_next_basic_block() {
            Some(next_basic_block) => self.prepend_basic_block(next_basic_block, name),
            None => {
                let parent_fn = basic_block.get_parent().unwrap();

                self.append_basic_block(parent_fn, name)
            },
        }
    }

    fn prepend_basic_block<'ctx>(&self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx> {
        let c_string = to_c_str(name);

        unsafe {
            BasicBlock::new(LLVMInsertBasicBlockInContext(
                self.0,
                basic_block.basic_block,
                c_string.as_ptr(),
            ))
            .expect("Prepending basic block should never fail")
        }
    }

    #[allow(deprecated)]
    fn metadata_node<'ctx>(&self, values: &[BasicMetadataValueEnum<'ctx>]) -> MetadataValue<'ctx> {
        let mut tuple_values: Vec<LLVMValueRef> = values.iter().map(|val| val.as_value_ref()).collect();
        unsafe {
            MetadataValue::new(LLVMMDNodeInContext(
                self.0,
                tuple_values.as_mut_ptr(),
                tuple_values.len() as u32,
            ))
        }
    }

    #[allow(deprecated)]
    fn metadata_string<'ctx>(&self, string: &str) -> MetadataValue<'ctx> {
        let c_string = to_c_str(string);

        unsafe {
            MetadataValue::new(LLVMMDStringInContext(
                self.0,
                c_string.as_ptr(),
                c_string.to_bytes().len() as u32,
            ))
        }
    }

    fn get_kind_id(&self, key: &str) -> u32 {
        unsafe { LLVMGetMDKindIDInContext(self.0, key.as_ptr() as *const ::libc::c_char, key.len() as u32) }
    }

    fn create_enum_attribute(&self, kind_id: u32, val: u64) -> Attribute {
        unsafe { Attribute::new(LLVMCreateEnumAttribute(self.0, kind_id, val)) }
    }

    fn create_string_attribute(&self, key: &str, val: &str) -> Attribute {
        unsafe {
            Attribute::new(LLVMCreateStringAttribute(
                self.0,
                key.as_ptr() as *const _,
                key.len() as u32,
                val.as_ptr() as *const _,
                val.len() as u32,
            ))
        }
    }

    #[llvm_versions(12..)]
    fn create_type_attribute(&self, kind_id: u32, type_ref: AnyTypeEnum) -> Attribute {
        unsafe { Attribute::new(LLVMCreateTypeAttribute(self.0, kind_id, type_ref.as_type_ref())) }
    }

    fn const_string<'ctx>(&self, string: &[u8], null_terminated: bool) -> ArrayValue<'ctx> {
        unsafe {
            ArrayValue::new(LLVMConstStringInContext(
                self.0,
                string.as_ptr() as *const ::libc::c_char,
                string.len() as u32,
                !null_terminated as i32,
            ))
        }
    }

    fn set_diagnostic_handler(
        &self,
        handler: extern "C" fn(LLVMDiagnosticInfoRef, *mut c_void),
        void_ptr: *mut c_void,
    ) {
        unsafe { LLVMContextSetDiagnosticHandler(self.0, Some(handler), void_ptr) }
    }
}

impl PartialEq<Context> for ContextRef<'_> {
    fn eq(&self, other: &Context) -> bool {
        self.context == other.context
    }
}

impl PartialEq<ContextRef<'_>> for Context {
    fn eq(&self, other: &ContextRef<'_>) -> bool {
        self.context == other.context
    }
}

/// A `Context` is a container for all LLVM entities including `Module`s.
///
/// A `Context` is not thread safe and cannot be shared across threads. Multiple `Context`s
/// can, however, execute on different threads simultaneously according to the LLVM docs.
#[derive(Debug, PartialEq, Eq)]
pub struct Context {
    pub(crate) context: ContextImpl,
}

unsafe impl Send for Context {}

impl Context {
    /// Get raw [`LLVMContextRef`].
    ///
    /// This function is exposed only for interoperability with other LLVM IR libraries.
    /// It's not intended to be used by most users.
    pub fn raw(&self) -> LLVMContextRef {
        self.context.0
    }

    /// Creates a new `Context` from [`LLVMContextRef`].
    ///
    /// # Safety
    ///
    /// This function is exposed only for interoperability with other LLVM IR libraries.
    /// It's not intended to be used by most users, hence marked as unsafe.
    /// Use [`Context::create`] instead.
    pub unsafe fn new(context: LLVMContextRef) -> Self {
        Context {
            context: ContextImpl::new(context),
        }
    }

    /// Creates a new `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// ```
    pub fn create() -> Self {
        unsafe { Context::new(LLVMContextCreate()) }
    }

    /// Gets a `Mutex<Context>` which points to the global context singleton.
    /// This function is marked unsafe because another program within the same
    /// process could easily gain access to the same LLVM context pointer and bypass
    /// our `Mutex`. Therefore, using `Context::create()` is the preferred context
    /// creation function when you do not specifically need the global context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = unsafe {
    ///     Context::get_global(|_global_context| {
    ///         // do stuff
    ///     })
    /// };
    /// ```
    pub unsafe fn get_global<F, R>(func: F) -> R
    where
        F: FnOnce(&Context) -> R,
    {
        GLOBAL_CTX_LOCK.with(|lazy| func(lazy))
    }

    /// Creates a new `Builder` for a `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// ```
    #[inline]
    pub fn create_builder(&self) -> Builder {
        self.context.create_builder()
    }

    /// Creates a new `Module` for a `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// ```
    #[inline]
    pub fn create_module(&self, name: &str) -> Module {
        self.context.create_module(name)
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
    /// let basic_block = context.append_basic_block(fn_val, "entry");
    ///
    /// builder.position_at_end(basic_block);
    /// builder.build_return(None).unwrap();
    ///
    /// let memory_buffer = module.write_bitcode_to_memory();
    ///
    /// let module2 = context.create_module_from_ir(memory_buffer).unwrap();
    /// ```
    // REVIEW: I haven't yet been able to find docs or other wrappers that confirm, but my suspicion
    // is that the method needs to take ownership of the MemoryBuffer... otherwise I see what looks like
    // a double free in valgrind when the MemoryBuffer drops so we are `forget`ting MemoryBuffer here
    // for now until we can confirm this is the correct thing to do
    #[inline]
    pub fn create_module_from_ir(&self, memory_buffer: MemoryBuffer) -> Result<Module, LLVMString> {
        self.context.create_module_from_ir(memory_buffer)
    }

    /// Creates a inline asm function pointer.
    ///
    /// # Example
    /// ```no_run
    /// use std::convert::TryFrom;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_val = module.add_function("my_fn", fn_type, None);
    /// let basic_block = context.append_basic_block(fn_val, "entry");
    ///
    /// builder.position_at_end(basic_block);
    /// let asm_fn = context.i64_type().fn_type(&[context.i64_type().into(), context.i64_type().into()], false);
    /// let asm = context.create_inline_asm(
    ///     asm_fn,
    ///     "syscall".to_string(),
    ///     "=r,{rax},{rdi}".to_string(),
    ///     true,
    ///     false,
    ///     None,
    ///     #[cfg(not(any(
    ///         feature = "llvm8-0",
    ///         feature = "llvm9-0",
    ///         feature = "llvm10-0",
    ///         feature = "llvm11-0",
    ///         feature = "llvm12-0"
    ///     )))]
    ///     false,
    /// );
    /// let params = &[context.i64_type().const_int(60, false).into(), context.i64_type().const_int(1, false).into()];
    ///
    /// #[cfg(any(
    ///     feature = "llvm8-0",
    ///     feature = "llvm9-0",
    ///     feature = "llvm10-0",
    ///     feature = "llvm11-0",
    ///     feature = "llvm12-0",
    ///     feature = "llvm13-0",
    ///     feature = "llvm14-0"
    /// ))]
    /// {
    ///     use inkwell::values::CallableValue;
    ///     let callable_value = CallableValue::try_from(asm).unwrap();
    ///     builder.build_call(callable_value, params, "exit").unwrap();
    /// }
    ///
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-1"))]
    /// builder.build_indirect_call(asm_fn, asm, params, "exit").unwrap();
    ///
    /// builder.build_return(None).unwrap();
    /// ```
    #[inline]
    pub fn create_inline_asm<'ctx>(
        &'ctx self,
        ty: FunctionType<'ctx>,
        assembly: String,
        constraints: String,
        sideeffects: bool,
        alignstack: bool,
        dialect: Option<InlineAsmDialect>,
        #[cfg(not(any(
            feature = "llvm8-0",
            feature = "llvm9-0",
            feature = "llvm10-0",
            feature = "llvm11-0",
            feature = "llvm12-0"
        )))]
        can_throw: bool,
    ) -> PointerValue<'ctx> {
        self.context.create_inline_asm(
            ty,
            assembly,
            constraints,
            sideeffects,
            alignstack,
            dialect,
            #[cfg(not(any(
                feature = "llvm8-0",
                feature = "llvm9-0",
                feature = "llvm10-0",
                feature = "llvm11-0",
                feature = "llvm12-0"
            )))]
            can_throw,
        )
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
    /// assert_eq!(void_type.get_context(), context);
    /// ```
    #[inline]
    pub fn void_type(&self) -> VoidType {
        self.context.void_type()
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
    /// assert_eq!(bool_type.get_context(), context);
    /// ```
    #[inline]
    pub fn bool_type(&self) -> IntType {
        self.context.bool_type()
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
    /// assert_eq!(i8_type.get_context(), context);
    /// ```
    #[inline]
    pub fn i8_type(&self) -> IntType {
        self.context.i8_type()
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
    /// assert_eq!(i16_type.get_context(), context);
    /// ```
    #[inline]
    pub fn i16_type(&self) -> IntType {
        self.context.i16_type()
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
    /// assert_eq!(i32_type.get_context(), context);
    /// ```
    #[inline]
    pub fn i32_type(&self) -> IntType {
        self.context.i32_type()
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
    /// assert_eq!(i64_type.get_context(), context);
    /// ```
    #[inline]
    pub fn i64_type(&self) -> IntType {
        self.context.i64_type()
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
    /// assert_eq!(i128_type.get_context(), context);
    /// ```
    #[inline]
    pub fn i128_type(&self) -> IntType {
        self.context.i128_type()
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
    /// assert_eq!(i42_type.get_context(), context);
    /// ```
    #[inline]
    pub fn custom_width_int_type(&self, bits: u32) -> IntType {
        self.context.custom_width_int_type(bits)
    }

    /// Gets the `MetadataType` representing 128 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::values::IntValue;
    ///
    /// let context = Context::create();
    /// let md_type = context.metadata_type();
    ///
    /// assert_eq!(md_type.get_context(), context);
    /// ```
    #[inline]
    pub fn metadata_type(&self) -> MetadataType {
        self.context.metadata_type()
    }

    /// Gets the `IntType` representing a bit width of a pointer. It will be assigned the referenced context.
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
    /// let int_type = context.ptr_sized_int_type(&target_data, None);
    /// ```
    #[inline]
    pub fn ptr_sized_int_type(&self, target_data: &TargetData, address_space: Option<AddressSpace>) -> IntType {
        self.context.ptr_sized_int_type(target_data, address_space)
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
    /// assert_eq!(f16_type.get_context(), context);
    /// ```
    #[inline]
    pub fn f16_type(&self) -> FloatType {
        self.context.f16_type()
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
    /// assert_eq!(f32_type.get_context(), context);
    /// ```
    #[inline]
    pub fn f32_type(&self) -> FloatType {
        self.context.f32_type()
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
    /// assert_eq!(f64_type.get_context(), context);
    /// ```
    #[inline]
    pub fn f64_type(&self) -> FloatType {
        self.context.f64_type()
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
    /// assert_eq!(x86_f80_type.get_context(), context);
    /// ```
    #[inline]
    pub fn x86_f80_type(&self) -> FloatType {
        self.context.x86_f80_type()
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
    /// assert_eq!(f128_type.get_context(), context);
    /// ```
    // IEEE 754-2008’s binary128 floats according to https://internals.rust-lang.org/t/pre-rfc-introduction-of-half-and-quadruple-precision-floats-f16-and-f128/7521
    #[inline]
    pub fn f128_type(&self) -> FloatType {
        self.context.f128_type()
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
    /// assert_eq!(f128_type.get_context(), context);
    /// ```
    // Two 64 bits according to https://internals.rust-lang.org/t/pre-rfc-introduction-of-half-and-quadruple-precision-floats-f16-and-f128/7521
    #[inline]
    pub fn ppc_f128_type(&self) -> FloatType {
        self.context.ppc_f128_type()
    }

    /// Gets the `PointerType`. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let ptr_type = context.ptr_type(AddressSpace::default());
    ///
    /// assert_eq!(ptr_type.get_address_space(), AddressSpace::default());
    /// assert_eq!(ptr_type.get_context(), context);
    /// ```
    #[cfg(not(feature = "typed-pointers"))]
    #[inline]
    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.context.ptr_type(address_space)
    }

    /// Creates a `StructType` definition from heterogeneous types in the current `Context`.
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
    #[inline]
    pub fn struct_type(&self, field_types: &[BasicTypeEnum], packed: bool) -> StructType {
        self.context.struct_type(field_types, packed)
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
    #[inline]
    pub fn opaque_struct_type(&self, name: &str) -> StructType {
        self.context.opaque_struct_type(name)
    }

    /// Gets a named [`StructType`] from this `Context`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    ///
    /// assert!(context.get_struct_type("foo").is_none());
    ///
    /// let opaque = context.opaque_struct_type("foo");
    ///
    /// assert_eq!(context.get_struct_type("foo").unwrap(), opaque);
    /// ```
    #[inline]
    #[llvm_versions(12..)]
    pub fn get_struct_type<'ctx>(&self, name: &str) -> Option<StructType<'ctx>> {
        self.context.get_struct_type(name)
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
    #[inline]
    pub fn const_struct(&self, values: &[BasicValueEnum], packed: bool) -> StructValue {
        self.context.const_struct(values, packed)
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
    /// let entry_basic_block = context.append_basic_block(fn_value, "entry");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 1);
    ///
    /// let last_basic_block = context.append_basic_block(fn_value, "last");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 2);
    /// assert_eq!(fn_value.get_first_basic_block().unwrap(), entry_basic_block);
    /// assert_eq!(fn_value.get_last_basic_block().unwrap(), last_basic_block);
    /// ```
    #[inline]
    pub fn append_basic_block<'ctx>(&'ctx self, function: FunctionValue<'ctx>, name: &str) -> BasicBlock<'ctx> {
        self.context.append_basic_block(function, name)
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
    /// let entry_basic_block = context.append_basic_block(fn_value, "entry");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 1);
    ///
    /// let last_basic_block = context.insert_basic_block_after(entry_basic_block, "last");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 2);
    /// assert_eq!(fn_value.get_first_basic_block().unwrap(), entry_basic_block);
    /// assert_eq!(fn_value.get_last_basic_block().unwrap(), last_basic_block);
    /// ```
    // REVIEW: What happens when using these methods and the BasicBlock doesn't have a parent?
    // Should they be callable at all? Needs testing to see what LLVM will do, I suppose. See below unwrap.
    // Maybe need SubTypes: BasicBlock<HasParent>, BasicBlock<Orphan>?
    #[inline]
    pub fn insert_basic_block_after<'ctx>(&'ctx self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx> {
        self.context.insert_basic_block_after(basic_block, name)
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
    /// let entry_basic_block = context.append_basic_block(fn_value, "entry");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 1);
    ///
    /// let first_basic_block = context.prepend_basic_block(entry_basic_block, "first");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 2);
    /// assert_eq!(fn_value.get_first_basic_block().unwrap(), first_basic_block);
    /// assert_eq!(fn_value.get_last_basic_block().unwrap(), entry_basic_block);
    /// ```
    #[inline]
    pub fn prepend_basic_block<'ctx>(&'ctx self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx> {
        self.context.prepend_basic_block(basic_block, name)
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
    /// let void_type = context.void_type();
    ///
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let fn_type = void_type.fn_type(&[f32_type.into()], false);
    /// let fn_value = module.add_function("my_func", fn_type, None);
    /// let entry_block = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_block);
    ///
    /// let ret_instr = builder.build_return(None).unwrap();
    ///
    /// assert!(md_node.is_node());
    ///
    /// ret_instr.set_metadata(md_node, 0);
    /// ```
    // REVIEW: Maybe more helpful to beginners to call this metadata_tuple?
    // REVIEW: Seems to be unassgned to anything
    #[inline]
    pub fn metadata_node<'ctx>(&'ctx self, values: &[BasicMetadataValueEnum<'ctx>]) -> MetadataValue<'ctx> {
        self.context.metadata_node(values)
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
    /// let void_type = context.void_type();
    ///
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let fn_type = void_type.fn_type(&[f32_type.into()], false);
    /// let fn_value = module.add_function("my_func", fn_type, None);
    /// let entry_block = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_block);
    ///
    /// let ret_instr = builder.build_return(None).unwrap();
    ///
    /// assert!(md_string.is_string());
    ///
    /// ret_instr.set_metadata(md_string, 0);
    /// ```
    // REVIEW: Seems to be unassigned to anything
    #[inline]
    pub fn metadata_string(&self, string: &str) -> MetadataValue {
        self.context.metadata_string(string)
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
    #[inline]
    pub fn get_kind_id(&self, key: &str) -> u32 {
        self.context.get_kind_id(key)
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
    #[inline]
    pub fn create_enum_attribute(&self, kind_id: u32, val: u64) -> Attribute {
        self.context.create_enum_attribute(kind_id, val)
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
    #[inline]
    pub fn create_string_attribute(&self, key: &str, val: &str) -> Attribute {
        self.context.create_string_attribute(key, val)
    }

    /// Create an enum `Attribute` with an `AnyTypeEnum` attached to it.
    ///
    /// # Example
    /// ```rust
    /// use inkwell::context::Context;
    /// use inkwell::attributes::Attribute;
    /// use inkwell::types::AnyType;
    ///
    /// let context = Context::create();
    /// let kind_id = Attribute::get_named_enum_kind_id("sret");
    /// let any_type = context.i32_type().as_any_type_enum();
    /// let type_attribute = context.create_type_attribute(
    ///     kind_id,
    ///     any_type,
    /// );
    ///
    /// assert!(type_attribute.is_type());
    /// assert_eq!(type_attribute.get_type_value(), any_type);
    /// assert_ne!(type_attribute.get_type_value(), context.i64_type().as_any_type_enum());
    /// ```
    #[inline]
    #[llvm_versions(12..)]
    pub fn create_type_attribute(&self, kind_id: u32, type_ref: AnyTypeEnum) -> Attribute {
        self.context.create_type_attribute(kind_id, type_ref)
    }

    /// Creates a const string which may be null terminated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::values::AnyValue;
    ///
    /// let context = Context::create();
    /// let string = context.const_string(b"my_string", false);
    ///
    /// assert_eq!(string.print_to_string().to_string(), "[9 x i8] c\"my_string\"");
    /// ```
    // SubTypes: Should return ArrayValue<IntValue<i8>>
    #[inline]
    pub fn const_string(&self, string: &[u8], null_terminated: bool) -> ArrayValue {
        self.context.const_string(string, null_terminated)
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn set_diagnostic_handler(
        &self,
        handler: extern "C" fn(LLVMDiagnosticInfoRef, *mut c_void),
        void_ptr: *mut c_void,
    ) {
        self.context.set_diagnostic_handler(handler, void_ptr)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.context.0);
        }
    }
}

/// A `ContextRef` is a smart pointer allowing borrowed access to a type's `Context`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ContextRef<'ctx> {
    pub(crate) context: ContextImpl,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> ContextRef<'ctx> {
    /// Get raw [`LLVMContextRef`].
    ///
    /// This function is exposed only for interoperability with other LLVM IR libraries.
    /// It's not intended to be used by most users.
    pub fn raw(&self) -> LLVMContextRef {
        self.context.0
    }

    /// Creates a new `ContextRef` from [`LLVMContextRef`].
    ///
    /// # Safety
    ///
    /// This function is exposed only for interoperability with other LLVM IR libraries.
    /// It's not intended to be used by most users, hence marked as unsafe.
    pub unsafe fn new(context: LLVMContextRef) -> Self {
        ContextRef {
            context: ContextImpl::new(context),
            _marker: PhantomData,
        }
    }

    /// Creates a new `Builder` for a `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let builder = context.create_builder();
    /// ```
    #[inline]
    pub fn create_builder(&self) -> Builder<'ctx> {
        self.context.create_builder()
    }

    /// Creates a new `Module` for a `Context`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// ```
    #[inline]
    pub fn create_module(&self, name: &str) -> Module<'ctx> {
        self.context.create_module(name)
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
    /// let basic_block = context.append_basic_block(fn_val, "entry");
    ///
    /// builder.position_at_end(basic_block);
    /// builder.build_return(None).unwrap();
    ///
    /// let memory_buffer = module.write_bitcode_to_memory();
    ///
    /// let module2 = context.create_module_from_ir(memory_buffer).unwrap();
    /// ```
    // REVIEW: I haven't yet been able to find docs or other wrappers that confirm, but my suspicion
    // is that the method needs to take ownership of the MemoryBuffer... otherwise I see what looks like
    // a double free in valgrind when the MemoryBuffer drops so we are `forget`ting MemoryBuffer here
    // for now until we can confirm this is the correct thing to do
    #[inline]
    pub fn create_module_from_ir(&self, memory_buffer: MemoryBuffer) -> Result<Module<'ctx>, LLVMString> {
        self.context.create_module_from_ir(memory_buffer)
    }

    /// Creates a inline asm function pointer.
    ///
    /// # Example
    /// ```no_run
    /// use std::convert::TryFrom;
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let builder = context.create_builder();
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    /// let fn_val = module.add_function("my_fn", fn_type, None);
    /// let basic_block = context.append_basic_block(fn_val, "entry");
    ///
    /// builder.position_at_end(basic_block);
    /// let asm_fn = context.i64_type().fn_type(&[context.i64_type().into(), context.i64_type().into()], false);
    /// let asm = context.create_inline_asm(
    ///     asm_fn,
    ///     "syscall".to_string(),
    ///     "=r,{rax},{rdi}".to_string(),
    ///     true,
    ///     false,
    ///     None,
    ///     #[cfg(not(any(
    ///         feature = "llvm8-0",
    ///         feature = "llvm9-0",
    ///         feature = "llvm10-0",
    ///         feature = "llvm11-0",
    ///         feature = "llvm12-0"
    ///     )))]
    ///     false,
    /// );
    /// let params = &[context.i64_type().const_int(60, false).into(), context.i64_type().const_int(1, false).into()];
    ///
    /// #[cfg(any(
    ///     feature = "llvm8-0",
    ///     feature = "llvm9-0",
    ///     feature = "llvm10-0",
    ///     feature = "llvm11-0",
    ///     feature = "llvm12-0",
    ///     feature = "llvm13-0",
    ///     feature = "llvm14-0"
    /// ))]
    /// {
    ///     use inkwell::values::CallableValue;
    ///     let callable_value = CallableValue::try_from(asm).unwrap();
    ///     builder.build_call(callable_value, params, "exit").unwrap();
    /// }
    ///
    /// #[cfg(any(feature = "llvm15-0", feature = "llvm16-0", feature = "llvm17-0", feature = "llvm18-1"))]
    /// builder.build_indirect_call(asm_fn, asm, params, "exit").unwrap();
    ///
    /// builder.build_return(None).unwrap();
    /// ```
    #[inline]
    pub fn create_inline_asm(
        &self,
        ty: FunctionType<'ctx>,
        assembly: String,
        constraints: String,
        sideeffects: bool,
        alignstack: bool,
        dialect: Option<InlineAsmDialect>,
        #[cfg(not(any(
            feature = "llvm8-0",
            feature = "llvm9-0",
            feature = "llvm10-0",
            feature = "llvm11-0",
            feature = "llvm12-0"
        )))]
        can_throw: bool,
    ) -> PointerValue<'ctx> {
        self.context.create_inline_asm(
            ty,
            assembly,
            constraints,
            sideeffects,
            alignstack,
            dialect,
            #[cfg(not(any(
                feature = "llvm8-0",
                feature = "llvm9-0",
                feature = "llvm10-0",
                feature = "llvm11-0",
                feature = "llvm12-0"
            )))]
            can_throw,
        )
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
    /// assert_eq!(void_type.get_context(), context);
    /// ```
    #[inline]
    pub fn void_type(&self) -> VoidType<'ctx> {
        self.context.void_type()
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
    /// assert_eq!(bool_type.get_context(), context);
    /// ```
    #[inline]
    pub fn bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
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
    /// assert_eq!(i8_type.get_context(), context);
    /// ```
    #[inline]
    pub fn i8_type(&self) -> IntType<'ctx> {
        self.context.i8_type()
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
    /// assert_eq!(i16_type.get_context(), context);
    /// ```
    #[inline]
    pub fn i16_type(&self) -> IntType<'ctx> {
        self.context.i16_type()
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
    /// assert_eq!(i32_type.get_context(), context);
    /// ```
    #[inline]
    pub fn i32_type(&self) -> IntType<'ctx> {
        self.context.i32_type()
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
    /// assert_eq!(i64_type.get_context(), context);
    /// ```
    #[inline]
    pub fn i64_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
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
    /// assert_eq!(i128_type.get_context(), context);
    /// ```
    #[inline]
    pub fn i128_type(&self) -> IntType<'ctx> {
        self.context.i128_type()
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
    /// assert_eq!(i42_type.get_context(), context);
    /// ```
    #[inline]
    pub fn custom_width_int_type(&self, bits: u32) -> IntType<'ctx> {
        self.context.custom_width_int_type(bits)
    }

    /// Gets the `MetadataType` representing 128 bit width. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```
    /// use inkwell::context::Context;
    /// use inkwell::values::IntValue;
    ///
    /// let context = Context::create();
    /// let md_type = context.metadata_type();
    ///
    /// assert_eq!(md_type.get_context(), context);
    /// ```
    #[inline]
    pub fn metadata_type(&self) -> MetadataType<'ctx> {
        self.context.metadata_type()
    }

    /// Gets the `IntType` representing a bit width of a pointer. It will be assigned the referenced context.
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
    /// let int_type = context.ptr_sized_int_type(&target_data, None);
    /// ```
    #[inline]
    pub fn ptr_sized_int_type(&self, target_data: &TargetData, address_space: Option<AddressSpace>) -> IntType<'ctx> {
        self.context.ptr_sized_int_type(target_data, address_space)
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
    /// assert_eq!(f16_type.get_context(), context);
    /// ```
    #[inline]
    pub fn f16_type(&self) -> FloatType<'ctx> {
        self.context.f16_type()
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
    /// assert_eq!(f32_type.get_context(), context);
    /// ```
    #[inline]
    pub fn f32_type(&self) -> FloatType<'ctx> {
        self.context.f32_type()
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
    /// assert_eq!(f64_type.get_context(), context);
    /// ```
    #[inline]
    pub fn f64_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
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
    /// assert_eq!(x86_f80_type.get_context(), context);
    /// ```
    #[inline]
    pub fn x86_f80_type(&self) -> FloatType<'ctx> {
        self.context.x86_f80_type()
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
    /// assert_eq!(f128_type.get_context(), context);
    /// ```
    // IEEE 754-2008’s binary128 floats according to https://internals.rust-lang.org/t/pre-rfc-introduction-of-half-and-quadruple-precision-floats-f16-and-f128/7521
    #[inline]
    pub fn f128_type(&self) -> FloatType<'ctx> {
        self.context.f128_type()
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
    /// assert_eq!(f128_type.get_context(), context);
    /// ```
    // Two 64 bits according to https://internals.rust-lang.org/t/pre-rfc-introduction-of-half-and-quadruple-precision-floats-f16-and-f128/7521
    #[inline]
    pub fn ppc_f128_type(&self) -> FloatType<'ctx> {
        self.context.ppc_f128_type()
    }

    /// Gets the `PointerType`. It will be assigned the current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::AddressSpace;
    ///
    /// let context = Context::create();
    /// let ptr_type = context.ptr_type(AddressSpace::default());
    ///
    /// assert_eq!(ptr_type.get_address_space(), AddressSpace::default());
    /// assert_eq!(ptr_type.get_context(), context);
    /// ```
    #[cfg(not(feature = "typed-pointers"))]
    #[inline]
    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType<'ctx> {
        self.context.ptr_type(address_space)
    }

    /// Creates a `StructType` definition from heterogeneous types in the current `Context`.
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
    #[inline]
    pub fn struct_type(&self, field_types: &[BasicTypeEnum<'ctx>], packed: bool) -> StructType<'ctx> {
        self.context.struct_type(field_types, packed)
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
    #[inline]
    pub fn opaque_struct_type(&self, name: &str) -> StructType<'ctx> {
        self.context.opaque_struct_type(name)
    }

    /// Gets a named [`StructType`] from this `Context`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use inkwell::context::Context;
    ///
    /// let context = Context::create();
    ///
    /// assert!(context.get_struct_type("foo").is_none());
    ///
    /// let opaque = context.opaque_struct_type("foo");
    ///
    /// assert_eq!(context.get_struct_type("foo").unwrap(), opaque);
    /// ```
    #[inline]
    #[llvm_versions(12..)]
    pub fn get_struct_type(&self, name: &str) -> Option<StructType<'ctx>> {
        self.context.get_struct_type(name)
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
    #[inline]
    pub fn const_struct(&self, values: &[BasicValueEnum<'ctx>], packed: bool) -> StructValue<'ctx> {
        self.context.const_struct(values, packed)
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
    /// let entry_basic_block = context.append_basic_block(fn_value, "entry");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 1);
    ///
    /// let last_basic_block = context.append_basic_block(fn_value, "last");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 2);
    /// assert_eq!(fn_value.get_first_basic_block().unwrap(), entry_basic_block);
    /// assert_eq!(fn_value.get_last_basic_block().unwrap(), last_basic_block);
    /// ```
    #[inline]
    pub fn append_basic_block(&self, function: FunctionValue<'ctx>, name: &str) -> BasicBlock<'ctx> {
        self.context.append_basic_block(function, name)
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
    /// let entry_basic_block = context.append_basic_block(fn_value, "entry");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 1);
    ///
    /// let last_basic_block = context.insert_basic_block_after(entry_basic_block, "last");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 2);
    /// assert_eq!(fn_value.get_first_basic_block().unwrap(), entry_basic_block);
    /// assert_eq!(fn_value.get_last_basic_block().unwrap(), last_basic_block);
    /// ```
    // REVIEW: What happens when using these methods and the BasicBlock doesn't have a parent?
    // Should they be callable at all? Needs testing to see what LLVM will do, I suppose. See below unwrap.
    // Maybe need SubTypes: BasicBlock<HasParent>, BasicBlock<Orphan>?
    #[inline]
    pub fn insert_basic_block_after(&self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx> {
        self.context.insert_basic_block_after(basic_block, name)
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
    /// let entry_basic_block = context.append_basic_block(fn_value, "entry");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 1);
    ///
    /// let first_basic_block = context.prepend_basic_block(entry_basic_block, "first");
    ///
    /// assert_eq!(fn_value.count_basic_blocks(), 2);
    /// assert_eq!(fn_value.get_first_basic_block().unwrap(), first_basic_block);
    /// assert_eq!(fn_value.get_last_basic_block().unwrap(), entry_basic_block);
    /// ```
    #[inline]
    pub fn prepend_basic_block(&self, basic_block: BasicBlock<'ctx>, name: &str) -> BasicBlock<'ctx> {
        self.context.prepend_basic_block(basic_block, name)
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
    /// let void_type = context.void_type();
    ///
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let fn_type = void_type.fn_type(&[f32_type.into()], false);
    /// let fn_value = module.add_function("my_func", fn_type, None);
    /// let entry_block = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_block);
    ///
    /// let ret_instr = builder.build_return(None).unwrap();
    ///
    /// assert!(md_node.is_node());
    ///
    /// ret_instr.set_metadata(md_node, 0);
    /// ```
    // REVIEW: Maybe more helpful to beginners to call this metadata_tuple?
    // REVIEW: Seems to be unassgned to anything
    #[inline]
    pub fn metadata_node(&self, values: &[BasicMetadataValueEnum<'ctx>]) -> MetadataValue<'ctx> {
        self.context.metadata_node(values)
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
    /// let void_type = context.void_type();
    ///
    /// let builder = context.create_builder();
    /// let module = context.create_module("my_mod");
    /// let fn_type = void_type.fn_type(&[f32_type.into()], false);
    /// let fn_value = module.add_function("my_func", fn_type, None);
    /// let entry_block = context.append_basic_block(fn_value, "entry");
    ///
    /// builder.position_at_end(entry_block);
    ///
    /// let ret_instr = builder.build_return(None).unwrap();
    ///
    /// assert!(md_string.is_string());
    ///
    /// ret_instr.set_metadata(md_string, 0);
    /// ```
    // REVIEW: Seems to be unassigned to anything
    #[inline]
    pub fn metadata_string(&self, string: &str) -> MetadataValue<'ctx> {
        self.context.metadata_string(string)
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
    #[inline]
    pub fn get_kind_id(&self, key: &str) -> u32 {
        self.context.get_kind_id(key)
    }

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
    #[inline]
    pub fn create_enum_attribute(&self, kind_id: u32, val: u64) -> Attribute {
        self.context.create_enum_attribute(kind_id, val)
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
    #[inline]
    pub fn create_string_attribute(&self, key: &str, val: &str) -> Attribute {
        self.context.create_string_attribute(key, val)
    }

    /// Create an enum `Attribute` with an `AnyTypeEnum` attached to it.
    ///
    /// # Example
    /// ```rust
    /// use inkwell::context::Context;
    /// use inkwell::attributes::Attribute;
    /// use inkwell::types::AnyType;
    ///
    /// let context = Context::create();
    /// let kind_id = Attribute::get_named_enum_kind_id("sret");
    /// let any_type = context.i32_type().as_any_type_enum();
    /// let type_attribute = context.create_type_attribute(
    ///     kind_id,
    ///     any_type,
    /// );
    ///
    /// assert!(type_attribute.is_type());
    /// assert_eq!(type_attribute.get_type_value(), any_type);
    /// assert_ne!(type_attribute.get_type_value(), context.i64_type().as_any_type_enum());
    /// ```
    #[inline]
    #[llvm_versions(12..)]
    pub fn create_type_attribute(&self, kind_id: u32, type_ref: AnyTypeEnum) -> Attribute {
        self.context.create_type_attribute(kind_id, type_ref)
    }

    /// Creates a const string which may be null terminated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::context::Context;
    /// use inkwell::values::AnyValue;
    ///
    /// let context = Context::create();
    /// let string = context.const_string(b"my_string", false);
    ///
    /// assert_eq!(string.print_to_string().to_string(), "[9 x i8] c\"my_string\"");
    /// ```
    // SubTypes: Should return ArrayValue<IntValue<i8>>
    #[inline]
    pub fn const_string(&self, string: &[u8], null_terminated: bool) -> ArrayValue<'ctx> {
        self.context.const_string(string, null_terminated)
    }

    #[inline]
    pub(crate) fn set_diagnostic_handler(
        &self,
        handler: extern "C" fn(LLVMDiagnosticInfoRef, *mut c_void),
        void_ptr: *mut c_void,
    ) {
        self.context.set_diagnostic_handler(handler, void_ptr)
    }
}

/// This trait abstracts an LLVM `Context` type and should be implemented with caution.
pub unsafe trait AsContextRef<'ctx> {
    /// Returns the internal LLVM reference behind the type
    fn as_ctx_ref(&self) -> LLVMContextRef;
}

unsafe impl<'ctx> AsContextRef<'ctx> for &'ctx Context {
    /// Acquires the underlying raw pointer belonging to this `Context` type.
    fn as_ctx_ref(&self) -> LLVMContextRef {
        self.context.0
    }
}

unsafe impl<'ctx> AsContextRef<'ctx> for ContextRef<'ctx> {
    /// Acquires the underlying raw pointer belonging to this `ContextRef` type.
    fn as_ctx_ref(&self) -> LLVMContextRef {
        self.context.0
    }
}
