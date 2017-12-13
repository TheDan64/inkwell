use llvm_sys::core::{LLVMConstNamedStruct, LLVMConstStruct, LLVMConstNull, LLVMStructType, LLVMCountStructElementTypes, LLVMGetStructElementTypes, LLVMGetStructName, LLVMIsPackedStruct, LLVMIsOpaqueStruct, LLVMStructSetBody};
#[cfg(not(feature = "llvm3-6"))]
use llvm_sys::core::LLVMStructGetTypeAtIndex;
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};

use std::ffi::CStr;
use std::mem::forget;

use AddressSpace;
use context::ContextRef;
use types::traits::AsTypeRef;
use types::{Type, BasicType, BasicTypeEnum, ArrayType, PointerType, FunctionType, VectorType};
use values::{BasicValue, StructValue, PointerValue, IntValue};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

    // TODO: Would be great to be able to smartly be able to do this by field name
    #[cfg(not(feature = "llvm3-6"))]
    pub fn get_field_type_at_index(&self, index: u32) -> Option<BasicTypeEnum> {
        // LLVM doesn't seem to just return null if opaque.
        // TODO: One day, with SubTypes (& maybe specialization?) we could just
        // impl this method for non opaque structs only
        if self.is_opaque() {
            return None;
        }

        // OoB indexing seems to be unchecked and therefore is UB
        if index >= self.count_fields() {
            return None;
        }

        let type_ = unsafe {
            LLVMStructGetTypeAtIndex(self.struct_type.type_, index)
        };

        Some(BasicTypeEnum::new(type_))
    }

    // REVIEW: What's the difference between these two??
    pub fn const_named_struct(&self, values: &[&BasicValue]) -> StructValue {
        let mut args: Vec<LLVMValueRef> = values.iter()
                                                .map(|val| val.as_value_ref())
                                                .collect();
        let value = unsafe {
            LLVMConstNamedStruct(self.as_type_ref(), args.as_mut_ptr(), args.len() as u32)
        };

        StructValue::new(value)
    }

    pub fn const_struct(values: &[&BasicValue], packed: bool) -> StructValue {
        let mut args: Vec<LLVMValueRef> = values.iter()
                                                .map(|val| val.as_value_ref())
                                                .collect();
        let value = unsafe {
            LLVMConstStruct(args.as_mut_ptr(), args.len() as u32, packed as i32)
        };

        StructValue::new(value)
    }

    pub fn const_null_ptr(&self) -> PointerValue {
        self.struct_type.const_null_ptr()
    }

    pub fn const_null(&self) -> StructValue {
        let null = unsafe {
            LLVMConstNull(self.as_type_ref())
        };

        StructValue::new(null)
    }

    // REVIEW: Can be false if opaque. To make a const fn, we'd have to have
    // have separate impls for Struct<Opaque> and StructType<T*>
    pub fn is_sized(&self) -> bool {
        self.struct_type.is_sized()
    }

    // TODO: impl it only for StructType<T*>
    pub fn size_of(&self) -> Option<IntValue> {
        if self.is_sized() {
            return Some(self.struct_type.size_of());
        }

        None
    }

    pub fn get_context(&self) -> ContextRef {
        self.struct_type.get_context()
    }

    pub fn get_name(&self) -> Option<&CStr> {
        let name = unsafe {
            LLVMGetStructName(self.as_type_ref())
        };

        if name.is_null() {
            return None;
        }

        let c_str = unsafe {
            CStr::from_ptr(name)
        };

        Some(c_str)
    }

    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.struct_type.ptr_type(address_space)
    }

    pub fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
        self.struct_type.fn_type(param_types, is_var_args)
    }

    pub fn array_type(&self, size: u32) -> ArrayType {
        self.struct_type.array_type(size)
    }

    pub fn is_packed(&self) -> bool {
        unsafe {
            LLVMIsPackedStruct(self.struct_type.type_) == 1
        }
    }

    // TODO: Worth documenting that a sturct is opaque when types are not
    // yet assigned (empty array to struct_type)
    pub fn is_opaque(&self) -> bool {
        unsafe {
            LLVMIsOpaqueStruct(self.struct_type.type_) == 1
        }
    }

    // REVIEW: Is there an equivalent method to make opaque?
    // REVIEW: No way to set name like in context.struct_type() method?
    // DOC: This method will not create an opaque struct, even if empty array is passed!
    pub fn struct_type(field_types: &[&BasicType], packed: bool) -> Self {
        let mut field_types: Vec<LLVMTypeRef> = field_types.iter()
                                                           .map(|val| val.as_type_ref())
                                                           .collect();
        let struct_type = unsafe {
            LLVMStructType(field_types.as_mut_ptr(), field_types.len() as u32, packed as i32)
        };

        StructType::new(struct_type)
    }

    pub fn count_fields(&self) -> u32 {
        unsafe {
            LLVMCountStructElementTypes(self.as_type_ref())
        }
    }

    // REVIEW: Method name
    pub fn get_field_types(&self) -> Vec<BasicTypeEnum> {
        let count = self.count_fields();
        let mut raw_vec: Vec<LLVMTypeRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetStructElementTypes(self.as_type_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| BasicTypeEnum::new(*val)).collect()
    }

    pub fn print_to_string(&self) -> &CStr {
        self.struct_type.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.struct_type.print_to_stderr()
    }

    pub fn get_undef(&self) -> StructValue {
        StructValue::new(self.struct_type.get_undef())
    }

    // REVIEW: SubTypes should allow this to only be implemented for StructType<Opaque> one day
    // but would have to return StructType<Tys>
    // REVIEW: What happens if called with &[]? Should that still be opaque? Does the call break?
    pub fn set_body(&self, field_types: &[&BasicType], packed: bool) -> bool {
        let is_opaque = self.is_opaque();
        let mut field_types: Vec<LLVMTypeRef> = field_types.iter()
                                                           .map(|val| val.as_type_ref())
                                                           .collect();
        if is_opaque {
            unsafe {
                LLVMStructSetBody(self.as_type_ref(), field_types.as_mut_ptr(), field_types.len() as u32, packed as i32);
            }
        }

        is_opaque
    }

    pub fn vec_type(&self, size: u32) -> VectorType {
        self.struct_type.vec_type(size)
    }
}

impl AsTypeRef for StructType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.struct_type.type_
    }
}
