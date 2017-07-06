use llvm_sys::core::{LLVMAppendBasicBlockInContext, LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDoubleTypeInContext, LLVMFloatTypeInContext, LLVMFP128TypeInContext, LLVMInsertBasicBlockInContext, LLVMInt16TypeInContext, LLVMInt1TypeInContext, LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMStructCreateNamed, LLVMStructSetBody, LLVMStructTypeInContext, LLVMVoidTypeInContext};
use llvm_sys::prelude::{LLVMContextRef, LLVMTypeRef};

use basic_block::BasicBlock;
use builder::Builder;
use module::Module;
use types::{BasicType, FloatType, IntType, StructType, VoidType};
use values::FunctionValue;

use std::ffi::CString;
use std::mem::forget;
use std::ops::Deref;

// From Docs: A single context is not thread safe.
// However, different contexts can execute on different threads simultaneously.
pub struct Context {
    context: LLVMContextRef,
}

impl Context {
    pub fn create() -> Self {
        let context = unsafe {
            LLVMContextCreate()
        };

        Context::new(context)
    }

    pub(crate) fn new(context: LLVMContextRef) -> Self {
        assert!(!context.is_null());

        Context {
            context: context
        }
    }

    pub fn create_builder(&self) -> Builder {
        let builder = unsafe {
            LLVMCreateBuilderInContext(self.context)
        };

        Builder::new(builder)
    }

    pub fn create_module(&self, name: &str) -> Module {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let module = unsafe {
            LLVMModuleCreateWithNameInContext(c_string.as_ptr(), self.context)
        };

        Module::new(module)
    }

    pub fn void_type(&self) -> VoidType {
        let void_type = unsafe {
            LLVMVoidTypeInContext(self.context)
        };

        VoidType::new(void_type)
    }

    pub fn bool_type(&self) -> IntType {
        let bool_type = unsafe {
            LLVMInt1TypeInContext(self.context)
        };

        IntType::new(bool_type)
    }

    pub fn i8_type(&self) -> IntType {
        let i8_type = unsafe {
            LLVMInt8TypeInContext(self.context)
        };

        IntType::new(i8_type)
    }

    pub fn i16_type(&self) -> IntType {
        let i16_type = unsafe {
            LLVMInt16TypeInContext(self.context)
        };

        IntType::new(i16_type)
    }

    pub fn i32_type(&self) -> IntType {
        let i32_type = unsafe {
            LLVMInt32TypeInContext(self.context)
        };

        IntType::new(i32_type)
    }

    pub fn i64_type(&self) -> IntType {
        let i64_type = unsafe {
            LLVMInt64TypeInContext(self.context)
        };

        IntType::new(i64_type)
    }

    pub fn i128_type(&self) -> IntType {
        // REVIEW: The docs says there's a LLVMInt128TypeInContext, but
        // it might only be in a newer version

        let i128_type = unsafe {
            LLVMIntTypeInContext(self.context, 128)
        };

        IntType::new(i128_type)
    }

    pub fn custom_width_int_type(&self, bits: u32) -> IntType {
        let int_type = unsafe {
            LLVMIntTypeInContext(self.context, bits)
        };

        IntType::new(int_type)
    }

    pub fn f32_type(&self) -> FloatType {
        let f32_type = unsafe {
            LLVMFloatTypeInContext(self.context)
        };

        FloatType::new(f32_type)
    }

    pub fn f64_type(&self) -> FloatType {
        let f64_type = unsafe {
            LLVMDoubleTypeInContext(self.context)
        };

        FloatType::new(f64_type)
    }

    pub fn f128_type(&self) -> FloatType {
        let f128_type = unsafe {
            LLVMFP128TypeInContext(self.context)
        };

        FloatType::new(f128_type)
    }

    // REVIEW: AnyType but VoidType? FunctionType?
    // REVIEW: Changed field_types signature, untested
    pub fn struct_type(&self, field_types: &[&BasicType], packed: bool, name: &str) -> StructType {
        let mut field_types: Vec<LLVMTypeRef> = field_types.iter()
                                                           .map(|val| val.as_ref().type_)
                                                           .collect();
        let struct_type = if name.is_empty() {
            unsafe {
                LLVMStructTypeInContext(self.context, field_types.as_mut_ptr(), field_types.len() as u32, packed as i32)
            }
        } else {
            let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

            unsafe {
                let struct_type = LLVMStructCreateNamed(self.context, c_string.as_ptr());

                LLVMStructSetBody(struct_type, field_types.as_mut_ptr(), field_types.len() as u32, packed as i32);

                struct_type
            }
        };

        StructType::new(struct_type)
    }

    pub fn append_basic_block(&self, function: &FunctionValue, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMAppendBasicBlockInContext(self.context, function.fn_value.value, c_string.as_ptr())
        };

        BasicBlock::new(bb)
    }

    pub fn insert_basic_block_after(&self, basic_block: &BasicBlock, name: &str) -> BasicBlock {
        match basic_block.get_next_basic_block() {
            Some(next_basic_block) => self.prepend_basic_block(&next_basic_block, name),
            None => {
                let parent_fn = basic_block.get_parent();

                self.append_basic_block(&parent_fn, name)
            },
        }
    }

    pub fn prepend_basic_block(&self, basic_block: &BasicBlock, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMInsertBasicBlockInContext(self.context, basic_block.basic_block, c_string.as_ptr())
        };

        BasicBlock::new(bb)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.context);
        }
    }
}

pub struct ContextRef {
    context: Option<Context>,
}

impl ContextRef {
    pub fn new(context: Context) -> Self {
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
        forget(self.context.take());
    }
}

#[test]
fn test_no_context_double_free() {
    let context = Context::create();
    let int = context.i8_type();

    {
        int.get_context();
    }
}

#[test]
fn test_no_context_double_free2() {
    let context = Context::create();
    let int = context.i8_type();
    let context2 = int.get_context();

    fn move_(context: Context) {}

    move_(context);

    context2.i8_type().const_int(0, false);
}

#[test]
fn test_get_context_from_contextless_value() {
    let int = IntType::i8_type();

    assert!(!(*int.get_context()).context.is_null());
}
