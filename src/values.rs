use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMViewFunctionCFG, LLVMViewFunctionCFGOnly};
use llvm_sys::core::{LLVMAddIncoming, LLVMCountParams, LLVMGetBasicBlocks, LLVMGetElementType, LLVMGetFirstBasicBlock, LLVMGetFirstParam, LLVMGetLastBasicBlock, LLVMGetNextParam, LLVMGetParam, LLVMGetReturnType, LLVMGetValueName, LLVMIsAConstantArray, LLVMIsAConstantDataArray, LLVMIsAFunction, LLVMIsConstant, LLVMIsNull, LLVMIsUndef, LLVMPrintTypeToString, LLVMPrintValueToString, LLVMSetGlobalConstant, LLVMSetValueName, LLVMTypeOf, LLVMGetTypeKind, LLVMGetNextFunction, LLVMGetPreviousFunction, LLVMIsAConstantVector, LLVMIsAConstantDataVector, LLVMDumpValue, LLVMCountBasicBlocks, LLVMIsAInstruction, LLVMGetInstructionOpcode, LLVMGetLinkage, LLVMDeleteFunction, LLVMGetLastParam, LLVMGetEntryBasicBlock, LLVMAppendBasicBlock, LLVMConstNeg, LLVMConstFNeg, LLVMConstFAdd, LLVMConstAdd, LLVMConstSub, LLVMConstFSub, LLVMConstMul, LLVMConstFMul, LLVMConstFDiv, LLVMConstNot, LLVMConstNSWAdd, LLVMConstNUWAdd, LLVMConstNUWSub, LLVMConstNSWSub, LLVMConstNUWMul, LLVMConstNSWMul, LLVMConstUDiv, LLVMConstSDiv, LLVMConstExactSDiv, LLVMConstURem, LLVMConstSRem, LLVMConstFRem, LLVMConstAnd, LLVMConstOr, LLVMConstXor};
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMValueRef};
use llvm_sys::{LLVMOpcode, LLVMTypeKind};

use std::ffi::{CString, CStr};
use std::fmt;
use std::mem::forget;

use basic_block::BasicBlock;
use module::Linkage;
use types::{AnyTypeEnum, ArrayType, BasicTypeEnum, PointerType, FloatType, IntType, StructType, VectorType};

mod private {
    // This is an ugly privacy hack so that Type can stay private to this module
    // and so that super traits using this trait will be not be implementable
    // outside this library
    use llvm_sys::prelude::LLVMValueRef;

    pub trait AsValueRef {
        fn as_value_ref(&self) -> LLVMValueRef;
    }
}

pub(crate) use self::private::AsValueRef;

struct Value {
    value: LLVMValueRef,
}

impl Value {
    pub(crate) fn new(value: LLVMValueRef) -> Value {
        assert!(!value.is_null());

        Value {
            value: value
        }
    }

    fn is_instruction(&self) -> bool {
        unsafe {
            !LLVMIsAInstruction(self.value).is_null()
        }
    }

    fn as_instruction(&self) -> Option<InstructionValue> {
        if !self.is_instruction() {
            return None;
        }

        Some(InstructionValue::new(self.value))
    }

    fn is_null(&self) -> bool {
        unsafe {
            LLVMIsNull(self.value) == 1
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

    // REVIEW: Is incoming_values really ArrayValue? Or an &[AnyValue]?
    fn add_incoming(&self, incoming_values: &AnyValue, incoming_basic_block: &BasicBlock, count: u32) {
        let value = &mut [incoming_values.as_value_ref()];
        let basic_block = &mut [incoming_basic_block.basic_block];

        unsafe {
            LLVMAddIncoming(self.value, value.as_mut_ptr(), basic_block.as_mut_ptr(), count);
        }
    }

    fn is_undef(&self) -> bool {
        unsafe {
            LLVMIsUndef(self.value) == 1
        }
    }

    fn get_type(&self) -> AnyTypeEnum {
        let type_ = unsafe {
            LLVMTypeOf(self.value)
        };

        AnyTypeEnum::new(type_)
    }

    fn print_to_string(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.value))
        }
    }

    fn print_to_stderr(&self) {
        unsafe {
            LLVMDumpValue(self.value)
        }
    }

    // REVIEW: Remove?
    // fn get_type_kind(&self) -> LLVMTypeKind {
    //     (*self.get_type()).as_llvm_type_ref().get_kind()
    // }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = self.print_to_string();
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.value) == 1
        };
        let is_null = self.is_null();
        let is_undef = self.is_undef();

        write!(f, "Value {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_null: {:?}\n    is_undef: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.value, is_const, is_null, is_undef, llvm_value, llvm_type)
    }
}

pub struct FunctionValue {
    fn_value: Value,
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

    pub fn get_linkage(&self) -> Linkage {
        let linkage = unsafe {
            LLVMGetLinkage(self.as_value_ref())
        };

        Linkage::new(linkage)
    }

    pub fn is_null(&self) -> bool {
        self.fn_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.fn_value.is_undef()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.fn_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.fn_value.print_to_stderr()
    }

    // TODO: Maybe support LLVMAbortProcessAction?
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

    // If there's a demand, could easily create a module.get_functions() -> Iterator
    pub fn get_next_function(&self) -> Option<Self> {
        let function = unsafe {
            LLVMGetNextFunction(self.as_value_ref())
        };

        if function.is_null() {
            return None;
        }

        Some(FunctionValue::new(function))
    }

    pub fn get_previous_function(&self) -> Option<Self> {
        let function = unsafe {
            LLVMGetPreviousFunction(self.as_value_ref())
        };

        if function.is_null() {
            return None;
        }

        Some(FunctionValue::new(function))
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

    pub fn get_entry_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetEntryBasicBlock(self.as_value_ref())
        };

        if bb.is_null() {
            return None;
        }

        Some(BasicBlock::new(bb))
    }

    pub fn get_first_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetFirstBasicBlock(self.as_value_ref())
        };

        if bb.is_null() {
            return None;
        }

        Some(BasicBlock::new(bb))
    }

    pub fn append_basic_block(&self, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMAppendBasicBlock(self.as_value_ref(), c_string.as_ptr())
        };

        BasicBlock::new(bb)
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

        raw_vec.iter().map(|val| BasicBlock::new(*val)).collect()
    }

    pub fn get_return_type(&self) -> BasicTypeEnum {
        let type_ = unsafe {
            LLVMGetReturnType(LLVMGetElementType(LLVMTypeOf(self.fn_value.value)))
        };

        BasicTypeEnum::new(type_)
    }

    pub fn params(&self) -> ParamValueIter {
        ParamValueIter {
            param_iter_value: self.fn_value.value,
            start: true,
        }
    }

    pub fn get_last_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetLastBasicBlock(self.fn_value.value)
        };

        if bb.is_null() {
            return None;
        }

        Some(BasicBlock::new(bb))
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

    // TODO: Look for ways to use after delete
    pub fn delete(self) {
        unsafe {
            LLVMDeleteFunction(self.as_value_ref())
        }
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
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.fn_value.value)))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.fn_value.value))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.fn_value.value) == 1
        };
        let is_null = self.is_null();

        write!(f, "FunctionValue {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.fn_value, is_const, is_null, llvm_value, llvm_type)
    }
}

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

            return Some(BasicValueEnum::new(first_value));
        }

        let next_value = unsafe {
            LLVMGetNextParam(self.param_iter_value)
        };

        if next_value.is_null() {
            return None;
        }

        self.param_iter_value = next_value;

        Some(BasicValueEnum::new(next_value))
    }
}

#[derive(Debug)]
pub struct IntValue {
    int_value: Value,
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

    pub fn set_name(&self, name: &str) {
        self.int_value.set_name(name);
    }

    pub fn get_type(&self) -> IntType {
        let int_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        IntType::new(int_type)
    }

    pub fn is_null(&self) -> bool {
        self.int_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.int_value.is_undef()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.int_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.int_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.int_value.as_instruction()
    }

    pub fn const_not(&self) -> Self {
        let value = unsafe {
            LLVMConstNot(self.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_neg(&self) -> Self {
        let value = unsafe {
            LLVMConstNeg(self.as_value_ref())
        };

        IntValue::new(value)
    }

    // TODO: operator overloading to call this
    pub fn const_add(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstAdd(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nsw_add(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstNSWAdd(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nuw_add(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstNUWAdd(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    // TODO: operator overloading to call this
    pub fn const_sub(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstSub(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nsw_sub(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstNSWSub(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nuw_sub(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstNUWSub(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    // TODO: operator overloading to call this
    pub fn const_mul(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstMul(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nsw_mul(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstNSWMul(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_nuw_mul(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstNUWMul(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_unsigned_div(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstUDiv(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_signed_div(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstSDiv(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_exact_signed_div(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstExactSDiv(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_unsigned_remainder(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstURem(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_signed_remainder(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstSRem(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_and(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstAnd(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_or(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstOr(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }

    pub fn const_xor(&self, rhs: &IntValue) -> Self {
        let value = unsafe {
            LLVMConstXor(self.as_value_ref(), rhs.as_value_ref())
        };

        IntValue::new(value)
    }
}

impl AsValueRef for IntValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.int_value.value
    }
}

#[derive(Debug)]
pub struct FloatValue {
    float_value: Value
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

    pub fn set_name(&self, name: &str) {
        self.float_value.set_name(name);
    }

    pub fn get_type(&self) -> FloatType {
        let float_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        FloatType::new(float_type)
    }

    pub fn is_null(&self) -> bool {
        self.float_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.float_value.is_undef()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.float_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.float_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.float_value.as_instruction()
    }

    pub fn const_neg(&self) -> Self {
        let value = unsafe {
            LLVMConstFNeg(self.as_value_ref())
        };

        FloatValue::new(value)
    }

    // TODO: operator overloading to call this
    pub fn const_add(&self, rhs: &FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFAdd(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    // TODO: operator overloading to call this
    pub fn const_sub(&self, rhs: &FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFSub(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    // TODO: operator overloading to call this
    pub fn const_mul(&self, rhs: &FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFMul(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    // TODO: operator overloading to call this
    pub fn const_div(&self, rhs: &FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFDiv(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }

    pub fn const_remainder(&self, rhs: &FloatValue) -> Self {
        let value = unsafe {
            LLVMConstFRem(self.as_value_ref(), rhs.as_value_ref())
        };

        FloatValue::new(value)
    }
}

impl AsValueRef for FloatValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.float_value.value
    }
}

#[derive(Debug)]
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

    pub fn set_name(&self, name: &str) {
        self.struct_value.set_name(name);
    }

    pub fn get_type(&self) -> StructType {
        let struct_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        StructType::new(struct_type)
    }

    pub fn is_null(&self) -> bool {
        self.struct_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.struct_value.is_undef()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.struct_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.struct_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.struct_value.as_instruction()
    }
}

impl AsValueRef for StructValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.struct_value.value
    }
}

#[derive(Debug)]
pub struct PointerValue {
    ptr_value: Value
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

    pub fn set_name(&self, name: &str) {
        self.ptr_value.set_name(name);
    }

    pub fn get_type(&self) -> PointerType {
        let pointer_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        PointerType::new(pointer_type)
    }

    pub fn is_null(&self) -> bool {
        self.ptr_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.ptr_value.is_undef()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.ptr_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.ptr_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.ptr_value.as_instruction()
    }
}

impl AsValueRef for PointerValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.ptr_value.value
    }
}

#[derive(Debug)]
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

    pub fn is_null(&self) -> bool {
        self.phi_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.phi_value.is_undef()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.phi_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.phi_value.print_to_stderr()
    }

    // REVIEW: Maybe this is should always return InstructionValue?
    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.phi_value.as_instruction()
    }
}

impl AsValueRef for PhiValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.phi_value.value
    }
}

impl AsValueRef for Value { // TODO: Remove
    fn as_value_ref(&self) -> LLVMValueRef {
        self.value
    }
}

pub struct ArrayValue {
    array_value: Value
}

impl ArrayValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        ArrayValue {
            array_value: Value::new(value),
        }
    }

    pub fn get_name(&self) -> &CStr {
        self.array_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.array_value.set_name(name);
    }

    pub fn get_type(&self) -> ArrayType {
        let array_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        ArrayType::new(array_type)
    }

    pub fn is_null(&self) -> bool {
        self.array_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.array_value.is_undef()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.array_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.array_value.print_to_stderr()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.array_value.as_instruction()
    }
}

impl AsValueRef for ArrayValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.array_value.value
    }
}

impl fmt::Debug for ArrayValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = self.print_to_string();
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.as_value_ref())))
        };
        let name = unsafe {
            CStr::from_ptr(LLVMGetValueName(self.as_value_ref()))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.as_value_ref()) == 1
        };
        let is_null = self.is_null();
        let is_const_array = unsafe {
            !LLVMIsAConstantArray(self.as_value_ref()).is_null()
        };
        let is_const_data_array = unsafe {
            !LLVMIsAConstantDataArray(self.as_value_ref()).is_null()
        };

        write!(f, "Value {{\n    name: {:?}\n    address: {:?}\n    is_const: {:?}\n    is_const_array: {:?}\n    is_const_data_array: {:?}\n    is_null: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", name, self.as_value_ref(), is_const, is_const_array, is_const_data_array, is_null, llvm_value, llvm_type)
    }
}

#[derive(Debug)]
pub struct VectorValue {
    vec_value: Value,
}

impl VectorValue {
    pub(crate) fn new(vector_value: LLVMValueRef) -> Self {
        assert!(!vector_value.is_null());

        VectorValue {
            vec_value: Value::new(vector_value)
        }
    }

    // REVIEW: Should this be !int_value.is_null() to return bool?
    pub fn is_constant_vector(&self) -> IntValue { // TSv2: IntValue<bool>
        let int_value = unsafe {
            LLVMIsAConstantVector(self.as_value_ref())
        };

        IntValue::new(int_value)
    }

    // REVIEW: Should this be !int_value.is_null() to return bool?
    pub fn is_constant_data_vector(&self) -> IntValue { // TSv2: IntValue<bool>
        let int_value = unsafe {
            LLVMIsAConstantDataVector(self.as_value_ref())
        };

        IntValue::new(int_value)
    }

    pub fn print_to_string(&self) -> &CStr {
        self.vec_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.vec_value.print_to_stderr()
    }

    pub fn get_name(&self) -> &CStr {
        self.vec_value.get_name()
    }

    pub fn set_name(&self, name: &str) {
        self.vec_value.set_name(name);
    }

    pub fn get_type(&self) -> VectorType {
        let vec_type = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        VectorType::new(vec_type)
    }

    pub fn is_null(&self) -> bool {
        self.vec_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.vec_value.is_undef()
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        self.vec_value.as_instruction()
    }
}

impl AsValueRef for VectorValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.vec_value.value
    }
}

// REVIEW: Maybe this should go into it's own opcode module?
#[derive(Debug, PartialEq)]
pub enum InstructionOpcode {
    Add,
    AddrSpaceCast,
    Alloca,
    And,
    AShr,
    AtomicCmpXchg,
    AtomicRMW,
    BitCast,
    Br,
    Call,
    // Later versions:
    // CatchPad,
    // CatchRet,
    // CatchSwitch,
    // CleanupPad,
    // CleanupRet,
    ExtractElement,
    ExtractValue,
    FAdd,
    FCmp,
    FDiv,
    Fence,
    FMul,
    FPExt,
    FPToSI,
    FPToUI,
    FPTrunc,
    FRem,
    FSub,
    GetElementPtr,
    ICmp,
    IndirectBr,
    InsertElement,
    InsertValue,
    IntToPtr,
    Invoke,
    LandingPad,
    Load,
    LShr,
    Mul,
    Or,
    PHI,
    PtrToInt,
    Resume,
    Return,
    SDiv,
    Select,
    SExt,
    Shl,
    ShuffleVector,
    SIToFP,
    SRem,
    Store,
    Sub,
    Switch,
    Trunc,
    UDiv,
    UIToFP,
    Unreachable,
    URem,
    UserOp1,
    UserOp2,
    VAArg,
    Xor,
    ZExt,
}

impl InstructionOpcode {
    fn new(opcode: LLVMOpcode) -> Self {
        match opcode {
            LLVMOpcode::LLVMAdd => InstructionOpcode::Add,
            LLVMOpcode::LLVMAddrSpaceCast => InstructionOpcode::AddrSpaceCast,
            LLVMOpcode::LLVMAlloca => InstructionOpcode::Alloca,
            LLVMOpcode::LLVMAnd => InstructionOpcode::And,
            LLVMOpcode::LLVMAShr => InstructionOpcode::AShr,
            LLVMOpcode::LLVMAtomicCmpXchg => InstructionOpcode::AtomicCmpXchg,
            LLVMOpcode::LLVMAtomicRMW => InstructionOpcode::AtomicRMW,
            LLVMOpcode::LLVMBitCast => InstructionOpcode::BitCast,
            LLVMOpcode::LLVMBr => InstructionOpcode::Br,
            LLVMOpcode::LLVMCall => InstructionOpcode::Call,
            // Newer versions:
            // LLVMOpcode::LLVMCatchPad => InstructionOpcode::CatchPad,
            // LLVMOpcode::LLVMCatchRet => InstructionOpcode::CatchRet,
            // LLVMOpcode::LLVMCatchSwitch => InstructionOpcode::CatchSwitch,
            // LLVMOpcode::LLVMCleanupPad => InstructionOpcode::CleanupPad,
            // LLVMOpcode::LLVMCleanupRet => InstructionOpcode::CleanupRet,
            LLVMOpcode::LLVMExtractElement => InstructionOpcode::ExtractElement,
            LLVMOpcode::LLVMExtractValue => InstructionOpcode::ExtractValue,
            LLVMOpcode::LLVMFAdd => InstructionOpcode::FAdd,
            LLVMOpcode::LLVMFCmp => InstructionOpcode::FCmp,
            LLVMOpcode::LLVMFDiv => InstructionOpcode::FDiv,
            LLVMOpcode::LLVMFence => InstructionOpcode::Fence,
            LLVMOpcode::LLVMFMul => InstructionOpcode::FMul,
            LLVMOpcode::LLVMFPExt => InstructionOpcode::FPExt,
            LLVMOpcode::LLVMFPToSI => InstructionOpcode::FPToSI,
            LLVMOpcode::LLVMFPToUI => InstructionOpcode::FPToUI,
            LLVMOpcode::LLVMFPTrunc => InstructionOpcode::FPTrunc,
            LLVMOpcode::LLVMFRem => InstructionOpcode::FRem,
            LLVMOpcode::LLVMFSub => InstructionOpcode::FSub,
            LLVMOpcode::LLVMGetElementPtr => InstructionOpcode::GetElementPtr,
            LLVMOpcode::LLVMICmp => InstructionOpcode::ICmp,
            LLVMOpcode::LLVMIndirectBr => InstructionOpcode::IndirectBr,
            LLVMOpcode::LLVMInsertElement => InstructionOpcode::InsertElement,
            LLVMOpcode::LLVMInsertValue => InstructionOpcode::InsertValue,
            LLVMOpcode::LLVMIntToPtr => InstructionOpcode::IntToPtr,
            LLVMOpcode::LLVMInvoke => InstructionOpcode::Invoke,
            LLVMOpcode::LLVMLandingPad => InstructionOpcode::LandingPad,
            LLVMOpcode::LLVMLoad => InstructionOpcode::Load,
            LLVMOpcode::LLVMLShr => InstructionOpcode::LShr,
            LLVMOpcode::LLVMMul => InstructionOpcode::Mul,
            LLVMOpcode::LLVMOr => InstructionOpcode::Or,
            LLVMOpcode::LLVMPHI => InstructionOpcode::PHI,
            LLVMOpcode::LLVMPtrToInt => InstructionOpcode::PtrToInt,
            LLVMOpcode::LLVMResume => InstructionOpcode::Resume,
            LLVMOpcode::LLVMRet => InstructionOpcode::Return,
            LLVMOpcode::LLVMSDiv => InstructionOpcode::SDiv,
            LLVMOpcode::LLVMSelect => InstructionOpcode::Select,
            LLVMOpcode::LLVMSExt => InstructionOpcode::SExt,
            LLVMOpcode::LLVMShl => InstructionOpcode::Shl,
            LLVMOpcode::LLVMShuffleVector => InstructionOpcode::ShuffleVector,
            LLVMOpcode::LLVMSIToFP => InstructionOpcode::SIToFP,
            LLVMOpcode::LLVMSRem => InstructionOpcode::SRem,
            LLVMOpcode::LLVMStore => InstructionOpcode::Store,
            LLVMOpcode::LLVMSub => InstructionOpcode::Sub,
            LLVMOpcode::LLVMSwitch => InstructionOpcode::Switch,
            LLVMOpcode::LLVMTrunc => InstructionOpcode::Trunc,
            LLVMOpcode::LLVMUDiv => InstructionOpcode::UDiv,
            LLVMOpcode::LLVMUIToFP => InstructionOpcode::UIToFP,
            LLVMOpcode::LLVMUnreachable => InstructionOpcode::Unreachable,
            LLVMOpcode::LLVMURem => InstructionOpcode::URem,
            LLVMOpcode::LLVMUserOp1 => InstructionOpcode::UserOp1,
            LLVMOpcode::LLVMUserOp2 => InstructionOpcode::UserOp2,
            LLVMOpcode::LLVMVAArg => InstructionOpcode::VAArg,
            LLVMOpcode::LLVMXor => InstructionOpcode::Xor,
            LLVMOpcode::LLVMZExt => InstructionOpcode::ZExt,
        }
    }
}

#[derive(Debug)]
pub struct InstructionValue {
    instruction_value: Value,
}

impl InstructionValue {
    pub(crate) fn new(instruction_value: LLVMValueRef) -> Self {
        assert!(!instruction_value.is_null());

        let value = Value::new(instruction_value);

        assert!(value.is_instruction());

        InstructionValue {
            instruction_value: value,
        }
    }

    pub fn get_opcode(&self) -> InstructionOpcode {
        let opcode = unsafe {
            LLVMGetInstructionOpcode(self.as_value_ref())
        };

        InstructionOpcode::new(opcode)
    }
}

impl AsValueRef for InstructionValue {
    fn as_value_ref(&self) -> LLVMValueRef {
        self.instruction_value.value
    }
}

macro_rules! trait_value_set {
    ($trait_name:ident: $($args:ident),*) => (
        pub trait $trait_name: AsValueRef {}

        $(
            impl $trait_name for $args {}
        )*

        // REVIEW: Possible encompassing methods to implement:
        // as_instruction, is_sized
    );
}

macro_rules! enum_value_set {
    ($enum_name:ident: $($args:ident),*) => (
        #[derive(Debug, EnumAsGetters, EnumIntoGetters, EnumIsA)]
        pub enum $enum_name {
            $(
                $args($args),
            )*
        }

        impl AsValueRef for $enum_name {
            fn as_value_ref(&self) -> LLVMValueRef {
                match *self {
                    $(
                        $enum_name::$args(ref t) => t.as_value_ref(),
                    )*
                }
            }
        }

        $(
            impl From<$args> for $enum_name {
                fn from(value: $args) -> $enum_name {
                    $enum_name::$args(value)
                }
            }
        )*

        // REVIEW: Possible encompassing methods to implement:
        // as_instruction, is_sized
    );
}

enum_value_set! {AggregateValueEnum: ArrayValue, StructValue}
enum_value_set! {AnyValueEnum: ArrayValue, IntValue, FloatValue, PhiValue, FunctionValue, PointerValue, StructValue, VectorValue}
enum_value_set! {BasicValueEnum: ArrayValue, IntValue, FloatValue, PointerValue, StructValue, VectorValue}

trait_value_set! {AggregateValue: ArrayValue, AggregateValueEnum, StructValue}
trait_value_set! {AnyValue: AnyValueEnum, BasicValueEnum, AggregateValueEnum, ArrayValue, IntValue, FloatValue, PhiValue, PointerValue, FunctionValue, StructValue, VectorValue, Value} // TODO: Remove Value
trait_value_set! {BasicValue: ArrayValue, BasicValueEnum, AggregateValueEnum, IntValue, FloatValue, StructValue, PointerValue, VectorValue}

// REVIEW: into/as/is functions are getting rediculious. Need macros or enum-methods crate

impl BasicValueEnum {
    pub(crate) fn new(value: LLVMValueRef) -> BasicValueEnum {
        let type_kind = unsafe {
            LLVMGetTypeKind(LLVMTypeOf(value))
        };

        match type_kind {
            LLVMTypeKind::LLVMFloatTypeKind |
            LLVMTypeKind::LLVMFP128TypeKind |
            LLVMTypeKind::LLVMDoubleTypeKind |
            LLVMTypeKind::LLVMHalfTypeKind |
            LLVMTypeKind::LLVMX86_FP80TypeKind |
            LLVMTypeKind::LLVMPPC_FP128TypeKind => BasicValueEnum::FloatValue(FloatValue::new(value)),
            LLVMTypeKind::LLVMIntegerTypeKind => BasicValueEnum::IntValue(IntValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => BasicValueEnum::StructValue(StructValue::new(value)),
            LLVMTypeKind::LLVMPointerTypeKind => BasicValueEnum::PointerValue(PointerValue::new(value)),
            LLVMTypeKind::LLVMArrayTypeKind => BasicValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMVectorTypeKind => panic!("TODO: Unsupported type: Vector"),
            _ => unreachable!("Unsupported type"),
        }
    }

    pub fn get_type(&self) -> BasicTypeEnum {
        let type_ = unsafe {
            LLVMTypeOf(self.as_value_ref())
        };

        BasicTypeEnum::new(type_)
    }
}

#[test]
fn test_linkage() {
    use context::Context;
    use module::Linkage::*;

    let context = Context::create();
    let module = context.create_module("testing");

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);

    let function = module.add_function("free_f32", &fn_type, None);

    assert_eq!(function.get_linkage(), ExternalLinkage);
}
