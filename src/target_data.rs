use llvm_sys::target::{LLVMTargetDataRef, LLVMCopyStringRepOfTargetData, LLVMSizeOfTypeInBits, LLVMCreateTargetData, LLVMAddTargetData, LLVMByteOrder, LLVMPointerSize, LLVMByteOrdering, LLVMStoreSizeOfType, LLVMABISizeOfType, LLVMABIAlignmentOfType, LLVMCallFrameAlignmentOfType, LLVMPreferredAlignmentOfType, LLVMPreferredAlignmentOfGlobal, LLVMElementAtOffset, LLVMOffsetOfElement, LLVMDisposeTargetData};

use data_layout::DataLayout;
use pass_manager::PassManager;
use types::{AnyType, AsLLVMTypeRef, StructType};
use values::AnyValue;

use std::ffi::CString;

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
            LLVMSizeOfTypeInBits(self.target_data, type_.as_llvm_type_ref())
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

    // REVIEW: Untested
    pub fn get_byte_ordering(&self) -> ByteOrdering {
        let byte_ordering = unsafe {
            LLVMByteOrder(self.target_data)
        };

        match byte_ordering {
            LLVMByteOrdering::LLVMBigEndian => ByteOrdering::BigEndian,
            LLVMByteOrdering::LLVMLittleEndian => ByteOrdering::LittleEndian,
        }
    }

    // REVIEW: Untested
    pub fn get_pointer_size(&self) -> u32 {
        unsafe {
            LLVMPointerSize(self.target_data)
        }
    }

    // REVIEW: Untested
    pub fn get_store_size(&self, type_: &AnyType) -> u64 {
        unsafe {
            LLVMStoreSizeOfType(self.target_data, type_.as_llvm_type_ref())
        }
    }

    // REVIEW: Untested
    pub fn get_abi_size(&self, type_: &AnyType) -> u64 {
        unsafe {
            LLVMABISizeOfType(self.target_data, type_.as_llvm_type_ref())
        }
    }

    // REVIEW: Untested
    pub fn get_abi_alignment(&self, type_: &AnyType) -> u32 {
        unsafe {
            LLVMABIAlignmentOfType(self.target_data, type_.as_llvm_type_ref())
        }
    }

    // REVIEW: Untested
    pub fn get_call_frame_alignment(&self, type_: &AnyType) -> u32 {
        unsafe {
            LLVMCallFrameAlignmentOfType(self.target_data, type_.as_llvm_type_ref())
        }
    }

    // REVIEW: Untested
    pub fn get_preferred_alignment(&self, type_: &AnyType) -> u32 {
        unsafe {
            LLVMPreferredAlignmentOfType(self.target_data, type_.as_llvm_type_ref())
        }
    }

    // REVIEW: Untested
    pub fn get_preferred_alignment_of_global(&self, value: &AnyValue) -> u32 {
        unsafe {
            LLVMPreferredAlignmentOfGlobal(self.target_data, value.as_llvm_value_ref())
        }
    }

    // REVIEW: Untested
    pub fn element_at_offset(&self, type_: &StructType, offset: u64) -> u32 {
        unsafe {
            LLVMElementAtOffset(self.target_data, type_.as_llvm_type_ref(), offset)
        }
    }

    // REVIEW: Untested
    pub fn offset_of_element(&self, type_: &StructType, element: u32) -> u64 {
        unsafe {
            LLVMOffsetOfElement(self.target_data, type_.as_llvm_type_ref(), element)
        }
    }
}

// TODO: Make sure this doesn't SegFault:
// impl Drop for TargetData {
//     fn drop(&mut self) {
//         unsafe {
//             LLVMDisposeTargetData(self.target_data)
//         }
//     }
// }

