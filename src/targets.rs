use llvm_sys::core::LLVMDisposeMessage;
use llvm_sys::target::{LLVMTargetDataRef, LLVMCopyStringRepOfTargetData, LLVMSizeOfTypeInBits, LLVMCreateTargetData, LLVMAddTargetData, LLVMByteOrder, LLVMPointerSize, LLVMByteOrdering, LLVMStoreSizeOfType, LLVMABISizeOfType, LLVMABIAlignmentOfType, LLVMCallFrameAlignmentOfType, LLVMPreferredAlignmentOfType, LLVMPreferredAlignmentOfGlobal, LLVMElementAtOffset, LLVMOffsetOfElement, LLVMDisposeTargetData};
use llvm_sys::target_machine::{LLVMGetFirstTarget, LLVMTargetRef, LLVMGetNextTarget, LLVMGetTargetFromName, LLVMGetTargetFromTriple, LLVMGetTargetName, LLVMGetTargetDescription, LLVMTargetHasJIT, LLVMTargetHasTargetMachine, LLVMTargetHasAsmBackend, LLVMTargetMachineRef, LLVMDisposeTargetMachine, LLVMGetTargetMachineTarget, LLVMGetTargetMachineTriple, LLVMSetTargetMachineAsmVerbosity, LLVMCreateTargetMachine, LLVMGetTargetMachineCPU, LLVMGetTargetMachineFeatureString, LLVMGetDefaultTargetTriple, LLVMAddAnalysisPasses, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMRelocMode};

use data_layout::DataLayout;
use pass_manager::PassManager;
use types::{AnyType, AsTypeRef, StructType};
use values::AnyValue;

use std::ffi::{CStr, CString};
use std::mem::zeroed;
use std::ptr;

pub enum CodeGenOptLevel {
    Less,
    Default,
    Aggressive,
}

pub enum CodeModel {
    Default,
    JITDefault,
    Small,
    Kernel,
    Medium,
    Large,
}

pub enum RelocMode {
    Default,
    Static,
    PIC,
    DynamicNoPic,
}

pub struct Target {
    target: LLVMTargetRef,
}

impl Target {
    fn new(target: LLVMTargetRef) -> Self {
        assert!(!target.is_null());

        Target {
            target,
        }
    }

    // REVIEW: Should this just initialize all? Is opt into each a good idea?

    // pub fn initialize_x86(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_arm(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_mips(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_aarch64(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_r600(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_system_z(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_hexagon(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_nvptx(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_cpp_backend(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_msp430(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_x_core(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_power_pc(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    // pub fn initialize_sparc(base: bool, info: bool, machine_code: bool, asm_printer: bool, asm_parser: bool, dissassembler: bool) {

    // }

    pub fn create_target_machine(&self, triple: &str, cpu: &str, features: &str, level: Option<CodeGenOptLevel>, reloc_mode: RelocMode, code_model: CodeModel) -> TargetMachine {
        let triple = CString::new(triple).expect("Conversion to CString failed unexpectedly");
        let cpu = CString::new(cpu).expect("Conversion to CString failed unexpectedly");
        let features = CString::new(features).expect("Conversion to CString failed unexpectedly");
        let level = match level {
            None => LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            Some(CodeGenOptLevel::Less) => LLVMCodeGenOptLevel::LLVMCodeGenLevelLess,
            Some(CodeGenOptLevel::Default) => LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            Some(CodeGenOptLevel::Aggressive) => LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
        };
        let code_model = match code_model {
            CodeModel::Default => LLVMCodeModel::LLVMCodeModelDefault,
            CodeModel::JITDefault => LLVMCodeModel::LLVMCodeModelJITDefault,
            CodeModel::Small => LLVMCodeModel::LLVMCodeModelSmall,
            CodeModel::Kernel => LLVMCodeModel::LLVMCodeModelKernel,
            CodeModel::Medium => LLVMCodeModel::LLVMCodeModelMedium,
            CodeModel::Large => LLVMCodeModel::LLVMCodeModelLarge,
        };
        let reloc_mode = match reloc_mode {
            RelocMode::Default => LLVMRelocMode::LLVMRelocDefault,
            RelocMode::Static => LLVMRelocMode::LLVMRelocStatic,
            RelocMode::PIC => LLVMRelocMode::LLVMRelocPIC,
            RelocMode::DynamicNoPic => LLVMRelocMode::LLVMRelocDynamicNoPic,
        };
        let target_machine = unsafe {
            LLVMCreateTargetMachine(self.target, triple.as_ptr(), cpu.as_ptr(), features.as_ptr(), level, reloc_mode, code_model)
        };

        TargetMachine::new(target_machine)
    }

    pub fn get_first() -> Option<Self> {
        let target = unsafe {
            LLVMGetFirstTarget()
        };

        if target.is_null() {
            return None;
        }

        Some(Target::new(target))
    }

    pub fn get_next(&self) -> Option<Target> {
        let target = unsafe {
            LLVMGetNextTarget(self.target)
        };

        if target.is_null() {
            return None;
        }

        Some(Target::new(target))
    }

    pub fn get_name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTargetName(self.target))
        }
    }

    pub fn get_description(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTargetDescription(self.target))
        }
    }

    pub fn from_name(name: &str) -> Target { // REVIEW: Option?
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let target = unsafe {
            LLVMGetTargetFromName(c_string.as_ptr())
        };

        Target::new(target)
    }

    pub fn from_triple(triple: &str) -> Result<Target, String> {
        let c_string = CString::new(triple).expect("Conversion to CString failed unexpectedly");
        let target = ptr::null_mut();
        let err_str = unsafe { zeroed() };

        let code = unsafe {
            LLVMGetTargetFromTriple(c_string.as_ptr(), target, err_str)
        };

        if code == 1 { // REVIEW: 1 is error value
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(*err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(*err_str);

                rust_str
            };

            return Err(rust_str);
        }

        unsafe {
            Ok(Target::new(*target))
        }
    }

    pub fn has_jit(&self) -> bool {
        unsafe {
            LLVMTargetHasJIT(self.target) == 1
        }
    }

    pub fn has_target_machine(&self) -> bool {
        unsafe {
            LLVMTargetHasTargetMachine(self.target) == 1
        }
    }

    pub fn has_asm_backend(&self) -> bool {
        unsafe {
            LLVMTargetHasAsmBackend(self.target) == 1
        }
    }
}

pub struct TargetMachine {
    target_machine: LLVMTargetMachineRef,
}

impl TargetMachine {
    fn new(target_machine: LLVMTargetMachineRef) -> Self {
        assert!(!target_machine.is_null());

        TargetMachine {
            target_machine,
        }
    }

    pub fn get_target(&self)-> Target {
        let target = unsafe {
            LLVMGetTargetMachineTarget(self.target_machine)
        };

        Target::new(target)
    }

    pub fn get_triple(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTargetMachineTriple(self.target_machine))
        }
    }

    pub fn get_default_triple() -> &'static CStr { // FIXME: Probably not static?
        unsafe {
            CStr::from_ptr(LLVMGetDefaultTargetTriple())
        }
    }

    pub fn get_cpu(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTargetMachineCPU(self.target_machine))
        }
    }

    pub fn get_feature_string(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTargetMachineFeatureString(self.target_machine))
        }
    }

    pub fn set_asm_verbosity(&self, verbosity: bool) {
        unsafe {
            LLVMSetTargetMachineAsmVerbosity(self.target_machine, verbosity as i32)
        }
    }

    pub fn add_analysis_passes(&self, pass_manager: &PassManager) {
        unsafe {
            LLVMAddAnalysisPasses(self.target_machine, pass_manager.pass_manager)
        }
    }
}

impl Drop for TargetMachine {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeTargetMachine(self.target_machine)
        }
    }
}

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
fn test_target() {
    use llvm_sys::target:: LLVMInitializeX86TargetInfo;

    // REVIEW: Some of the machine specific stuff may vary. Should allow multiple possibilites
    assert_eq!(TargetMachine::get_default_triple(), &*CString::new("x86_64-pc-linux-gnu").unwrap());
    assert!(Target::get_first().is_none());

    // TODO: Replace with safe wrapper
    unsafe {
        LLVMInitializeX86TargetInfo();
    }

    let target = Target::get_first().expect("Did not find any target");

    assert_eq!(target.get_name(), &*CString::new("x86-64").unwrap());
    assert_eq!(target.get_description(), &*CString::new("64-bit X86: EM64T and AMD64").unwrap());
    assert!(target.has_jit());
    assert!(!target.has_asm_backend());
    assert!(!target.has_target_machine());

    // TODO: See what happens to create_target_machine when when target.has_target_machine() is false
    // Maybe it should return an Option<TargetMachine>
    // TODO: TargetMachine testing

    target.get_next().expect("Did not find any target2");
}

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
