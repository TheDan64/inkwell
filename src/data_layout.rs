//! The data layout: LLVM's `DataLayout` -- which the C API still calls `LLVMTargetDataRef`,
//! the class's name until LLVM 3.2. One object describes a target's data model: endianness,
//! pointer sizes, the size and alignment of every type, struct layouts. A module carries one, a
//! target machine computes one, an execution engine has one.
//!
//! [`DataLayout`] owns its object; [`DataLayoutRef`] borrows one that belongs to a module or an
//! engine. Both answer the same queries.

use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;
use std::ptr::NonNull;

use llvm_sys::target::{
    LLVMABIAlignmentOfType, LLVMABISizeOfType, LLVMByteOrder, LLVMByteOrdering, LLVMCallFrameAlignmentOfType,
    LLVMCopyStringRepOfTargetData, LLVMCreateTargetData, LLVMDisposeTargetData, LLVMElementAtOffset,
    LLVMIntPtrTypeForASInContext, LLVMIntPtrTypeInContext, LLVMOffsetOfElement, LLVMOpaqueTargetData, LLVMPointerSize,
    LLVMPointerSizeForAS, LLVMPreferredAlignmentOfGlobal, LLVMPreferredAlignmentOfType, LLVMSizeOfTypeInBits,
    LLVMStoreSizeOfType, LLVMTargetDataRef,
};

use crate::AddressSpace;
use crate::context::AsContextRef;
use crate::support::{LLVMString, assert_niche, to_c_str};
use crate::types::{AnyType, AsTypeRef, IntType, StructType};
use crate::values::{AsValueRef, GlobalValue};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}

/// An owned data layout: parsed from a layout string by [`DataLayout::create`] or computed by
/// [`TargetMachine::get_data_layout`](crate::targets::TargetMachine::get_data_layout)
/// (`LLVMCreateTargetData` / `LLVMCreateTargetDataLayout`), and disposed with this value.
/// [`DataLayoutRef`] is the borrowed counterpart.
#[repr(transparent)]
pub struct DataLayout {
    data_layout: NonNull<LLVMOpaqueTargetData>,
}
const _: () = assert_niche::<DataLayout>();

/// A data layout owned by something else and borrowed from it for `'a`: a module's own
/// ([`Module::get_data_layout`](crate::module::Module::get_data_layout)) or an execution
/// engine's ([`ExecutionEngine::get_data_layout`](crate::execution_engine::ExecutionEngine::get_data_layout)).
/// It answers the same queries as [`DataLayout`]; [`DataLayout::as_ref`] borrows an owned one.
#[derive(Clone, Copy)]
pub struct DataLayoutRef<'a> {
    data_layout: NonNull<LLVMOpaqueTargetData>,
    _marker: PhantomData<&'a ()>,
}

impl DataLayout {
    /// Takes ownership of a raw data layout.
    ///
    /// # Safety
    ///
    /// `data_layout` must be non-null and owned by the caller (created by `LLVMCreateTargetData`
    /// or `LLVMCreateTargetDataLayout`, and not disposed).
    pub unsafe fn new(data_layout: LLVMTargetDataRef) -> DataLayout {
        DataLayout {
            data_layout: NonNull::new(data_layout).expect("data layout must be non-null"),
        }
    }

    /// Parses a layout string (`LLVMCreateTargetData`).
    ///
    /// On a malformed string LLVM ends the process with `report_fatal_error`: the C API has no
    /// error path (`llvm::DataLayout::parse` is not exported).
    pub fn create(str_repr: &str) -> DataLayout {
        let c_string = to_c_str(str_repr);

        unsafe { DataLayout::new(LLVMCreateTargetData(c_string.as_ptr())) }
    }

    /// Borrows this data layout.
    pub fn as_ref(&self) -> DataLayoutRef<'_> {
        DataLayoutRef {
            data_layout: self.data_layout,
            _marker: PhantomData,
        }
    }
}

impl<'a> DataLayoutRef<'a> {
    /// # Safety
    ///
    /// `data_layout` must be non-null and stay valid and unchanged for `'a`.
    pub(crate) unsafe fn new(data_layout: LLVMTargetDataRef) -> DataLayoutRef<'a> {
        DataLayoutRef {
            data_layout: NonNull::new(data_layout).expect("data layout must be non-null"),
            _marker: PhantomData,
        }
    }
}

impl<'a> From<&'a DataLayout> for DataLayoutRef<'a> {
    fn from(data_layout: &'a DataLayout) -> DataLayoutRef<'a> {
        data_layout.as_ref()
    }
}

impl<'a> From<&DataLayoutRef<'a>> for DataLayoutRef<'a> {
    fn from(data_layout: &DataLayoutRef<'a>) -> DataLayoutRef<'a> {
        *data_layout
    }
}

/// The queries, shared by the owned [`DataLayout`] and the borrowed [`DataLayoutRef`].
macro_rules! data_layout_methods {
    ($Self:ty) => {
        impl $Self {
            /// Acquires the underlying raw pointer belonging to this data layout.
            pub fn as_mut_ptr(&self) -> LLVMTargetDataRef {
                self.data_layout.as_ptr()
            }

            /// The layout string, as a copy: the C API only copies it out of the object
            /// (`LLVMCopyStringRepOfTargetData`).
            pub fn get_string_representation(&self) -> LLVMString {
                unsafe { LLVMString::new(LLVMCopyStringRepOfTargetData(self.data_layout.as_ptr())) }
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
            /// let data_layout = execution_engine.get_data_layout();
            /// let int_type = data_layout.ptr_sized_int_type_in_context(&context, None);
            /// ```
            #[deprecated(
                note = "This method will be removed in the future. Please use Context::ptr_sized_int_type instead."
            )]
            pub fn ptr_sized_int_type_in_context<'ctx>(
                &self,
                context: impl AsContextRef<'ctx>,
                address_space: Option<AddressSpace>,
            ) -> IntType<'ctx> {
                let int_type_ptr = match address_space {
                    Some(address_space) => unsafe {
                        LLVMIntPtrTypeForASInContext(context.as_ctx_ref(), self.data_layout.as_ptr(), address_space.0)
                    },
                    None => unsafe { LLVMIntPtrTypeInContext(context.as_ctx_ref(), self.data_layout.as_ptr()) },
                };

                unsafe { IntType::new(int_type_ptr) }
            }

            // REVIEW: Does this only work if Sized?
            pub fn get_bit_size(&self, type_: &dyn AnyType) -> u64 {
                unsafe { LLVMSizeOfTypeInBits(self.data_layout.as_ptr(), type_.as_type_ref()) }
            }

            pub fn get_byte_ordering(&self) -> ByteOrdering {
                let byte_ordering = unsafe { LLVMByteOrder(self.data_layout.as_ptr()) };

                match byte_ordering {
                    LLVMByteOrdering::LLVMBigEndian => ByteOrdering::BigEndian,
                    LLVMByteOrdering::LLVMLittleEndian => ByteOrdering::LittleEndian,
                }
            }

            pub fn get_pointer_byte_size(&self, address_space: Option<AddressSpace>) -> u32 {
                match address_space {
                    Some(address_space) => unsafe { LLVMPointerSizeForAS(self.data_layout.as_ptr(), address_space.0) },
                    None => unsafe { LLVMPointerSize(self.data_layout.as_ptr()) },
                }
            }

            pub fn get_store_size(&self, type_: &dyn AnyType) -> u64 {
                unsafe { LLVMStoreSizeOfType(self.data_layout.as_ptr(), type_.as_type_ref()) }
            }

            pub fn get_abi_size(&self, type_: &dyn AnyType) -> u64 {
                unsafe { LLVMABISizeOfType(self.data_layout.as_ptr(), type_.as_type_ref()) }
            }

            pub fn get_abi_alignment(&self, type_: &dyn AnyType) -> u32 {
                unsafe { LLVMABIAlignmentOfType(self.data_layout.as_ptr(), type_.as_type_ref()) }
            }

            pub fn get_call_frame_alignment(&self, type_: &dyn AnyType) -> u32 {
                unsafe { LLVMCallFrameAlignmentOfType(self.data_layout.as_ptr(), type_.as_type_ref()) }
            }

            pub fn get_preferred_alignment(&self, type_: &dyn AnyType) -> u32 {
                unsafe { LLVMPreferredAlignmentOfType(self.data_layout.as_ptr(), type_.as_type_ref()) }
            }

            pub fn get_preferred_alignment_of_global(&self, value: &GlobalValue) -> u32 {
                unsafe { LLVMPreferredAlignmentOfGlobal(self.data_layout.as_ptr(), value.as_value_ref()) }
            }

            pub fn element_at_offset(&self, struct_type: &StructType, offset: u64) -> u32 {
                unsafe { LLVMElementAtOffset(self.data_layout.as_ptr(), struct_type.as_type_ref(), offset) }
            }

            pub fn offset_of_element(&self, struct_type: &StructType, element: u32) -> Option<u64> {
                if element > struct_type.count_fields() - 1 {
                    return None;
                }

                unsafe {
                    Some(LLVMOffsetOfElement(
                        self.data_layout.as_ptr(),
                        struct_type.as_type_ref(),
                        element,
                    ))
                }
            }
        }

        impl fmt::Debug for $Self {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_struct(stringify!($Self).trim_end_matches("<'_>"))
                    .field("address", &self.data_layout)
                    .field("repr", &self.get_string_representation())
                    .finish()
            }
        }

        /// The layout string.
        impl fmt::Display for $Self {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(&self.get_string_representation().to_string_lossy())
            }
        }
    };
}

data_layout_methods!(DataLayout);
data_layout_methods!(DataLayoutRef<'_>);

impl Drop for DataLayout {
    fn drop(&mut self) {
        unsafe { LLVMDisposeTargetData(self.data_layout.as_ptr()) }
    }
}

/// Equality is equality of the layout strings (`llvm::DataLayout::operator==` is not in the
/// C API), so an owned and a borrowed layout compare too.
fn same_layout(a: LLVMTargetDataRef, b: LLVMTargetDataRef) -> bool {
    a == b
        || unsafe {
            *LLVMString::new(LLVMCopyStringRepOfTargetData(a)) == *LLVMString::new(LLVMCopyStringRepOfTargetData(b))
        }
}

impl PartialEq for DataLayout {
    fn eq(&self, other: &DataLayout) -> bool {
        same_layout(self.as_mut_ptr(), other.as_mut_ptr())
    }
}

impl Eq for DataLayout {}

impl<'a> PartialEq for DataLayoutRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        same_layout(self.as_mut_ptr(), other.as_mut_ptr())
    }
}

impl Eq for DataLayoutRef<'_> {}

impl<'a> PartialEq<DataLayoutRef<'a>> for DataLayout {
    fn eq(&self, other: &DataLayoutRef<'a>) -> bool {
        same_layout(self.as_mut_ptr(), other.as_mut_ptr())
    }
}

impl PartialEq<DataLayout> for DataLayoutRef<'_> {
    fn eq(&self, other: &DataLayout) -> bool {
        same_layout(self.as_mut_ptr(), other.as_mut_ptr())
    }
}

impl PartialEq<CStr> for DataLayoutRef<'_> {
    /// Compares the layout string.
    fn eq(&self, other: &CStr) -> bool {
        *self.get_string_representation() == *other
    }
}
