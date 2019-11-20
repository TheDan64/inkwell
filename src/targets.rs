#[llvm_versions(7.0..=latest)]
use either::Either;
use llvm_sys::target::{
    LLVMABIAlignmentOfType, LLVMABISizeOfType, LLVMByteOrder, LLVMByteOrdering,
    LLVMCallFrameAlignmentOfType, LLVMCopyStringRepOfTargetData, LLVMCreateTargetData,
    LLVMDisposeTargetData, LLVMElementAtOffset,
    LLVMIntPtrTypeForASInContext, LLVMIntPtrTypeInContext, LLVMOffsetOfElement, LLVMPointerSize,
    LLVMPointerSizeForAS, LLVMPreferredAlignmentOfGlobal, LLVMPreferredAlignmentOfType,
    LLVMSizeOfTypeInBits, LLVMStoreSizeOfType, LLVMTargetDataRef,
};
#[llvm_versions(4.0..=latest)]
use llvm_sys::target_machine::LLVMCreateTargetDataLayout;
use llvm_sys::target_machine::{
    LLVMAddAnalysisPasses, LLVMCodeGenFileType, LLVMCodeGenOptLevel, LLVMCodeModel,
    LLVMCreateTargetMachine, LLVMDisposeTargetMachine, LLVMGetDefaultTargetTriple,
    LLVMGetFirstTarget, LLVMGetNextTarget, LLVMGetTargetDescription, LLVMGetTargetFromName,
    LLVMGetTargetFromTriple, LLVMGetTargetMachineCPU, LLVMGetTargetMachineFeatureString,
    LLVMGetTargetMachineTarget, LLVMGetTargetMachineTriple, LLVMGetTargetName, LLVMRelocMode,
    LLVMSetTargetMachineAsmVerbosity, LLVMTargetHasAsmBackend, LLVMTargetHasJIT,
    LLVMTargetHasTargetMachine, LLVMTargetMachineEmitToFile, LLVMTargetMachineEmitToMemoryBuffer,
    LLVMTargetMachineRef, LLVMTargetRef,
};
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::context::Context;
use crate::data_layout::DataLayout;
use crate::memory_buffer::MemoryBuffer;
use crate::module::Module;
use crate::passes::PassManager;
use crate::support::LLVMString;
use crate::types::{AnyType, AsTypeRef, IntType, StructType};
use crate::values::{AsValueRef, GlobalValue};
use crate::{AddressSpace, OptimizationLevel};

use std::default::Default;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

static TARGET_LOCK: Lazy<RwLock<()>> = Lazy::new(|| RwLock::new(()));

// NOTE: Versions verified as target-complete: 3.6, 3.7, 3.8, 3.9, 4.0
#[derive(Debug, Eq, PartialEq)]
pub struct Target {
    target: LLVMTargetRef,
}

impl Target {
    fn new(target: LLVMTargetRef) -> Self {
        assert!(!target.is_null());

        Target { target }
    }

    // REVIEW: Should this just initialize all? Is opt into each a good idea?
    #[cfg(feature = "target-x86")]
    pub fn initialize_x86(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeX86AsmParser, LLVMInitializeX86AsmPrinter, LLVMInitializeX86Disassembler,
            LLVMInitializeX86Target, LLVMInitializeX86TargetInfo, LLVMInitializeX86TargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeX86Target() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeX86TargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeX86AsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeX86AsmParser() };
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeX86Disassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeX86TargetMC() };
        }
    }

    #[cfg(feature = "target-arm")]
    pub fn initialize_arm(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeARMAsmParser, LLVMInitializeARMAsmPrinter, LLVMInitializeARMDisassembler,
            LLVMInitializeARMTarget, LLVMInitializeARMTargetInfo, LLVMInitializeARMTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeARMTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeARMTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeARMAsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeARMAsmParser() };
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeARMDisassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeARMTargetMC() };
        }
    }

    #[cfg(feature = "target-mips")]
    pub fn initialize_mips(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeMipsAsmParser, LLVMInitializeMipsAsmPrinter,
            LLVMInitializeMipsDisassembler, LLVMInitializeMipsTarget, LLVMInitializeMipsTargetInfo,
            LLVMInitializeMipsTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeMipsTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeMipsTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeMipsAsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeMipsAsmParser() };
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeMipsDisassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeMipsTargetMC() };
        }
    }

    #[cfg(feature = "target-aarch64")]
    pub fn initialize_aarch64(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeAArch64AsmParser, LLVMInitializeAArch64AsmPrinter,
            LLVMInitializeAArch64Disassembler, LLVMInitializeAArch64Target,
            LLVMInitializeAArch64TargetInfo, LLVMInitializeAArch64TargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAArch64Target() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAArch64TargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAArch64AsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAArch64AsmParser() };
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAArch64Disassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAArch64TargetMC() };
        }
    }

    // TODOC: Called AMDGPU in 3.7+
    #[cfg(feature = "llvm3-6")]
    pub fn initialize_r600(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeR600AsmParser, LLVMInitializeR600AsmPrinter, LLVMInitializeR600Target,
            LLVMInitializeR600TargetInfo, LLVMInitializeR600TargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeR600Target() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeR600TargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeR600AsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeR600AsmParser() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeR600TargetMC() };
        }

        // Disassembler Status Unknown
    }

    // TODOC: Called R600 in 3.6
    #[cfg(feature = "target-amdgpu")]
    #[llvm_versions(3.7..=latest)]
    pub fn initialize_amd_gpu(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeAMDGPUAsmParser, LLVMInitializeAMDGPUAsmPrinter,
            LLVMInitializeAMDGPUTarget, LLVMInitializeAMDGPUTargetInfo,
            LLVMInitializeAMDGPUTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAMDGPUTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAMDGPUTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAMDGPUAsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAMDGPUAsmParser() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeAMDGPUTargetMC() };
        }

        // Disassembler Status Unknown
    }

    #[cfg(feature = "target-systemz")]
    pub fn initialize_system_z(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeSystemZAsmParser, LLVMInitializeSystemZAsmPrinter,
            LLVMInitializeSystemZDisassembler, LLVMInitializeSystemZTarget,
            LLVMInitializeSystemZTargetInfo, LLVMInitializeSystemZTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSystemZTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSystemZTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSystemZAsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSystemZAsmParser() };
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSystemZDisassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSystemZTargetMC() };
        }
    }

    #[cfg(feature = "target-hexagon")]
    pub fn initialize_hexagon(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeHexagonAsmPrinter, LLVMInitializeHexagonDisassembler,
            LLVMInitializeHexagonTarget, LLVMInitializeHexagonTargetInfo,
            LLVMInitializeHexagonTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeHexagonTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeHexagonTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeHexagonAsmPrinter() };
        }

        // Asm parser status unknown

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeHexagonDisassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeHexagonTargetMC() };
        }
    }

    #[cfg(feature = "target-nvptx")]
    pub fn initialize_nvptx(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeNVPTXAsmPrinter, LLVMInitializeNVPTXTarget,
            LLVMInitializeNVPTXTargetInfo, LLVMInitializeNVPTXTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeNVPTXTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeNVPTXTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeNVPTXAsmPrinter() };
        }

        // Asm parser status unknown

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeNVPTXTargetMC() };
        }

        // Disassembler status unknown
    }

    #[llvm_versions(3.6..=3.8)]
    pub fn initialize_cpp_backend(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeCppBackendTarget, LLVMInitializeCppBackendTargetInfo,
            LLVMInitializeCppBackendTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeCppBackendTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeCppBackendTargetInfo() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeCppBackendTargetMC() };
        }
    }

    #[cfg(feature = "target-msp430")]
    pub fn initialize_msp430(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeMSP430AsmPrinter, LLVMInitializeMSP430Target,
            LLVMInitializeMSP430TargetInfo, LLVMInitializeMSP430TargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeMSP430Target() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeMSP430TargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeMSP430AsmPrinter() };
        }

        // Asm parser status unknown

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeMSP430TargetMC() };
        }

        // Disassembler status unknown
    }

    #[cfg(feature = "target-xcore")]
    pub fn initialize_x_core(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeXCoreAsmPrinter, LLVMInitializeXCoreDisassembler,
            LLVMInitializeXCoreTarget, LLVMInitializeXCoreTargetInfo, LLVMInitializeXCoreTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeXCoreTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeXCoreTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeXCoreAsmPrinter() };
        }

        // Asm parser status unknown

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeXCoreDisassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeXCoreTargetMC() };
        }
    }

    #[cfg(feature = "target-powerpc")]
    pub fn initialize_power_pc(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializePowerPCAsmParser, LLVMInitializePowerPCAsmPrinter,
            LLVMInitializePowerPCDisassembler, LLVMInitializePowerPCTarget,
            LLVMInitializePowerPCTargetInfo, LLVMInitializePowerPCTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializePowerPCTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializePowerPCTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializePowerPCAsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializePowerPCAsmParser() };
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializePowerPCDisassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializePowerPCTargetMC() };
        }
    }

    #[cfg(feature = "target-sparc")]
    pub fn initialize_sparc(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeSparcAsmParser, LLVMInitializeSparcAsmPrinter,
            LLVMInitializeSparcDisassembler, LLVMInitializeSparcTarget,
            LLVMInitializeSparcTargetInfo, LLVMInitializeSparcTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSparcTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSparcTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSparcAsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSparcAsmParser() };
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSparcDisassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeSparcTargetMC() };
        }
    }

    // TODOC: Disassembler only supported in LLVM 4.0+
    #[cfg(feature = "target-bpf")]
    #[llvm_versions(3.7..=latest)]
    pub fn initialize_bpf(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeBPFAsmPrinter, LLVMInitializeBPFTarget, LLVMInitializeBPFTargetInfo,
            LLVMInitializeBPFTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeBPFTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeBPFTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeBPFAsmPrinter() };
        }

        // No asm parser

        #[cfg(not(any(feature = "llvm3-7", feature = "llvm3-8", feature = "llvm3-9")))]
        {
            if config.disassembler {
                use llvm_sys::target::LLVMInitializeBPFDisassembler;

                let _guard = TARGET_LOCK.write();
                unsafe { LLVMInitializeBPFDisassembler() };
            }
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeBPFTargetMC() };
        }
    }

    #[cfg(feature = "target-lanai")]
    #[llvm_versions(4.0..=latest)]
    pub fn initialize_lanai(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeLanaiAsmParser, LLVMInitializeLanaiAsmPrinter,
            LLVMInitializeLanaiDisassembler, LLVMInitializeLanaiTarget,
            LLVMInitializeLanaiTargetInfo, LLVMInitializeLanaiTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeLanaiTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeLanaiTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeLanaiAsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeLanaiAsmParser() };
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeLanaiDisassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeLanaiTargetMC() };
        }
    }

    // REVIEW: As it turns out; RISCV was accidentally built by default in 4.0 since
    // it was meant to be marked experimental and so it was later removed from default
    // builds in 5.0+. Since llvm-sys doesn't officially support any experimental targets
    // we're going to make this 4.0 only for now so that it doesn't break test builds.
    // We can revisit this issue if someone wants RISCV support in inkwell, or if
    // llvm-sys starts supporting experimental llvm targets. See
    // https://lists.llvm.org/pipermail/llvm-dev/2017-August/116347.html for more info
    #[llvm_versions(4.0)]
    pub fn initialize_riscv(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeRISCVTarget, LLVMInitializeRISCVTargetInfo, LLVMInitializeRISCVTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeRISCVTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeRISCVTargetInfo() };
        }

        // No asm printer

        // No asm parser

        // No disassembler

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeRISCVTargetMC() };
        }
    }

    #[cfg(feature = "target-webassembly")]
    #[llvm_versions(8.0..=latest)]
    pub fn initialize_webassembly(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVMInitializeWebAssemblyAsmParser, LLVMInitializeWebAssemblyAsmPrinter,
            LLVMInitializeWebAssemblyDisassembler, LLVMInitializeWebAssemblyTarget,
            LLVMInitializeWebAssemblyTargetInfo, LLVMInitializeWebAssemblyTargetMC,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeWebAssemblyTarget() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeWebAssemblyTargetInfo() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeWebAssemblyAsmPrinter() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeWebAssemblyAsmParser() };
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeWebAssemblyDisassembler() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVMInitializeWebAssemblyTargetMC() };
        }
    }

    pub fn initialize_native(config: &InitializationConfig) -> Result<(), String> {
        use llvm_sys::target::{
            LLVM_InitializeNativeAsmParser, LLVM_InitializeNativeAsmPrinter,
            LLVM_InitializeNativeDisassembler, LLVM_InitializeNativeTarget,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            let code = unsafe { LLVM_InitializeNativeTarget() };

            if code == 1 {
                return Err("Unknown error in initializing native target".into());
            }
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            let code = unsafe { LLVM_InitializeNativeAsmPrinter() };

            if code == 1 {
                return Err("Unknown error in initializing native asm printer".into());
            }
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            let code = unsafe { LLVM_InitializeNativeAsmParser() };

            if code == 1 {
                // REVIEW: Does parser need to go before printer?
                return Err("Unknown error in initializing native asm parser".into());
            }
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            let code = unsafe { LLVM_InitializeNativeDisassembler() };

            if code == 1 {
                return Err("Unknown error in initializing native disassembler".into());
            }
        }

        Ok(())
    }

    pub fn initialize_all(config: &InitializationConfig) {
        use llvm_sys::target::{
            LLVM_InitializeAllAsmParsers, LLVM_InitializeAllAsmPrinters,
            LLVM_InitializeAllDisassemblers, LLVM_InitializeAllTargetInfos,
            LLVM_InitializeAllTargetMCs, LLVM_InitializeAllTargets,
        };

        if config.base {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVM_InitializeAllTargets() };
        }

        if config.info {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVM_InitializeAllTargetInfos() };
        }

        if config.asm_parser {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVM_InitializeAllAsmParsers() };
        }

        if config.asm_printer {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVM_InitializeAllAsmPrinters() };
        }

        if config.disassembler {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVM_InitializeAllDisassemblers() };
        }

        if config.machine_code {
            let _guard = TARGET_LOCK.write();
            unsafe { LLVM_InitializeAllTargetMCs() };
        }
    }

    pub fn create_target_machine(
        &self,
        triple: &str,
        cpu: &str,
        features: &str,
        level: OptimizationLevel,
        reloc_mode: RelocMode,
        code_model: CodeModel,
    ) -> Option<TargetMachine> {
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
            LLVMCreateTargetMachine(
                self.target,
                triple.as_ptr(),
                cpu.as_ptr(),
                features.as_ptr(),
                level,
                reloc_mode,
                code_model,
            )
        };

        if target_machine.is_null() {
            return None;
        }

        Some(TargetMachine::new(target_machine))
    }

    pub fn get_first() -> Option<Self> {
        let target = {
            let _guard = TARGET_LOCK.read();
            unsafe { LLVMGetFirstTarget() }
        };

        if target.is_null() {
            return None;
        }

        Some(Target::new(target))
    }

    pub fn get_next(&self) -> Option<Self> {
        let target = unsafe { LLVMGetNextTarget(self.target) };

        if target.is_null() {
            return None;
        }

        Some(Target::new(target))
    }

    pub fn get_name(&self) -> &CStr {
        unsafe { CStr::from_ptr(LLVMGetTargetName(self.target)) }
    }

    pub fn get_description(&self) -> &CStr {
        unsafe { CStr::from_ptr(LLVMGetTargetDescription(self.target)) }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        Self::from_name_raw(c_string.as_ptr())
    }

    pub(crate) fn from_name_raw(c_string: *const ::libc::c_char) -> Option<Self> {
        let target = {
            let _guard = TARGET_LOCK.read();
            unsafe { LLVMGetTargetFromName(c_string) }
        };

        if target.is_null() {
            return None;
        }

        Some(Target::new(target))
    }

    pub fn from_triple(triple: &str) -> Result<Self, LLVMString> {
        let c_string = CString::new(triple).expect("Conversion to CString failed unexpectedly");
        let mut target = ptr::null_mut();
        let mut err_string = MaybeUninit::uninit();

        let code = {
            let _guard = TARGET_LOCK.read();
            unsafe { LLVMGetTargetFromTriple(c_string.as_ptr(), &mut target, err_string.as_mut_ptr()) }
        };

        if code == 1 {
            let err_string = unsafe { err_string.assume_init() };
            return Err(LLVMString::new(err_string));
        }

        Ok(Target::new(target))
    }

    pub fn has_jit(&self) -> bool {
        unsafe { LLVMTargetHasJIT(self.target) == 1 }
    }

    pub fn has_target_machine(&self) -> bool {
        unsafe { LLVMTargetHasTargetMachine(self.target) == 1 }
    }

    pub fn has_asm_backend(&self) -> bool {
        unsafe { LLVMTargetHasAsmBackend(self.target) == 1 }
    }
}

#[derive(Debug)]
pub struct TargetMachine {
    target_machine: LLVMTargetMachineRef,
}

impl TargetMachine {
    fn new(target_machine: LLVMTargetMachineRef) -> Self {
        assert!(!target_machine.is_null());

        TargetMachine { target_machine }
    }

    pub fn get_target(&self) -> Target {
        let target = unsafe { LLVMGetTargetMachineTarget(self.target_machine) };

        Target::new(target)
    }

    pub fn get_triple(&self) -> LLVMString {
        let ptr = unsafe { LLVMGetTargetMachineTriple(self.target_machine) };

        LLVMString::new(ptr)
    }

    /// Gets the default triple for the current system.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::targets::TargetMachine;
    ///
    /// use std::ffi::CString;
    ///
    /// let default_triple = TargetMachine::get_default_triple();
    ///
    /// assert_eq!(*default_triple, *CString::new("x86_64-pc-linux-gnu").unwrap());
    /// ```
    pub fn get_default_triple() -> LLVMString {
        let llvm_string = unsafe { LLVMGetDefaultTargetTriple() };

        LLVMString::new(llvm_string)
    }

    #[llvm_versions(7.0..=latest)]
    pub fn normalize_target_triple(triple: Either<&str, &CStr>) -> LLVMString {
        use llvm_sys::target_machine::LLVMNormalizeTargetTriple;

        let ptr = match triple {
            Either::Left(triple_str) => {
                let c_string =
                    CString::new(triple_str).expect("Conversion to CString failed unexpectedly");

                unsafe { LLVMNormalizeTargetTriple(c_string.as_ptr()) }
            }
            Either::Right(triple_cstr) => unsafe {
                LLVMNormalizeTargetTriple(triple_cstr.as_ptr())
            },
        };

        LLVMString::new(ptr)
    }

    /// Gets a string containing the host CPU's name (triple).
    ///
    /// # Example Output
    ///
    /// `x86_64-pc-linux-gnu`
    #[llvm_versions(7.0..=latest)]
    pub fn get_host_cpu_name() -> LLVMString {
        use llvm_sys::target_machine::LLVMGetHostCPUName;

        let ptr = unsafe { LLVMGetHostCPUName() };

        LLVMString::new(ptr)
    }

    /// Gets a comma separated list of supported features by the host CPU.
    ///
    /// # Example Output
    ///
    /// `+sse2,+cx16,+sahf,-tbm`
    #[llvm_versions(7.0..=latest)]
    pub fn get_host_cpu_features() -> LLVMString {
        use llvm_sys::target_machine::LLVMGetHostCPUFeatures;

        let ptr = unsafe { LLVMGetHostCPUFeatures() };

        LLVMString::new(ptr)
    }

    pub fn get_cpu(&self) -> LLVMString {
        let ptr = unsafe { LLVMGetTargetMachineCPU(self.target_machine) };

        LLVMString::new(ptr)
    }

    pub fn get_feature_string(&self) -> &CStr {
        unsafe { CStr::from_ptr(LLVMGetTargetMachineFeatureString(self.target_machine)) }
    }

    /// Create TargetData from this target machine
    #[llvm_versions(4.0..=latest)]
    pub fn get_target_data(&self) -> TargetData {
        let data_layout = unsafe { LLVMCreateTargetDataLayout(self.target_machine) };

        TargetData::new(data_layout)
    }

    pub fn set_asm_verbosity(&self, verbosity: bool) {
        unsafe { LLVMSetTargetMachineAsmVerbosity(self.target_machine, verbosity as i32) }
    }

    // TODO: Move to PassManager?
    pub fn add_analysis_passes<T>(&self, pass_manager: &PassManager<T>) {
        unsafe { LLVMAddAnalysisPasses(self.target_machine, pass_manager.pass_manager) }
    }

    /// Writes a `TargetMachine` to a `MemoryBuffer`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::OptimizationLevel;
    /// use inkwell::context::Context;
    /// use inkwell::targets::{CodeModel, RelocMode, FileType, Target, TargetMachine, InitializationConfig};
    ///
    /// Target::initialize_x86(&InitializationConfig::default());
    ///
    /// let opt = OptimizationLevel::Default;
    /// let reloc = RelocMode::Default;
    /// let model = CodeModel::Default;
    /// let target = Target::from_name("x86-64").unwrap();
    /// let target_machine = target.create_target_machine("x86_64-pc-linux-gnu", "x86-64", "+avx2", opt, reloc, model).unwrap();
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    ///
    /// module.add_function("my_fn", fn_type, None);
    ///
    /// let buffer = target_machine.write_to_memory_buffer(&module, FileType::Assembly).unwrap();
    /// ```
    pub fn write_to_memory_buffer(
        &self,
        module: &Module,
        file_type: FileType,
    ) -> Result<MemoryBuffer, LLVMString> {
        let mut memory_buffer = ptr::null_mut();
        let mut err_string = MaybeUninit::uninit();
        let return_code = unsafe {
            let module_ptr = module.module.get();
            let file_type_ptr = file_type.as_llvm_file_type();

            LLVMTargetMachineEmitToMemoryBuffer(
                self.target_machine,
                module_ptr,
                file_type_ptr,
                err_string.as_mut_ptr(),
                &mut memory_buffer,
            )
        };

        if return_code == 1 {
            let err_string = unsafe { err_string.assume_init() };
            return Err(LLVMString::new(err_string));
        }

        Ok(MemoryBuffer::new(memory_buffer))
    }

    /// Saves a `TargetMachine` to a file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inkwell::OptimizationLevel;
    /// use inkwell::context::Context;
    /// use inkwell::targets::{CodeModel, RelocMode, FileType, Target, TargetMachine, InitializationConfig};
    ///
    /// use std::path::Path;
    ///
    /// Target::initialize_x86(&InitializationConfig::default());
    ///
    /// let opt = OptimizationLevel::Default;
    /// let reloc = RelocMode::Default;
    /// let model = CodeModel::Default;
    /// let path = Path::new("/tmp/some/path/main.asm");
    /// let target = Target::from_name("x86-64").unwrap();
    /// let target_machine = target.create_target_machine("x86_64-pc-linux-gnu", "x86-64", "+avx2", opt, reloc, model).unwrap();
    ///
    /// let context = Context::create();
    /// let module = context.create_module("my_module");
    /// let void_type = context.void_type();
    /// let fn_type = void_type.fn_type(&[], false);
    ///
    /// module.add_function("my_fn", fn_type, None);
    ///
    /// assert!(target_machine.write_to_file(&module, FileType::Object, &path).is_ok());
    /// ```
    pub fn write_to_file(
        &self,
        module: &Module,
        file_type: FileType,
        path: &Path,
    ) -> Result<(), LLVMString> {
        let path = path
            .to_str()
            .expect("Did not find a valid Unicode path string");
        let path_c_string = CString::new(path).expect("Conversion to CString failed unexpectedly");
        let mut err_string = MaybeUninit::uninit();
        let return_code = unsafe {
            // REVIEW: Why does LLVM need a mutable ptr to path...?
            let module_ptr = module.module.get();
            let path_ptr = path_c_string.as_ptr() as *mut _;
            let file_type_ptr = file_type.as_llvm_file_type();

            LLVMTargetMachineEmitToFile(
                self.target_machine,
                module_ptr,
                path_ptr,
                file_type_ptr,
                err_string.as_mut_ptr(),
            )
        };

        if return_code == 1 {
            let err_string = unsafe { err_string.assume_init() };
            return Err(LLVMString::new(err_string));
        }

        Ok(())
    }
}

impl Drop for TargetMachine {
    fn drop(&mut self) {
        unsafe { LLVMDisposeTargetMachine(self.target_machine) }
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
            target_data: target_data,
        }
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
    /// let target_data = execution_engine.get_target_data();
    /// let int_type = target_data.ptr_sized_int_type_in_context(&context, None);
    /// ```
    pub fn ptr_sized_int_type_in_context(
        &self,
        context: &Context,
        address_space: Option<AddressSpace>,
    ) -> IntType {
        let int_type_ptr = match address_space {
            Some(address_space) => unsafe {
                LLVMIntPtrTypeForASInContext(
                    context.context,
                    self.target_data,
                    address_space as u32,
                )
            },
            None => unsafe { LLVMIntPtrTypeInContext(context.context, self.target_data) },
        };

        IntType::new(int_type_ptr)
    }

    pub fn get_data_layout(&self) -> DataLayout {
        let data_layout = unsafe { LLVMCopyStringRepOfTargetData(self.target_data) };

        DataLayout::new_owned(data_layout)
    }

    // REVIEW: Does this only work if Sized?
    pub fn get_bit_size(&self, type_: &dyn AnyType) -> u64 {
        unsafe { LLVMSizeOfTypeInBits(self.target_data, type_.as_type_ref()) }
    }

    // TODOC: This can fail on LLVM's side(exit?), but it doesn't seem like we have any way to check this in rust
    pub fn create(str_repr: &str) -> TargetData {
        let c_string = CString::new(str_repr).expect("Conversion to CString failed unexpectedly");

        let target_data = unsafe { LLVMCreateTargetData(c_string.as_ptr()) };

        TargetData::new(target_data)
    }

    pub fn get_byte_ordering(&self) -> ByteOrdering {
        let byte_ordering = unsafe { LLVMByteOrder(self.target_data) };

        match byte_ordering {
            LLVMByteOrdering::LLVMBigEndian => ByteOrdering::BigEndian,
            LLVMByteOrdering::LLVMLittleEndian => ByteOrdering::LittleEndian,
        }
    }

    pub fn get_pointer_byte_size(&self, address_space: Option<AddressSpace>) -> u32 {
        match address_space {
            Some(address_space) => unsafe {
                LLVMPointerSizeForAS(self.target_data, address_space as u32)
            },
            None => unsafe { LLVMPointerSize(self.target_data) },
        }
    }

    pub fn get_store_size(&self, type_: &dyn AnyType) -> u64 {
        unsafe { LLVMStoreSizeOfType(self.target_data, type_.as_type_ref()) }
    }

    pub fn get_abi_size(&self, type_: &dyn AnyType) -> u64 {
        unsafe { LLVMABISizeOfType(self.target_data, type_.as_type_ref()) }
    }

    pub fn get_abi_alignment(&self, type_: &dyn AnyType) -> u32 {
        unsafe { LLVMABIAlignmentOfType(self.target_data, type_.as_type_ref()) }
    }

    pub fn get_call_frame_alignment(&self, type_: &dyn AnyType) -> u32 {
        unsafe { LLVMCallFrameAlignmentOfType(self.target_data, type_.as_type_ref()) }
    }

    pub fn get_preferred_alignment(&self, type_: &dyn AnyType) -> u32 {
        unsafe { LLVMPreferredAlignmentOfType(self.target_data, type_.as_type_ref()) }
    }

    pub fn get_preferred_alignment_of_global(&self, value: &GlobalValue) -> u32 {
        unsafe { LLVMPreferredAlignmentOfGlobal(self.target_data, value.as_value_ref()) }
    }

    pub fn element_at_offset(&self, struct_type: &StructType, offset: u64) -> u32 {
        unsafe { LLVMElementAtOffset(self.target_data, struct_type.as_type_ref(), offset) }
    }

    pub fn offset_of_element(&self, struct_type: &StructType, element: u32) -> Option<u64> {
        if element > struct_type.count_fields() - 1 {
            return None;
        }

        unsafe {
            Some(LLVMOffsetOfElement(
                self.target_data,
                struct_type.as_type_ref(),
                element,
            ))
        }
    }
}

impl Drop for TargetData {
    fn drop(&mut self) {
        unsafe { LLVMDisposeTargetData(self.target_data) }
    }
}
