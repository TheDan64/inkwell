use llvm_sys::core::{LLVMAlignOf, LLVMArrayType, LLVMConstArray, LLVMConstInt, LLVMConstNamedStruct, LLVMConstReal, LLVMCountParamTypes, LLVMDumpType, LLVMFunctionType, LLVMGetParamTypes, LLVMGetTypeContext, LLVMGetTypeKind, LLVMGetUndef, LLVMIsFunctionVarArg, LLVMPointerType, LLVMPrintTypeToString, LLVMStructGetTypeAtIndex, LLVMTypeIsSized, LLVMInt1Type, LLVMInt8Type, LLVMInt16Type, LLVMInt32Type, LLVMInt64Type, LLVMIntType};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};
use llvm_sys::LLVMTypeKind;

use std::ffi::CStr;
use std::fmt;
use std::mem::{transmute, uninitialized};

use context::{Context, ContextRef};
use values::{IntValue, Value};

// Worth noting that types seem to be singletons. At the very least, primitives are.
// Though this is likely only true per thread since LLVM claims to not be very thread-safe.
// TODO: Make not public if possible
// TODO: Might be a good idea to create a google doc spreadsheet to outline which Types should get which methods from Type
pub struct Type {
    pub(crate) type_: LLVMTypeRef,
}

impl Type {
    pub(crate) fn new(type_: LLVMTypeRef) -> Self {
        assert!(!type_.is_null());

        Type {
            type_: type_,
        }
    }

    // NOTE: AnyType
    fn dump_type(&self) {
        unsafe {
            LLVMDumpType(self.type_);
        }
    }

    // NOTE: AnyType
    fn ptr_type(&self, address_space: u32) -> PointerType {
        let ptr_type = unsafe {
            LLVMPointerType(self.type_, address_space)
        };

        PointerType::new(ptr_type)
    }

    // TODO: param_types: &[&AnyType]
    // REVIEW: Is this actually AnyType except FunctionType? VoidType? Can you make a FunctionType from a FunctionType???
    fn fn_type(&self, param_types: &mut [&AnyType], is_var_args: bool) -> FunctionType {
        // WARNING: transmute will no longer work correctly if Type gains more fields
        // We're avoiding reallocation by telling rust Vec<Type> is identical to Vec<LLVMTypeRef>

        // FIXME: Turning AnyType into an enum likely broke transmute
        let mut param_types: &mut [LLVMTypeRef] = unsafe {
            transmute(param_types)
        };

        let fn_type = unsafe {
            LLVMFunctionType(self.type_, param_types.as_mut_ptr(), param_types.len() as u32, is_var_args as i32)
        };

        FunctionType::new(fn_type)
    }

    // NOTE: AnyType? -> ArrayType
    fn array_type(&self, size: u32) -> Self {
        let type_ = unsafe {
            LLVMArrayType(self.type_, size)
        };

        Type::new(type_)
    }

    // NOTE: AnyType? -> ArrayType
    fn const_array(&self, values: Vec<Value>) -> Value {
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

    // NOTE: AnyType?
    // REVIEW: Untested; impl AnyValue?
    fn get_undef(&self) -> Value {
        let value = unsafe {
            LLVMGetUndef(self.type_)
        };

        Value::new(value)
    }

    // NOTE: AnyType
    pub(crate) fn get_kind(&self) -> LLVMTypeKind {
        unsafe {
            LLVMGetTypeKind(self.type_)
        }
    }

    // NOTE: AnyType
    // REVIEW: Untested; Return IntValue?
    fn get_alignment(&self) -> Value {
        let val = unsafe {
            LLVMAlignOf(self.type_)
        };

        Value::new(val)
    }

    fn get_context(&self) -> ContextRef {
        // We don't return an option because LLVM seems
        // to always assign a context, even to types
        // created without an explicit context, somehow

        let context = unsafe {
            LLVMGetTypeContext(self.type_)
        };

        ContextRef::new(Context::new(context))
    }

    /// REVIEW: Untested
    fn is_sized(&self) -> bool {
        unsafe {
            LLVMTypeIsSized(self.type_) == 1
        }
    }
}

impl fmt::Debug for Type {
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

    // REVIEW: Not working
    fn is_var_arg(&self) -> bool {
        unsafe {
            LLVMIsFunctionVarArg(self.fn_type) != 0
        }
    }

    // REVIEW: This was marked as "not working properly". Maybe need more test cases,
    // particularly with types created without an explicit context
    pub fn get_param_types(&self) -> Vec<Type> {
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

    // pub fn is_sized(&self) -> bool {
    //     self.fn_type.is_sized()
    // }
}

impl fmt::Debug for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(self.fn_type))
        };

        write!(f, "FunctionType {{\n    address: {:?}\n    llvm_type: {:?}\n}}", self.fn_type, llvm_type)
    }
}

pub struct IntType {
    int_type: Type,
}

impl IntType {
    pub(crate) fn new(int_type: LLVMTypeRef) -> Self {
        assert!(!int_type.is_null());

        IntType {
            int_type: Type::new(int_type),
        }
    }

    pub fn bool_type() -> Self {
        let type_ = unsafe {
            LLVMInt1Type()
        };

        IntType::new(type_)
    }

    pub fn i8_type() -> Self {
        let type_ = unsafe {
            LLVMInt8Type()
        };

        IntType::new(type_)
    }

    pub fn i16_type() -> Self {
        let type_ = unsafe {
            LLVMInt16Type()
        };

        IntType::new(type_)
    }

    pub fn i32_type() -> Self {
        let type_ = unsafe {
            LLVMInt32Type()
        };

        IntType::new(type_)
    }

    pub fn i64_type() -> Self {
        let type_ = unsafe {
            LLVMInt64Type()
        };

        IntType::new(type_)
    }

    pub fn i128_type() -> Self {
        // REVIEW: The docs says there's a LLVMInt128Type, but
        // it might only be in a newer version

        let type_ = unsafe {
            LLVMIntType(128)
        };

        IntType::new(type_)
    }

    pub fn custom_width_int_type(bits: u32) -> Self {
        let type_ = unsafe {
            LLVMIntType(bits)
        };

        IntType::new(type_)
    }

    pub fn const_int(&self, value: u64, sign_extend: bool) -> IntValue {
        let value = unsafe {
            LLVMConstInt(self.int_type.type_, value, sign_extend as i32)
        };

        IntValue::new(value)
    }

    pub fn fn_type(&self, param_types: &mut [&AnyType], is_var_args: bool) -> FunctionType {
        self.int_type.fn_type(param_types, is_var_args)
    }

    pub fn get_context(&self) -> ContextRef {
        self.int_type.get_context()
    }

    pub fn is_sized(&self) -> bool {
        self.int_type.is_sized()
    }
}

pub struct FloatType {
    float_type: Type,
}

impl FloatType {
    pub(crate) fn new(float_type: LLVMTypeRef) -> Self {
        FloatType {
            float_type: Type::new(float_type),
        }
    }

    pub fn fn_type(&self, param_types: &mut [&AnyType], is_var_args: bool) -> FunctionType {
        self.float_type.fn_type(param_types, is_var_args)
    }

    // TODO: Return FloatValue
    pub fn const_float(&self, value: f64) -> Value {
        let value = unsafe {
            LLVMConstReal(self.float_type.type_, value)
        };

        Value::new(value)
    }

    pub fn is_sized(&self) -> bool {
        self.float_type.is_sized()
    }
}

pub struct StructType {
    struct_type: Type,
}

impl StructType {
    pub(crate) fn new(struct_type: LLVMTypeRef) -> Self {
        StructType {
            struct_type: Type::new(struct_type),
        }
    }

    // REVIEW: Untested
    // TODO: Would be great to be able to smartly be able to do this by field name
    // TODO: LLVM 3.7+ only
    pub fn get_type_at_field_index(&self, index: u32) -> Option<Type> {
        // REVIEW: This should only be used on Struct Types, so add a StructType?
        let type_ = unsafe {
            LLVMStructGetTypeAtIndex(self.struct_type.type_, index)
        };

        if type_.is_null() {
            return None;
        }

        Some(Type::new(type_))
    }

    // TODO: Return StructValue
    // REVIEW: Untested
    // TODO: Better name for num. What's it for?
    pub fn const_struct(&self, value: &mut Value, num: u32) -> Value {
        let val = unsafe {
            LLVMConstNamedStruct(self.struct_type.type_, &mut value.value, num)
        };

        Value::new(val)
    }

    pub fn is_sized(&self) -> bool {
        self.struct_type.is_sized()
    }
}

pub struct VoidType {
    void_type: Type,
}

impl VoidType {
    pub(crate) fn new(void_type: LLVMTypeRef) -> Self {
        VoidType {
            void_type: Type::new(void_type),
        }
    }

    pub fn is_sized(&self) -> bool {
        self.void_type.is_sized()
    }
}

macro_rules! type_set {
    ($trait_name:ident: $($args:ident),*) => (
        pub trait $trait_name {}

        $(
            impl $trait_name for $args {}
        )*
    );
}

pub struct PointerType {
    ptr_type: Type,
}

impl PointerType {
    pub(crate) fn new(ptr_type: LLVMTypeRef) -> Self {
        PointerType {
            ptr_type: Type::new(ptr_type),
        }
    }

    pub fn is_sized(&self) -> bool {
        self.ptr_type.is_sized()
    }
}

type_set! {AnyType: IntType, FloatType, PointerType, StructType, VoidType}

#[test]
fn test_function_type() {
    let context = Context::create();
    let int = context.i8_type();
    let float = context.f32_type();
    let fn_type = int.fn_type(&mut [&int, &int, &float], false);
    let param_types = fn_type.get_param_types();

    assert_eq!(param_types.len(), 3);
    // assert_eq!(param_types[0].type_, int.int_type);
    // assert_eq!(param_types[1].type_, int.int_type);
    // assert_eq!(param_types[2].type_, float.type_);

    // assert!(!fn_type.is_var_arg());

    // let fn_type = int.fn_type(&mut [context.i8_type()], true);

    // assert!(fn_type.is_var_arg());

    // TODO: Test fn_type with different type structs in one call
}
