use llvm_sys::core::{LLVMAlignOf, LLVMArrayType, LLVMConstArray, LLVMConstInt, LLVMConstNamedStruct, LLVMConstReal, LLVMCountParamTypes, LLVMDumpType, LLVMFunctionType, LLVMGetParamTypes, LLVMGetTypeContext, LLVMGetTypeKind, LLVMGetUndef, LLVMIsFunctionVarArg, LLVMPointerType, LLVMPrintTypeToString, LLVMStructGetTypeAtIndex, LLVMTypeIsSized, LLVMInt1Type, LLVMInt8Type, LLVMInt16Type, LLVMInt32Type, LLVMInt64Type, LLVMIntType};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};
use llvm_sys::LLVMTypeKind;

use std::ffi::CStr;
use std::fmt;
use std::mem::{transmute, forget};

use context::{Context, ContextRef};
use values::{IntValue, StructValue, Value};

// Worth noting that types seem to be singletons. At the very least, primitives are.
// Though this is likely only true per thread since LLVM claims to not be very thread-safe.
// TODO: Make not public if possible
#[doc(hidden)]
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

    fn ptr_type(&self, address_space: u32) -> PointerType {
        let ptr_type = unsafe {
            LLVMPointerType(self.type_, address_space)
        };

        PointerType::new(ptr_type)
    }

    // REVIEW: Is this actually AnyType except FunctionType? VoidType? Can you make a FunctionType from a FunctionType???
    fn fn_type(&self, param_types: &[&AnyType], is_var_args: bool) -> FunctionType {
        let mut param_types: Vec<LLVMTypeRef> = param_types.iter()
                                                           .map(|val| val.as_ref().type_)
                                                           .collect();
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
    pub(crate) fn_type: Type,
}

impl FunctionType {
    pub(crate) fn new(fn_type: LLVMTypeRef) -> FunctionType {
        assert!(!fn_type.is_null());

        FunctionType {
            fn_type: Type::new(fn_type)
        }
    }

    // REVIEW: Not working
    fn is_var_arg(&self) -> bool {
        unsafe {
            LLVMIsFunctionVarArg(self.fn_type.type_) != 0
        }
    }

    pub fn get_param_types(&self) -> Vec<BasicTypeEnum> {
        let count = self.count_param_types();
        let mut raw_vec: Vec<LLVMTypeRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetParamTypes(self.fn_type.type_, ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| BasicTypeEnum::new(*val)).collect()
    }

    pub fn count_param_types(&self) -> u32 {
        unsafe {
            LLVMCountParamTypes(self.fn_type.type_)
        }
    }

    pub fn is_sized(&self) -> bool {
        self.fn_type.is_sized()
    }

    pub fn get_context(&self) -> ContextRef {
        self.fn_type.get_context()
    }

    pub fn ptr_type(&self, address_space: u32) -> PointerType {
        self.fn_type.ptr_type(address_space)
    }
}

impl fmt::Debug for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(self.fn_type.type_))
        };

        write!(f, "FunctionType {{\n    address: {:?}\n    llvm_type: {:?}\n}}", self.fn_type.type_, llvm_type)
    }
}

impl AsRef<Type> for FunctionType {
    fn as_ref(&self) -> &Type {
        &self.fn_type
    }
}

#[derive(Debug)]
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

    pub fn fn_type(&self, param_types: &[&AnyType], is_var_args: bool) -> FunctionType {
        self.int_type.fn_type(param_types, is_var_args)
    }

    pub fn get_context(&self) -> ContextRef {
        self.int_type.get_context()
    }

    pub fn is_sized(&self) -> bool {
        self.int_type.is_sized()
    }

    pub fn ptr_type(&self, address_space: u32) -> PointerType {
        self.int_type.ptr_type(address_space)
    }
}

impl AsRef<Type> for IntType {
    fn as_ref(&self) -> &Type {
        &self.int_type
    }
}

#[derive(Debug)]
pub struct FloatType {
    float_type: Type,
}

impl FloatType {
    pub(crate) fn new(float_type: LLVMTypeRef) -> Self {
        assert!(!float_type.is_null());

        FloatType {
            float_type: Type::new(float_type),
        }
    }

    pub fn fn_type(&self, param_types: &[&AnyType], is_var_args: bool) -> FunctionType {
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

    pub fn get_context(&self) -> ContextRef {
        self.float_type.get_context()
    }

    pub fn ptr_type(&self, address_space: u32) -> PointerType {
        self.float_type.ptr_type(address_space)
    }
}

impl AsRef<Type> for FloatType {
    fn as_ref(&self) -> &Type {
        &self.float_type
    }
}

#[derive(Debug)]
pub struct StructType {
    struct_type: Type,
}

impl StructType {
    pub(crate) fn new(struct_type: LLVMTypeRef) -> Self {
        assert!(!struct_type.is_null());

        StructType {
            struct_type: Type::new(struct_type),
        }
    }

    // REVIEW: Untested
    // TODO: Would be great to be able to smartly be able to do this by field name
    // TODO: LLVM 3.7+ only
    pub fn get_type_at_field_index(&self, index: u32) -> Option<BasicTypeEnum> {
        // REVIEW: This should only be used on Struct Types, so add a StructType?
        let type_ = unsafe {
            LLVMStructGetTypeAtIndex(self.struct_type.type_, index)
        };

        if type_.is_null() {
            return None;
        }

        Some(BasicTypeEnum::new(type_))
    }

    // REVIEW: Untested
    // TODO: Better name for num. What's it for?
    pub fn const_struct(&self, value: &Value, num: u32) -> StructValue {
        let value = &mut [value.value];

        let val = unsafe {
            LLVMConstNamedStruct(self.struct_type.type_, value.as_mut_ptr(), num)
        };

        StructValue::new(val)
    }

    pub fn is_sized(&self) -> bool {
        self.struct_type.is_sized()
    }

    pub fn get_context(&self) -> ContextRef {
        self.struct_type.get_context()
    }

    pub fn ptr_type(&self, address_space: u32) -> PointerType {
        self.struct_type.ptr_type(address_space)
    }

    pub fn fn_type(&self, param_types: &[&AnyType], is_var_args: bool) -> FunctionType {
        self.struct_type.fn_type(param_types, is_var_args)
    }
}

impl AsRef<Type> for StructType {
    fn as_ref(&self) -> &Type {
        &self.struct_type
    }
}

#[derive(Debug)]
pub struct VoidType {
    void_type: Type,
}

impl VoidType {
    pub(crate) fn new(void_type: LLVMTypeRef) -> Self {
        assert!(!void_type.is_null());

        VoidType {
            void_type: Type::new(void_type),
        }
    }

    pub fn is_sized(&self) -> bool {
        self.void_type.is_sized()
    }

    pub fn get_context(&self) -> ContextRef {
        self.void_type.get_context()
    }

    pub fn ptr_type(&self, address_space: u32) -> PointerType {
        self.void_type.ptr_type(address_space)
    }

    pub fn fn_type(&self, param_types: &[&AnyType], is_var_args: bool) -> FunctionType {
        self.void_type.fn_type(param_types, is_var_args)
    }
}

impl AsRef<Type> for VoidType {
    fn as_ref(&self) -> &Type {
        &self.void_type
    }
}

#[derive(Debug)]
pub struct PointerType {
    ptr_type: Type,
}

impl PointerType {
    pub(crate) fn new(ptr_type: LLVMTypeRef) -> Self {
        assert!(!ptr_type.is_null());

        PointerType {
            ptr_type: Type::new(ptr_type),
        }
    }

    pub fn is_sized(&self) -> bool {
        self.ptr_type.is_sized()
    }

    pub fn ptr_type(&self, address_space: u32) -> PointerType {
        self.ptr_type.ptr_type(address_space)
    }

    pub fn get_context(&self) -> ContextRef {
        self.ptr_type.get_context()
    }

    pub fn fn_type(&self, param_types: &[&AnyType], is_var_args: bool) -> FunctionType {
        self.ptr_type.fn_type(param_types, is_var_args)
    }
}

impl AsRef<Type> for PointerType {
    fn as_ref(&self) -> &Type {
        &self.ptr_type
    }
}

// TODO: VectorType, ArrayType

macro_rules! trait_type_set {
    ($trait_name:ident: $($args:ident),*) => (
        pub trait $trait_name: AsRef<Type> {}

        $(
            impl $trait_name for $args {}
        )*
    );
}

macro_rules! enum_type_set {
    ($enum_name:ident: $($args:ident),*) => (
        #[derive(Debug)]
        pub enum $enum_name {
            $(
                $args($args),
            )*
        }
    );
}

// TODO: Possibly rename to AnyTypeTrait, BasicTypeTrait
trait_type_set! {AnyType: IntType, FunctionType, FloatType, PointerType, StructType, VoidType}
trait_type_set! {BasicType: IntType, FloatType, PointerType, StructType, VoidType}

enum_type_set! {AnyTypeEnum: IntType, FunctionType, FloatType, PointerType, StructType, VoidType}
enum_type_set! {BasicTypeEnum: IntType, FloatType, PointerType, StructType} // ArrayType, VectorType

impl BasicTypeEnum {
    pub(crate) fn new(type_: LLVMTypeRef) -> BasicTypeEnum {
        let type_kind = unsafe {
            LLVMGetTypeKind(type_)
        };

        match type_kind {
            LLVMTypeKind::LLVMHalfTypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMFloatTypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMDoubleTypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMX86_FP80TypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMFP128TypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMPPC_FP128TypeKind => BasicTypeEnum::FloatType(FloatType::new(type_)),
            LLVMTypeKind::LLVMIntegerTypeKind => BasicTypeEnum::IntType(IntType::new(type_)),
            LLVMTypeKind::LLVMStructTypeKind => BasicTypeEnum::StructType(StructType::new(type_)),
            LLVMTypeKind::LLVMPointerTypeKind => BasicTypeEnum::PointerType(PointerType::new(type_)),
            LLVMTypeKind::LLVMArrayTypeKind => panic!("TODO: Unsupported type: Array"),
            LLVMTypeKind::LLVMVectorTypeKind => panic!("TODO: Unsupported type: Vector"),
            _ => unreachable!("Unsupported type"),
        }
    }

    pub fn into_int_type(self) -> IntType {
        if let BasicTypeEnum::IntType(i) = self {
            i
        } else {
            panic!("Called BasicValueEnum.into_int_value on {:?}", self);
        }
    }

    pub fn into_float_type(self) -> FloatType {
        if let BasicTypeEnum::FloatType(f) = self {
            f
        } else {
            panic!("Called BasicValueEnum.into_float_value on {:?}", self);
        }
    }

    pub fn into_ptr_type(self) -> PointerType {
        if let BasicTypeEnum::PointerType(p) = self {
            p
        } else {
            panic!("Called BasicValueEnum.into_ptr_value on {:?}", self);
        }
    }

    pub fn into_struct_type(self) -> StructType {
        if let BasicTypeEnum::StructType(s) = self {
            s
        } else {
            panic!("Called BasicValueEnum.into_struct_value on {:?}", self);
        }
    }

    pub fn as_int_type(&self) -> &IntType {
        if let &BasicTypeEnum::IntType(ref i) = self {
            &i
        } else {
            panic!("Called BasicValueEnum.as_int_value on {:?}", self);
        }
    }

    pub fn as_float_type(&self) -> &FloatType {
        if let &BasicTypeEnum::FloatType(ref f) = self {
            &f
        } else {
            panic!("Called BasicValueEnum.as_float_value on {:?}", self);
        }
    }

    pub fn as_ptr_type(&self) -> &PointerType {
        if let &BasicTypeEnum::PointerType(ref p) = self {
            &p
        } else {
            panic!("Called BasicValueEnum.as_ptr_value on {:?}", self);
        }
    }

    pub fn as_struct_type(&self) -> &StructType {
        if let &BasicTypeEnum::StructType(ref s) = self {
            &s
        } else {
            panic!("Called BasicValueEnum.as_struct_value on {:?}", self);
        }
    }
}

// REVIEW: Possible to impl Debug for AnyType?

#[test]
fn test_function_type() {
    let context = Context::create();
    let int = context.i8_type();
    let float = context.f32_type();
    let fn_type = int.fn_type(&[&int, &int, &float], false);

    assert!(!fn_type.is_var_arg());

    let param_types = fn_type.get_param_types();

    assert_eq!(param_types.len(), 3);
    assert_eq!(param_types[0].as_int_type().as_ref().type_, int.as_ref().type_);
    assert_eq!(param_types[1].as_int_type().as_ref().type_, int.as_ref().type_);
    assert_eq!(param_types[2].as_float_type().as_ref().type_, float.as_ref().type_);

    let fn_type = int.fn_type(&[&int, &float], true);

    assert!(fn_type.is_var_arg());
}
