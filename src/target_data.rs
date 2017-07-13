use llvm_sys::target::{LLVMTargetDataRef, LLVMCopyStringRepOfTargetData, LLVMSizeOfTypeInBits, LLVMCreateTargetData, LLVMAddTargetData, LLVMByteOrder, LLVMPointerSize, LLVMByteOrdering, LLVMStoreSizeOfType, LLVMABISizeOfType, LLVMABIAlignmentOfType, LLVMCallFrameAlignmentOfType, LLVMPreferredAlignmentOfType, LLVMPreferredAlignmentOfGlobal, LLVMElementAtOffset, LLVMOffsetOfElement, LLVMDisposeTargetData};

use data_layout::DataLayout;
use pass_manager::PassManager;
use types::{AnyType, AsTypeRef, StructType};
use values::AnyValue;

use std::ffi::CString;

#[derive(Debug, PartialEq)]
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}

pub struct TargetData {
    pub(crate) target_data: LLVMTargetDataRef,
}

impl TargetData {
    pub(crate) fn new(target_data: LLVMTargetDataRef) -> TargetData {
        assert!(!target_data.is_null());

        TargetData {
            target_data: target_data
        }
    }

    pub fn get_data_layout(&self) -> DataLayout {
        let data_layout = unsafe {
            LLVMCopyStringRepOfTargetData(self.target_data)
        };

        DataLayout::new(data_layout)
    }

    // REVIEW: Does this only work if Sized?
    pub fn get_bit_size(&self, type_: &AnyType) -> u64 {
        unsafe {
            LLVMSizeOfTypeInBits(self.target_data, type_.as_type_ref())
        }
    }

    // REVIEW: Untested
    pub fn create(str_repr: &str) -> TargetData {
        let c_string = CString::new(str_repr).expect("Conversion to CString failed unexpectedly");

        let target_data = unsafe {
            LLVMCreateTargetData(c_string.as_ptr())
        };

        TargetData::new(target_data)
    }

    // REVIEW: Untested
    pub fn add_target_data(&self, pass_manager: &PassManager) {
        unsafe {
            LLVMAddTargetData(self.target_data, pass_manager.pass_manager)
        }
    }

    pub fn get_byte_ordering(&self) -> ByteOrdering {
        let byte_ordering = unsafe {
            LLVMByteOrder(self.target_data)
        };

        match byte_ordering {
            LLVMByteOrdering::LLVMBigEndian => ByteOrdering::BigEndian,
            LLVMByteOrdering::LLVMLittleEndian => ByteOrdering::LittleEndian,
        }
    }

    pub fn get_pointer_byte_size(&self) -> u32 {
        unsafe {
            LLVMPointerSize(self.target_data)
        }
    }

    pub fn get_store_size(&self, type_: &AnyType) -> u64 {
        unsafe {
            LLVMStoreSizeOfType(self.target_data, type_.as_type_ref())
        }
    }

    pub fn get_abi_size(&self, type_: &AnyType) -> u64 {
        unsafe {
            LLVMABISizeOfType(self.target_data, type_.as_type_ref())
        }
    }

    pub fn get_abi_alignment(&self, type_: &AnyType) -> u32 {
        unsafe {
            LLVMABIAlignmentOfType(self.target_data, type_.as_type_ref())
        }
    }

    pub fn get_call_frame_alignment(&self, type_: &AnyType) -> u32 {
        unsafe {
            LLVMCallFrameAlignmentOfType(self.target_data, type_.as_type_ref())
        }
    }

    pub fn get_preferred_alignment(&self, type_: &AnyType) -> u32 {
        unsafe {
            LLVMPreferredAlignmentOfType(self.target_data, type_.as_type_ref())
        }
    }

    // REVIEW: Untested
    pub fn get_preferred_alignment_of_global(&self, value: &AnyValue) -> u32 {
        unsafe {
            LLVMPreferredAlignmentOfGlobal(self.target_data, value.as_value_ref())
        }
    }

    pub fn element_at_offset(&self, type_: &StructType, offset: u64) -> u32 {
        unsafe {
            LLVMElementAtOffset(self.target_data, type_.as_type_ref(), offset)
        }
    }

    // FIXME: Out of bounds returns bad data, should return Option<u64>
    pub fn offset_of_element(&self, type_: &StructType, element: u32) -> u64 {
        unsafe {
            LLVMOffsetOfElement(self.target_data, type_.as_type_ref(), element)
        }
    }
}

// FIXME: Make sure this doesn't SegFault:
// impl Drop for TargetData {
//     fn drop(&mut self) {
//         unsafe {
//             LLVMDisposeTargetData(self.target_data)
//         }
//     }
// }

#[test]
fn test_target_data() {
    use context::Context;

    let context = Context::create();
    let module = context.create_module("sum");
    let builder = context.create_builder();
    let execution_engine = module.create_execution_engine(true).unwrap();
    let target_data = execution_engine.get_target_data();

    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let f32_type = context.f32_type();
    let f64_type = context.f64_type();
    let struct_type = context.struct_type(&[&i32_type, &i64_type, &f64_type, &f32_type], false, "struct1");
    let struct_type2 = context.struct_type(&[&f32_type, &i32_type, &i64_type, &f64_type], false, "struct2");

    let data_layout = target_data.get_data_layout(); // TODO: See if you can test data_layout

    assert_eq!(target_data.get_bit_size(&i32_type), 32);
    assert_eq!(target_data.get_bit_size(&i64_type), 64);
    assert_eq!(target_data.get_bit_size(&f32_type), 32);
    assert_eq!(target_data.get_bit_size(&f64_type), 64);
    assert_eq!(target_data.get_bit_size(&struct_type), 256);
    assert_eq!(target_data.get_bit_size(&struct_type2), 192);

    // REVIEW: What if these fail on a different system?
    assert_eq!(target_data.get_byte_ordering(), ByteOrdering::LittleEndian);
    assert_eq!(target_data.get_pointer_byte_size(), 8);

    // REVIEW: Are these just byte size? Maybe rename to get_byte_size?
    assert_eq!(target_data.get_store_size(&i32_type), 4);
    assert_eq!(target_data.get_store_size(&i64_type), 8);
    assert_eq!(target_data.get_store_size(&f32_type), 4);
    assert_eq!(target_data.get_store_size(&f64_type), 8);
    assert_eq!(target_data.get_store_size(&struct_type), 32);
    assert_eq!(target_data.get_store_size(&struct_type2), 24);

    // REVIEW: What's the difference between this an above?
    assert_eq!(target_data.get_abi_size(&i32_type), 4);
    assert_eq!(target_data.get_abi_size(&i64_type), 8);
    assert_eq!(target_data.get_abi_size(&f32_type), 4);
    assert_eq!(target_data.get_abi_size(&f64_type), 8);
    assert_eq!(target_data.get_abi_size(&struct_type), 32);
    assert_eq!(target_data.get_abi_size(&struct_type2), 24);

    assert_eq!(target_data.get_abi_alignment(&i32_type), 4);
    assert_eq!(target_data.get_abi_alignment(&i64_type), 8);
    assert_eq!(target_data.get_abi_alignment(&f32_type), 4);
    assert_eq!(target_data.get_abi_alignment(&f64_type), 8);
    assert_eq!(target_data.get_abi_alignment(&struct_type), 8);
    assert_eq!(target_data.get_abi_alignment(&struct_type2), 8);

    assert_eq!(target_data.get_call_frame_alignment(&i32_type), 4);
    assert_eq!(target_data.get_call_frame_alignment(&i64_type), 8);
    assert_eq!(target_data.get_call_frame_alignment(&f32_type), 4);
    assert_eq!(target_data.get_call_frame_alignment(&f64_type), 8);
    assert_eq!(target_data.get_call_frame_alignment(&struct_type), 8);
    assert_eq!(target_data.get_call_frame_alignment(&struct_type2), 8);

    assert_eq!(target_data.get_preferred_alignment(&i32_type), 4);
    assert_eq!(target_data.get_preferred_alignment(&i64_type), 8);
    assert_eq!(target_data.get_preferred_alignment(&f32_type), 4);
    assert_eq!(target_data.get_preferred_alignment(&f64_type), 8);
    assert_eq!(target_data.get_preferred_alignment(&struct_type), 8);
    assert_eq!(target_data.get_preferred_alignment(&struct_type2), 8);

    // REVIEW: offset in bytes? Rename to byte_offset_of_element?
    assert_eq!(target_data.offset_of_element(&struct_type, 0), 0);
    assert_eq!(target_data.offset_of_element(&struct_type, 1), 8);
    assert_eq!(target_data.offset_of_element(&struct_type, 2), 16);
    assert_eq!(target_data.offset_of_element(&struct_type, 3), 24);
    // assert_eq!(target_data.offset_of_element(&struct_type, 4), 32); // FIXME: Out of bounds returns bad data, maybe LLVM bug?

    assert_eq!(target_data.element_at_offset(&struct_type, 0), 0);
    assert_eq!(target_data.element_at_offset(&struct_type, 4), 0);
    assert_eq!(target_data.element_at_offset(&struct_type, 8), 1);
    assert_eq!(target_data.element_at_offset(&struct_type, 16), 2);
    assert_eq!(target_data.element_at_offset(&struct_type, 24), 3);
    assert_eq!(target_data.element_at_offset(&struct_type, 32), 3);
    assert_eq!(target_data.element_at_offset(&struct_type, 4200), 3); // Odd but seems to cap at max element number

    assert_eq!(target_data.offset_of_element(&struct_type2, 0), 0);
    assert_eq!(target_data.offset_of_element(&struct_type2, 1), 4);
    assert_eq!(target_data.offset_of_element(&struct_type2, 2), 8);
    assert_eq!(target_data.offset_of_element(&struct_type2, 3), 16);
    assert_eq!(target_data.offset_of_element(&struct_type2, 4), 0); // Maybe ok because of the element positioning?
    assert_eq!(target_data.offset_of_element(&struct_type2, 5), 0); // Maybe ok because of the element positioning?

    assert_eq!(target_data.element_at_offset(&struct_type2, 0), 0);
    assert_eq!(target_data.element_at_offset(&struct_type2, 2), 0);
    assert_eq!(target_data.element_at_offset(&struct_type2, 4), 1);
    assert_eq!(target_data.element_at_offset(&struct_type2, 8), 2);
    assert_eq!(target_data.element_at_offset(&struct_type2, 16), 3);
    assert_eq!(target_data.element_at_offset(&struct_type2, 32), 3);
    assert_eq!(target_data.element_at_offset(&struct_type2, 4200), 3); // Odd but seems to cap at max element number
}
