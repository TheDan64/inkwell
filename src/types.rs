use llvm_sys::core::{LLVMAlignOf, LLVMArrayType, LLVMConstArray, LLVMConstInt, LLVMConstNamedStruct, LLVMConstReal, LLVMCountParamTypes, LLVMDumpType, LLVMFunctionType, LLVMGetParamTypes, LLVMGetTypeContext, LLVMGetTypeKind, LLVMGetUndef, LLVMIsFunctionVarArg, LLVMPointerType, LLVMPrintTypeToString, LLVMStructGetTypeAtIndex, LLVMTypeIsSized};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};
use llvm_sys::LLVMTypeKind;

use std::ffi::CStr;
use std::fmt;
use std::mem::{transmute, uninitialized};
use std::marker::PhantomData;

use context::{Context, ContextRef};
use values::Value;

pub struct Type<'t> {
    pub(crate) type_: LLVMTypeRef,
    phantom: PhantomData<&'t bool>,
}

impl<'t> Type<'t> {
    pub(crate) fn new(type_: LLVMTypeRef) -> Self {
        assert!(!type_.is_null());

        Type {
            type_: type_,
            phantom: PhantomData,
        }
    }

    pub fn dump_type(&self) {
        unsafe {
            LLVMDumpType(self.type_);
        }
    }

    pub fn ptr_type(&self, address_space: u32) -> Self {
        let type_ = unsafe {
            LLVMPointerType(self.type_, address_space)
        };

        Type::new(type_)
    }

    pub fn fn_type(&self, param_types: &mut Vec<Type>, is_var_args: bool) -> FunctionType {
        // WARNING: transmute will no longer work correctly if Type gains more fields
        // We're avoiding reallocation by telling rust Vec<Type> is identical to Vec<LLVMTypeRef>
        let mut param_types: &mut Vec<LLVMTypeRef> = unsafe {
            transmute(param_types)
        };

        let fn_type = unsafe {
            LLVMFunctionType(self.type_, param_types.as_mut_ptr(), param_types.len() as u32, is_var_args as i32) // REVIEW: safe to cast usize to u32?
        };

        FunctionType::new(fn_type)
    }

    pub fn array_type(&self, size: u32) -> Self {
        let type_ = unsafe {
            LLVMArrayType(self.type_, size)
        };

        Type::new(type_)
    }

    pub fn const_int(&self, value: u64, sign_extend: bool) -> Value {
        // REVIEW: What if type is void??

        let value = unsafe {
            LLVMConstInt(self.type_, value, sign_extend as i32)
        };

        Value::new(value)
    }

    pub fn const_float(&self, value: f64) -> Value {
        // REVIEW: What if type is void??

        let value = unsafe {
            LLVMConstReal(self.type_, value)
        };

        Value::new(value)
    }

    pub fn const_array(&self, values: Vec<Value>) -> Value {
        // WARNING: transmute will no longer work correctly if Type gains more fields
        // We're avoiding reallocation by telling rust Vec<Type> is identical to Vec<LLVMTypeRef>
        let mut values: Vec<LLVMValueRef> = unsafe {
            transmute(values)
        };

        let value = unsafe {
            LLVMConstArray(self.type_, values.as_mut_ptr(), values.len() as u32)
        };

        Value::new(value)
    }

    /// REVIEW: Untested
    pub fn get_undef(&self) -> Value {
        let value = unsafe {
            LLVMGetUndef(self.type_)
        };

        Value::new(value)
    }

    /// LLVM 3.7+
    /// REVIEW: Untested
    pub fn get_type_at_struct_index(&self, index: u32) -> Option<Type> {
        // REVIEW: This should only be used on Struct Types, so add a StructType?
        let type_ = unsafe {
            LLVMStructGetTypeAtIndex(self.type_, index)
        };

        if type_.is_null() {
            return None;
        }

        Some(Type::new(type_))
    }

    pub fn get_kind(&self) -> LLVMTypeKind {
        unsafe {
            LLVMGetTypeKind(self.type_)
        }
    }

    /// REVIEW: Untested
    pub fn get_alignment(&self) -> Value {
        let val = unsafe {
            LLVMAlignOf(self.type_)
        };

        Value::new(val)
    }

    /// REVIEW: Untested
    pub fn const_struct(&self, value: &mut Value, num: u32) -> Value {
        // REVIEW: What if not a struct? Need StructType?
        // TODO: Better name for num. What's it for?
        let val = unsafe {
            LLVMConstNamedStruct(self.type_, &mut value.value, num)
        };

        Value::new(val)
    }

    pub(crate) fn get_context(&self) -> ContextRef<'t> { // REVIEW: Option<ContextRef>? I believe types can be context-less (maybe not, it might auto assign the global context (if any??))
        let context = unsafe {
            LLVMGetTypeContext(self.type_)
        };

        ContextRef::new(Context::new(context))
    }

    /// REVIEW: Untested
    pub fn is_sized(&self) -> bool {
        unsafe {
            LLVMTypeIsSized(self.type_) == 1
        }
    }
}

impl<'t> fmt::Debug for Type<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(self.type_))
        };
        write!(f, "Type {{\n    address: {:?}\n    llvm_type: {:?}\n}}", self.type_, llvm_type)
    }
}

pub struct FunctionType {
    pub(crate) fn_type: LLVMTypeRef,
}

impl FunctionType {
    pub(crate) fn new(fn_type: LLVMTypeRef) -> FunctionType {
        assert!(!fn_type.is_null());

        FunctionType {
            fn_type: fn_type
        }
    }

    /// REVIEW: Untested
    pub fn is_var_arg(&self) -> bool {
        unsafe {
            LLVMIsFunctionVarArg(self.fn_type) == 1
        }
    }

    /// FIXME: Not working correctly
    fn get_param_types(&self) -> Vec<Type> {
        let count = self.count_param_types();
        let raw_vec = unsafe { uninitialized() };

        unsafe {
            LLVMGetParamTypes(self.fn_type, raw_vec);

            transmute(Vec::from_raw_parts(raw_vec, count as usize, count as usize))
        }
    }

    pub fn count_param_types(&self) -> u32 {
        unsafe {
            LLVMCountParamTypes(self.fn_type)
        }
    }
}

impl fmt::Debug for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(self.fn_type))
        };

        write!(f, "FunctionType {{\n    address: {:?}\n    llvm_type: {:?}\n}}", self.fn_type, llvm_type)
    }
}
