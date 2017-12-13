use llvm_sys::core::LLVMDisposeMessage;
use llvm_sys::target::{LLVMTargetDataRef, LLVMCopyStringRepOfTargetData, LLVMSizeOfTypeInBits, LLVMCreateTargetData, LLVMAddTargetData, LLVMByteOrder, LLVMPointerSize, LLVMByteOrdering, LLVMStoreSizeOfType, LLVMABISizeOfType, LLVMABIAlignmentOfType, LLVMCallFrameAlignmentOfType, LLVMPreferredAlignmentOfType, LLVMPreferredAlignmentOfGlobal, LLVMElementAtOffset, LLVMOffsetOfElement, LLVMDisposeTargetData, LLVMPointerSizeForAS, LLVMIntPtrType, LLVMIntPtrTypeForAS, LLVMIntPtrTypeInContext, LLVMIntPtrTypeForASInContext};
use llvm_sys::target_machine::{LLVMGetFirstTarget, LLVMTargetRef, LLVMGetNextTarget, LLVMGetTargetFromName, LLVMGetTargetFromTriple, LLVMGetTargetName, LLVMGetTargetDescription, LLVMTargetHasJIT, LLVMTargetHasTargetMachine, LLVMTargetHasAsmBackend, LLVMTargetMachineRef, LLVMDisposeTargetMachine, LLVMGetTargetMachineTarget, LLVMGetTargetMachineTriple, LLVMSetTargetMachineAsmVerbosity, LLVMCreateTargetMachine, LLVMGetTargetMachineCPU, LLVMGetTargetMachineFeatureString, LLVMGetDefaultTargetTriple, LLVMAddAnalysisPasses, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMRelocMode, LLVMCodeGenFileType, LLVMTargetMachineEmitToMemoryBuffer, LLVMTargetMachineEmitToFile};

use OptimizationLevel;
use context::Context;
use data_layout::DataLayout;
use memory_buffer::MemoryBuffer;
use module::Module;
use passes::PassManager;
use types::{AnyType, AsTypeRef, StructType, PointerType};
use values::{AsValueRef, GlobalValue};

use std::default::Default;
use std::ffi::{CStr, CString};
use std::mem::zeroed;
use std::path::Path;
use std::ptr;

#[derive(Debug, PartialEq, Eq)]
pub enum CodeModel {
    Default,
    JITDefault,
    Small,
    Kernel,
    Medium,
    Large,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RelocMode {
    Default,
    Static,
    PIC,
    DynamicNoPic,
}


#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    Assembly,
    Object,
}

impl FileType {
    fn as_llvm_file_type(&self) -> LLVMCodeGenFileType {
        match *self {
            FileType::Assembly => LLVMCodeGenFileType::LLVMAssemblyFile,
            FileType::Object => LLVMCodeGenFileType::LLVMObjectFile,
        }
    }
}

// TODO: Doc: Base gets you TargetMachine support, machine_code gets you asm_backend
#[derive(Debug, PartialEq, Eq)]
pub struct InitializationConfig {
    pub asm_parser: bool,
    pub asm_printer: bool,
    pub base: bool,
    pub disassembler: bool,
    pub info: bool,
    pub machine_code: bool,
}

impl Default for InitializationConfig {
    fn default() -> Self {
        InitializationConfig {
            asm_parser: true,
            asm_printer: true,
            base: true,
            disassembler: true,
            info: true,
            machine_code: true,
        }
    }
}

#[derive(Debug)]
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
    pub fn initialize_x86(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeX86Target, LLVMInitializeX86TargetInfo, LLVMInitializeX86TargetMC, LLVMInitializeX86Disassembler, LLVMInitializeX86AsmPrinter, LLVMInitializeX86AsmParser};

        unsafe {
            if config.base {
                LLVMInitializeX86Target()
            }

            if config.info {
                LLVMInitializeX86TargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeX86AsmPrinter()
            }

            if config.asm_parser {
                LLVMInitializeX86AsmParser()
            }

            if config.disassembler {
                LLVMInitializeX86Disassembler()
            }

            if config.machine_code {
                LLVMInitializeX86TargetMC()
            }
        }
    }

    pub fn initialize_arm(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeARMTarget, LLVMInitializeARMTargetInfo, LLVMInitializeARMTargetMC, LLVMInitializeARMDisassembler, LLVMInitializeARMAsmPrinter, LLVMInitializeARMAsmParser};

        unsafe {
            if config.base {
                LLVMInitializeARMTarget()
            }

            if config.info {
                LLVMInitializeARMTargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeARMAsmPrinter()
            }

            if config.asm_parser {
                LLVMInitializeARMAsmParser()
            }

            if config.disassembler {
                LLVMInitializeARMDisassembler()
            }

            if config.machine_code {
                LLVMInitializeARMTargetMC()
            }
        }
    }

    pub fn initialize_mips(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeMipsTarget, LLVMInitializeMipsTargetInfo, LLVMInitializeMipsTargetMC, LLVMInitializeMipsDisassembler, LLVMInitializeMipsAsmPrinter, LLVMInitializeMipsAsmParser};

        unsafe {
            if config.base {
                LLVMInitializeMipsTarget()
            }

            if config.info {
                LLVMInitializeMipsTargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeMipsAsmPrinter()
            }

            if config.asm_parser {
                LLVMInitializeMipsAsmParser()
            }

            if config.disassembler {
                LLVMInitializeMipsDisassembler()
            }

            if config.machine_code {
                LLVMInitializeMipsTargetMC()
            }
        }
    }

    pub fn initialize_aarch64(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeAArch64Target, LLVMInitializeAArch64TargetInfo, LLVMInitializeAArch64TargetMC, LLVMInitializeAArch64Disassembler, LLVMInitializeAArch64AsmPrinter, LLVMInitializeAArch64AsmParser};

        unsafe {
            if config.base {
                LLVMInitializeAArch64Target()
            }

            if config.info {
                LLVMInitializeAArch64TargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeAArch64AsmPrinter()
            }

            if config.asm_parser {
                LLVMInitializeAArch64AsmParser()
            }

            if config.disassembler {
                LLVMInitializeAArch64Disassembler()
            }

            if config.machine_code {
                LLVMInitializeAArch64TargetMC()
            }
        }
    }

    pub fn initialize_amd_gpu(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeAMDGPUTarget, LLVMInitializeAMDGPUTargetInfo, LLVMInitializeAMDGPUTargetMC, LLVMInitializeAMDGPUAsmPrinter, LLVMInitializeAMDGPUAsmParser};

        unsafe {
            if config.base {
                LLVMInitializeAMDGPUTarget()
            }

            if config.info {
                LLVMInitializeAMDGPUTargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeAMDGPUAsmPrinter()
            }

            if config.asm_parser {
                LLVMInitializeAMDGPUAsmParser()
            }

            if config.machine_code {
                LLVMInitializeAMDGPUTargetMC()
            }
        }
    }

    pub fn initialize_system_z(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeSystemZTarget, LLVMInitializeSystemZTargetInfo, LLVMInitializeSystemZTargetMC, LLVMInitializeSystemZDisassembler, LLVMInitializeSystemZAsmPrinter, LLVMInitializeSystemZAsmParser};

        unsafe {
            if config.base {
                LLVMInitializeSystemZTarget()
            }

            if config.info {
                LLVMInitializeSystemZTargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeSystemZAsmPrinter()
            }

            if config.asm_parser {
                LLVMInitializeSystemZAsmParser()
            }

            if config.disassembler {
                LLVMInitializeSystemZDisassembler()
            }

            if config.machine_code {
                LLVMInitializeSystemZTargetMC()
            }
        }
    }

    pub fn initialize_hexagon(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeHexagonTarget, LLVMInitializeHexagonTargetInfo, LLVMInitializeHexagonTargetMC, LLVMInitializeHexagonDisassembler, LLVMInitializeHexagonAsmPrinter};

        unsafe {
            if config.base {
                LLVMInitializeHexagonTarget()
            }

            if config.info {
                LLVMInitializeHexagonTargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeHexagonAsmPrinter()
            }

            if config.disassembler {
                LLVMInitializeHexagonDisassembler()
            }

            if config.machine_code {
                LLVMInitializeHexagonTargetMC()
            }
        }
    }

    pub fn initialize_nvptx(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeNVPTXTarget, LLVMInitializeNVPTXTargetInfo, LLVMInitializeNVPTXTargetMC, LLVMInitializeNVPTXAsmPrinter};

        unsafe {
            if config.base {
                LLVMInitializeNVPTXTarget()
            }

            if config.info {
                LLVMInitializeNVPTXTargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeNVPTXAsmPrinter()
            }

            if config.machine_code {
                LLVMInitializeNVPTXTargetMC()
            }
        }
    }

    pub fn initialize_cpp_backend(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeCppBackendTarget, LLVMInitializeCppBackendTargetInfo, LLVMInitializeCppBackendTargetMC};

        unsafe {
            if config.base {
                LLVMInitializeCppBackendTarget()
            }

            if config.info {
                LLVMInitializeCppBackendTargetInfo()
            }

            if config.machine_code {
                LLVMInitializeCppBackendTargetMC()
            }
        }
    }

    pub fn initialize_msp430(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeMSP430Target, LLVMInitializeMSP430TargetInfo, LLVMInitializeMSP430TargetMC, LLVMInitializeMSP430AsmPrinter};

        unsafe {
            if config.base {
                LLVMInitializeMSP430Target()
            }

            if config.info {
                LLVMInitializeMSP430TargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeMSP430AsmPrinter()
            }

            if config.machine_code {
                LLVMInitializeMSP430TargetMC()
            }
        }
    }

    pub fn initialize_x_core(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeXCoreTarget, LLVMInitializeXCoreTargetInfo, LLVMInitializeXCoreTargetMC, LLVMInitializeXCoreDisassembler, LLVMInitializeXCoreAsmPrinter};

        unsafe {
            if config.base {
                LLVMInitializeXCoreTarget()
            }

            if config.info {
                LLVMInitializeXCoreTargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeXCoreAsmPrinter()
            }

            if config.disassembler {
                LLVMInitializeXCoreDisassembler()
            }

            if config.machine_code {
                LLVMInitializeXCoreTargetMC()
            }
        }
    }

    pub fn initialize_power_pc(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializePowerPCTarget, LLVMInitializePowerPCTargetInfo, LLVMInitializePowerPCTargetMC, LLVMInitializePowerPCDisassembler, LLVMInitializePowerPCAsmPrinter, LLVMInitializePowerPCAsmParser};

        unsafe {
            if config.base {
                LLVMInitializePowerPCTarget()
            }

            if config.info {
                LLVMInitializePowerPCTargetInfo()
            }

            if config.asm_printer {
                LLVMInitializePowerPCAsmPrinter()
            }

            if config.asm_parser {
                LLVMInitializePowerPCAsmParser()
            }

            if config.disassembler {
                LLVMInitializePowerPCDisassembler()
            }

            if config.machine_code {
                LLVMInitializePowerPCTargetMC()
            }
        }
    }

    pub fn initialize_sparc(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeSparcTarget, LLVMInitializeSparcTargetInfo, LLVMInitializeSparcTargetMC, LLVMInitializeSparcDisassembler, LLVMInitializeSparcAsmPrinter, LLVMInitializeSparcAsmParser};

        unsafe {
            if config.base {
                LLVMInitializeSparcTarget()
            }

            if config.info {
                LLVMInitializeSparcTargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeSparcAsmPrinter()
            }

            if config.asm_parser {
                LLVMInitializeSparcAsmParser()
            }

            if config.disassembler {
                LLVMInitializeSparcDisassembler()
            }

            if config.machine_code {
                LLVMInitializeSparcTargetMC()
            }
        }
    }

    // TODOC: Disassembler only supported in LLVM 4.0+
    #[cfg(not(feature = "llvm3-6"))]
    pub fn initialize_bpf(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeBPFTarget, LLVMInitializeBPFTargetInfo, LLVMInitializeBPFTargetMC, LLVMInitializeBPFAsmPrinter};

        unsafe {
            if config.base {
                LLVMInitializeBPFTarget()
            }

            if config.info {
                LLVMInitializeBPFTargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeBPFAsmPrinter()
            }

            // TODO: Added in 4.0
            // if config.disassembler {
            //     LLVMInitializeBPFDisassembler()
            // }

            if config.machine_code {
                LLVMInitializeBPFTargetMC()
            }
        }
    }

    pub fn initialize_native(config: &InitializationConfig) -> Result<(), String> {
        use llvm_sys::target::{LLVM_InitializeNativeTarget, LLVM_InitializeNativeAsmParser, LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeDisassembler};

        if config.base {
            let code = unsafe {
                LLVM_InitializeNativeTarget()
            };

            if code == 1 {
                return Err("Unknown error in initializing native target".into());
            }
        }

        if config.asm_printer {
            let code = unsafe {
                LLVM_InitializeNativeAsmPrinter()
            };

            if code == 1 {
                return Err("Unknown error in initializing native asm printer".into());
            }
        }

        if config.asm_parser {
            let code = unsafe {
                LLVM_InitializeNativeAsmParser()
            };

            if code == 1 { // REVIEW: Does parser need to go before printer?
                return Err("Unknown error in initializing native asm parser".into());
            }
        }

        if config.disassembler {
            let code = unsafe {
                LLVM_InitializeNativeDisassembler()
            };

            if code == 1 {
                return Err("Unknown error in initializing native disassembler".into());
            }
        }

        Ok(())
    }

    pub fn initialize_all(config: &InitializationConfig) {
        use llvm_sys::target::{LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargets, LLVM_InitializeAllTargetMCs, LLVM_InitializeAllAsmPrinters, LLVM_InitializeAllAsmParsers, LLVM_InitializeAllDisassemblers};

        unsafe {
            if config.base {
                LLVM_InitializeAllTargets()
            }

            if config.info {
                LLVM_InitializeAllTargetInfos()
            }

            if config.asm_parser {
                LLVM_InitializeAllAsmParsers()
            }

            if config.asm_printer {
                LLVM_InitializeAllAsmPrinters()
            }

            if config.disassembler {
                LLVM_InitializeAllDisassemblers()
            }

            if config.machine_code {
                LLVM_InitializeAllTargetMCs()
            }
        }
    }

    pub fn create_target_machine(&self, triple: &str, cpu: &str, features: &str, level: OptimizationLevel, reloc_mode: RelocMode, code_model: CodeModel) -> Option<TargetMachine> {
        let triple = CString::new(triple).expect("Conversion to CString failed unexpectedly");
        let cpu = CString::new(cpu).expect("Conversion to CString failed unexpectedly");
        let features = CString::new(features).expect("Conversion to CString failed unexpectedly");
        let level = match level {
            OptimizationLevel::None => LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            OptimizationLevel::Less => LLVMCodeGenOptLevel::LLVMCodeGenLevelLess,
            OptimizationLevel::Default => LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            OptimizationLevel::Aggressive => LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
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

        if target_machine.is_null() {
            return None;
        }

        Some(TargetMachine::new(target_machine))
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
        let mut target = ptr::null_mut();
        let mut err_str = unsafe { zeroed() };

        let code = unsafe {
            LLVMGetTargetFromTriple(c_string.as_ptr(), &mut target, &mut err_str)
        };

        if code == 1 { // REVIEW: 1 is error value
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(err_str);

                rust_str
            };

            return Err(rust_str);
        }

        Ok(Target::new(target))
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

#[derive(Debug)]
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

    // TODO: Move to PassManager?
    pub fn add_analysis_passes(&self, pass_manager: &PassManager) {
        unsafe {
            LLVMAddAnalysisPasses(self.target_machine, pass_manager.pass_manager)
        }
    }

    pub fn write_to_memory_buffer(&self, module: &Module, file_type: FileType) -> Result<MemoryBuffer, String> {
        let mut memory_buffer = ptr::null_mut();
        let mut err_str = unsafe { zeroed() };
        let return_code = unsafe {
            LLVMTargetMachineEmitToMemoryBuffer(self.target_machine, module.module.get(), file_type.as_llvm_file_type(), &mut err_str, &mut memory_buffer)
        };

        // TODO: Verify 1 is error code (LLVM can be inconsistent)
        if return_code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(err_str);

                rust_str
            };

            return Err(rust_str);
        }

        Ok(MemoryBuffer::new(memory_buffer))
    }

    pub fn write_to_file(&self, module: &Module, file_type: FileType, path: &Path) -> Result<(), String> {
        let path = path.to_str().expect("Did not find a valid Unicode path string");
        let mut err_str = unsafe { zeroed() };
        let return_code = unsafe {
            // REVIEW: Why does LLVM need a mutable reference to path...?
            LLVMTargetMachineEmitToFile(self.target_machine, module.module.get(), path.as_ptr() as *mut i8, file_type.as_llvm_file_type(), &mut err_str)
        };

        // TODO: Verify 1 is error code (LLVM can be inconsistent)
        if return_code == 1 {
            let rust_str = unsafe {
                let rust_str = CStr::from_ptr(err_str).to_string_lossy().into_owned();

                LLVMDisposeMessage(err_str);

                rust_str
            };

            return Err(rust_str);
        }

        Ok(())
    }
}

impl Drop for TargetMachine {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeTargetMachine(self.target_machine)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}

#[derive(PartialEq, Eq, Debug)]
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

    pub fn int_ptr_type(&self) -> PointerType {
        let ptr_type = unsafe {
            LLVMIntPtrType(self.target_data)
        };

        PointerType::new(ptr_type)
    }

    pub fn int_ptr_type_for_as(&self, as_: u32) -> PointerType {
        let ptr_type = unsafe {
            LLVMIntPtrTypeForAS(self.target_data, as_)
        };

        PointerType::new(ptr_type)
    }

    pub fn int_ptr_type_in_context(&self, context: &Context) -> PointerType {
        let ptr_type = unsafe {
            LLVMIntPtrTypeInContext(*context.context, self.target_data)
        };

        PointerType::new(ptr_type)
    }

    pub fn int_ptr_type_for_as_in_context(&self, context: &Context, as_: u32) -> PointerType {
        let ptr_type = unsafe {
            LLVMIntPtrTypeForASInContext(*context.context, self.target_data, as_)
        };

        PointerType::new(ptr_type)
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

    pub fn create(str_repr: &str) -> TargetData {
        let c_string = CString::new(str_repr).expect("Conversion to CString failed unexpectedly");

        let target_data = unsafe {
            LLVMCreateTargetData(c_string.as_ptr())
        };

        TargetData::new(target_data)
    }

    // REVIEW: Maybe this should be pass_manager.add_target_data()?
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

    pub fn get_pointer_byte_size_for_as(&self, as_: u32) -> u32 {
        unsafe {
            LLVMPointerSizeForAS(self.target_data, as_)
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

    pub fn get_preferred_alignment_of_global(&self, value: &GlobalValue) -> u32 {
        unsafe {
            LLVMPreferredAlignmentOfGlobal(self.target_data, value.as_value_ref())
        }
    }

    pub fn element_at_offset(&self, struct_type: &StructType, offset: u64) -> u32 {
        unsafe {
            LLVMElementAtOffset(self.target_data, struct_type.as_type_ref(), offset)
        }
    }

    pub fn offset_of_element(&self, struct_type: &StructType, element: u32) -> Option<u64> {
        if element > struct_type.count_fields() - 1 {
            return None;
        }

        unsafe {
            Some(LLVMOffsetOfElement(self.target_data, struct_type.as_type_ref(), element))
        }
    }
}

// FIXME: Make sure this doesn't SegFault:
impl Drop for TargetData {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeTargetData(self.target_data)
        }
    }
}
