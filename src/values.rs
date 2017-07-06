use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction};
use llvm_sys::core::{LLVMAddIncoming, LLVMCountParams, LLVMGetBasicBlocks, LLVMGetElementType, LLVMGetFirstBasicBlock, LLVMGetFirstParam, LLVMGetLastBasicBlock, LLVMGetNextParam, LLVMGetParam, LLVMGetReturnType, LLVMGetValueName, LLVMIsAConstantArray, LLVMIsAConstantDataArray, LLVMIsAFunction, LLVMIsConstant, LLVMIsNull, LLVMIsUndef, LLVMPrintTypeToString, LLVMPrintValueToString, LLVMSetGlobalConstant, LLVMSetValueName, LLVMTypeOf, LLVMGetTypeKind};
use llvm_sys::LLVMTypeKind;
use llvm_sys::prelude::LLVMValueRef;

use std::ffi::{CString, CStr};
use std::fmt;
use std::mem::transmute;

use basic_block::BasicBlock;
use types::{AnyType, IntType, StructType, FloatType, PointerType, FunctionType, VoidType, Type};

pub struct Value {
    pub(crate) value: LLVMValueRef,
}

impl Value {
    pub(crate) fn new(value: LLVMValueRef) -> Value {
        assert!(!value.is_null());

        Value {
            value: value
        }
    }

    fn set_global_constant(&self, num: i32) { // REVIEW: Need better name for this arg
        unsafe {
            LLVMSetGlobalConstant(self.value, num)
        }
    }

    fn set_name(&self, name: &str) {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetValueName(self.value, c_string.as_ptr());
        }
    }

    fn get_name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetValueName(self.value))
        }
    }

    // REVIEW: Untested
    // REVIEW: Is incoming_values really ArrayValue? Or an &[AnyValue]?
    fn add_incoming(&self, incoming_values: &AnyValue, incoming_basic_block: &BasicBlock, count: u32) {
        let value = &mut [incoming_values.as_ref().value];
        let basic_block = &mut [incoming_basic_block.basic_block];

        unsafe {
            LLVMAddIncoming(self.value, value.as_mut_ptr(), basic_block.as_mut_ptr(), count);
        }
    }

    // REVIEW: Untested
    fn is_undef(&self) -> bool {
        unsafe {
            LLVMIsUndef(self.value) == 1
        }
    }

    // TODO: impl AnyType when it stabilizes
    fn get_type(&self) -> Box<AnyType> {
        let type_ = unsafe {
            LLVMTypeOf(self.value)
        };
        let type_kind = unsafe {
            LLVMGetTypeKind(type_)
        };

        match type_kind {
            LLVMTypeKind::LLVMVoidTypeKind => Box::new(VoidType::new(type_)),
            LLVMTypeKind::LLVMHalfTypeKind => Box::new(FloatType::new(type_)),
            LLVMTypeKind::LLVMFloatTypeKind => Box::new(FloatType::new(type_)),
            LLVMTypeKind::LLVMDoubleTypeKind => Box::new(FloatType::new(type_)),
            LLVMTypeKind::LLVMX86_FP80TypeKind => Box::new(FloatType::new(type_)),
            LLVMTypeKind::LLVMFP128TypeKind => Box::new(FloatType::new(type_)),
            LLVMTypeKind::LLVMPPC_FP128TypeKind => Box::new(FloatType::new(type_)),
            LLVMTypeKind::LLVMLabelTypeKind => panic!("FIXME: Unsupported type: Label"),
            LLVMTypeKind::LLVMIntegerTypeKind => Box::new(IntType::new(type_)),
            LLVMTypeKind::LLVMFunctionTypeKind => Box::new(FunctionType::new(type_)),
            LLVMTypeKind::LLVMStructTypeKind => Box::new(StructType::new(type_)),
            LLVMTypeKind::LLVMArrayTypeKind => panic!("FIXME: Unsupported type: Array"),
            LLVMTypeKind::LLVMPointerTypeKind => Box::new(PointerType::new(type_)),
            LLVMTypeKind::LLVMVectorTypeKind => panic!("FIXME: Unsupported type: Vector"),
            LLVMTypeKind::LLVMMetadataTypeKind => panic!("FIXME: Unsupported type: Metadata"),
            LLVMTypeKind::LLVMX86_MMXTypeKind => panic!("FIXME: Unsupported type: MMX"),
            // LLVMTypeKind::LLVMTokenTypeKind => panic!("FIXME: Unsupported type: Token"), // Different version?
        }
    }

    // REVIEW: Remove?
    fn get_type_kind(&self) -> LLVMTypeKind {
        (*self.get_type()).as_ref().get_kind()
    }

    fn is_pointer(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMPointerTypeKind => true,
            _ => false,
        }
    }

    fn is_int(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMIntegerTypeKind => true,
            _ => false,
        }
    }

    fn is_f32(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMFloatTypeKind => true,
            _ => false,
        }
    }

    fn is_f64(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMDoubleTypeKind => true,
            _ => false,
        }
    }

    fn is_f128(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMFP128TypeKind => true,
            _ => false,
        }
    }

    fn is_float(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMHalfTypeKind |
            LLVMTypeKind::LLVMFloatTypeKind |
            LLVMTypeKind::LLVMDoubleTypeKind |
            LLVMTypeKind::LLVMX86_FP80TypeKind |
            LLVMTypeKind::LLVMFP128TypeKind |
            LLVMTypeKind::LLVMPPC_FP128TypeKind => true,
            _ => false,
        }
    }

    fn is_struct(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMStructTypeKind => true,
            _ => false,
        }
    }

    fn is_array(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMArrayTypeKind => true,
            _ => false,
        }
    }

    fn is_void(&self) -> bool {
        match self.get_type_kind() {
            LLVMTypeKind::LLVMVoidTypeKind => true,
            _ => false,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.value))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.value) == 1
        };
        let is_null = unsafe {
            LLVMIsNull(self.value) == 1
        };
        let is_const_array = unsafe {
            !LLVMIsAConstantArray(self.value).is_null()
        };
        let is_const_data_array = unsafe {
            !LLVMIsAConstantDataArray(self.value).is_null()
        };

        write!(f, "Value {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_const_array: {:?}\n    is_const_data_array: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.value, is_const, is_const_array, is_const_data_array, is_null, llvm_value, llvm_type)
    }
}

pub struct FunctionValue {
    pub(crate) fn_value: Value,
}

impl FunctionValue {
    pub(crate) fn new(value: LLVMValueRef) -> FunctionValue {
        assert!(!value.is_null());

        unsafe {
            assert!(!LLVMIsAFunction(value).is_null())
        }

        FunctionValue {
            fn_value: Value::new(value)
        }
    }

    pub fn verify(&self, print: bool) {
        let action = if print {
            LLVMVerifierFailureAction::LLVMPrintMessageAction
        } else {
            LLVMVerifierFailureAction::LLVMReturnStatusAction
        };

        let code = unsafe {
            LLVMVerifyFunction(self.fn_value.value, action)
        };

        if code == 1 {
            panic!("LLVMGenError")
        }
    }

    pub fn get_first_param(&self) -> Option<ParamValue> {
        let param = unsafe {
            LLVMGetFirstParam(self.fn_value.value)
        };

        if param.is_null() {
            return None;
        }

        Some(ParamValue::new(param))
    }

    pub fn get_first_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetFirstBasicBlock(self.fn_value.value)
        };

        if bb.is_null() {
            return None;
        }

        Some(BasicBlock::new(bb))
    }

    pub fn get_nth_param(&self, nth: u32) -> Option<ParamValue> {
        let count = self.count_params();

        if nth + 1 > count {
            return None;
        }

        let param = unsafe {
            LLVMGetParam(self.fn_value.value, nth)
        };

        Some(ParamValue::new(param))
    }

    pub fn count_params(&self) -> u32 {
        unsafe {
            LLVMCountParams(self.fn_value.value)
        }
    }

    /// REVIEW: Untested
    pub fn get_basic_blocks(&self) -> Vec<BasicBlock> {
        let mut blocks = vec![];

        unsafe {
            LLVMGetBasicBlocks(self.fn_value.value, blocks.as_mut_ptr());

            transmute(blocks)
        }
    }

    pub fn get_return_type(&self) -> Type {
        let type_ = unsafe {
            LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(self.fn_value.value)))
        };

        Type::new(type_)
    }

    pub fn params(&self) -> ParamValueIter {
        ParamValueIter {
            param_iter_value: self.fn_value.value,
            start: true,
        }
    }

    pub fn get_last_basic_block(&self) -> BasicBlock {
        let bb = unsafe {
            LLVMGetLastBasicBlock(self.fn_value.value)
        };

        BasicBlock::new(bb)
    }

    pub fn get_name(&self) -> &CStr {
        self.fn_value.get_name()
    }
}

impl AsRef<Value> for FunctionValue {
    fn as_ref(&self) -> &Value {
        &self.fn_value
    }
}

impl fmt::Debug for FunctionValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.fn_value.value))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.fn_value.value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.fn_value.value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.fn_value.value) == 1
        };
        let is_null = unsafe {
            LLVMIsNull(self.fn_value.value) == 1
        };

        write!(f, "FunctionValue {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.fn_value, is_const, is_null, llvm_value, llvm_type)
    }
}

pub struct ParamValue {
    param_value: Value,
}

impl ParamValue {
    pub(crate) fn new(param_value: LLVMValueRef) -> ParamValue {
        assert!(!param_value.is_null());

        ParamValue {
            param_value: Value::new(param_value)
        }
    }

    pub fn set_name(&self, name: &str) {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        unsafe {
            LLVMSetValueName(self.param_value.value, c_string.as_ptr())
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.param_value.get_name()
    }
}

impl fmt::Debug for ParamValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.param_value.value))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.param_value.value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.param_value.value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.param_value.value) == 1
        };
        let is_null = unsafe {
            LLVMIsNull(self.param_value.value) == 1
        };

        write!(f, "FunctionValue {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.param_value, is_const, is_null, llvm_value, llvm_type)
    }
}

pub struct ParamValueIter {
    param_iter_value: LLVMValueRef,
    start: bool,
}

impl Iterator for ParamValueIter {
    type Item = ParamValue;

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

            return Some(ParamValue::new(first_value));
        }

        let next_value = unsafe {
            LLVMGetNextParam(self.param_iter_value)
        };

        if next_value.is_null() {
            return None;
        }

        self.param_iter_value = next_value;

        Some(ParamValue::new(next_value))
    }
}

impl AsRef<Value> for ParamValue {
    fn as_ref(&self) -> &Value {
        &self.param_value
    }
}

pub struct IntValue {
    pub(crate) int_value: Value,
}

impl IntValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        IntValue {
            int_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.int_value.get_name()
    }
}

impl AsRef<Value> for IntValue {
    fn as_ref(&self) -> &Value {
        &self.int_value
    }
}

pub trait IntoIntValue {
    fn into_int_value(&self) -> IntValue;
}

impl IntoIntValue for IntValue {
    fn into_int_value(&self) -> IntValue {
        IntValue::new(self.int_value.value)
    }
}

impl IntoIntValue for u64 {
    fn into_int_value(&self) -> IntValue {
        IntType::i32_type().const_int(*self, false)
    }
}

pub struct FloatValue {
    pub(crate) float_value: Value
}

impl FloatValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        FloatValue {
            float_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.float_value.get_name()
    }
}

impl AsRef<Value> for FloatValue {
    fn as_ref(&self) -> &Value {
        &self.float_value
    }
}

pub struct StructValue {
    struct_value: Value
}

impl StructValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        StructValue {
            struct_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.struct_value.get_name()
    }
}

impl AsRef<Value> for StructValue {
    fn as_ref(&self) -> &Value {
        &self.struct_value
    }
}

pub struct PointerValue {
    pub(crate) ptr_value: Value
}

impl PointerValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        PointerValue {
            ptr_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.ptr_value.get_name()
    }
}

impl AsRef<Value> for PointerValue {
    fn as_ref(&self) -> &Value {
        &self.ptr_value
    }
}

pub struct PhiValue {
    phi_value: Value
}

impl PhiValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        PhiValue {
            phi_value: Value::new(value),
        }
    }

    pub fn add_incoming(&self, incoming_values: &AnyValue, incoming_basic_block: &BasicBlock, count: u32) {
        self.phi_value.add_incoming(incoming_values, incoming_basic_block, count)
    }

    pub fn get_name(&self) -> &CStr {
        self.phi_value.get_name()
    }
}

impl AsRef<Value> for PhiValue {
    fn as_ref(&self) -> &Value {
        &self.phi_value
    }
}

impl AsRef<Value> for Value { // TODO: Remove
    fn as_ref(&self) -> &Value {
        self
    }
}

macro_rules! value_set {
    ($trait_name:ident: $($args:ident),*) => (
        pub trait $trait_name: AsRef<Value> {}

        $(
            impl $trait_name for $args {}
        )*
    );
}

value_set! {AnyValue: IntValue, FloatValue, PhiValue, ParamValue, FunctionValue, StructValue, Value} // TODO: Remove Value, ParamValue?
value_set! {BasicValue: IntValue, FloatValue, StructValue, ParamValue} // TODO: Remove ParamValue?

// Case for separate Value structs:
// LLVMValueRef can be a value (ie int)
// LLVMValueRef can be a function
// LLVMValueRef can be a function param
// LLVMValueRef can be a comparison_op
