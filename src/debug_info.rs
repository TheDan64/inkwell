//! Debug symbols
//!
//! Example usage:
//! ```ignore
//! let context = Context::create();
//! let module = context.create_module("bin");
//! let builder = context.create_builder();
//! let dibuilder = module.create_debug_info_builder(true);
//!
//! let compile_unit = dibuilder.create_compile_unit(
//!     inkwell::debug_info::LLVMDWARFSourceLanguage::LLVMDWARFSourceLanguageC,
//!     dibuilder.create_file(/* filename */ "bin", /* directory */ "."),
//!     /* producer */ "toy compiler",
//!     /* is_optimized */ false,
//!     /*flags*/ "",
//!     /* runtime_ver */ 0,
//!     /*split_name */ "",
//!     /* kind */ inkwell::debug_info::LLVMDWARFEmissionKind::LLVMDWARFEmissionKindFull,
//!     /*dwo_id */ 0,
//!     /* split_debug_inling */ false,
//!     /* debug_info_for_profiling */ false,
//! );
//! /* Create function */
//!
//!  let func_scope: DIScope<'ctx> = compile_unit.as_debug_info_scope();
//!  let ditype = dibuilder.create_basic_type(
//!      "type_name",
//!      0_u64,
//!      0x00,
//!      LLVMDIFlags::LLVMDIFlagPublic,
//!  );
//!  let subroutine_type = dibuilder.create_subroutine_type(
//!      compile_unit.get_file(),
//!      /* return type */ ditype,
//!      /* parameter types */ vec![],
//!      LLVMDIFlags::LLVMDIFlagPublic,
//!  );
//!  let ret = dibuilder.create_function(
//!      /* scope */ func_scope,
//!      /* func name */ "main",
//!      /* linkage_name */ None,
//!      /* file */ compile_unit.get_file(),
//!      /* line_no */ 0,
//!      /* DIType */ subroutine_type,
//!      /* is_local_to_unit */ true,
//!      /* is_definition */ false,
//!      /* scope_line */ 0,
//!      /* flags */ LLVMDIFlags::LLVMDIFlagPublic,
//!      /* is_optimized */ false,
//!  );
//!
//! let loc = dibuilder
//!     .create_debug_location(context, /*line*/ 0, /* column */ 0, scope);
//! builder.set_current_debug_location(context, loc);
//! ```
//!
//! Debug symbols are finalized when calling `Module::print_to_stderr`,
//! `Module::print_to_string` and `Module::print_to_file`.

use crate::context::Context;
use crate::module::Module;
use llvm_sys::debuginfo::{
    LLVMCreateDIBuilder, LLVMCreateDIBuilderDisallowUnresolved, LLVMDIBuilderCreateBasicType,
    LLVMDIBuilderCreateCompileUnit, LLVMDIBuilderCreateDebugLocation, LLVMDIBuilderCreateFile,
    LLVMDIBuilderCreateFunction, LLVMDIBuilderCreateSubroutineType, LLVMDIBuilderFinalize,
};
pub use llvm_sys::debuginfo::{
    LLVMDIFlags, LLVMDWARFEmissionKind, LLVMDWARFSourceLanguage, LLVMDWARFTypeEncoding,
};
use llvm_sys::debuginfo::{LLVMDebugMetadataVersion, LLVMDisposeDIBuilder};
use llvm_sys::prelude::LLVMDIBuilderRef;
use llvm_sys::prelude::LLVMMetadataRef;
use std::ffi::CString;
use std::marker::PhantomData;

/// Gets the version of debug metadata produced by the current LLVM version.
pub fn debug_metadata_version() -> libc::c_uint {
    unsafe { LLVMDebugMetadataVersion() }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DebugInfoBuilder<'ctx> {
    pub(crate) builder: LLVMDIBuilderRef,
    _marker: PhantomData<&'ctx Context>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DIScope<'ctx> {
    metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

pub trait AsDIScope<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx>;
}

impl<'ctx> DebugInfoBuilder<'ctx> {
    pub(crate) fn new(module: &Module, allow_unresolved: bool) -> Self {
        let builder = unsafe {
            if allow_unresolved {
                LLVMCreateDIBuilder(module.module.get())
            } else {
                LLVMCreateDIBuilderDisallowUnresolved(module.module.get())
            }
        };

        DebugInfoBuilder {
            builder,
            _marker: PhantomData,
        }
    }

    /// A DICompileUnit provides an anchor for all debugging information generated during this instance of compilation.
    ///
    /// * `language` - Source programming language
    /// * `file` - File info
    /// * `producer` - Identify the producer of debugging information and code. Usually this is a compiler version string.
    /// * `is_optimized` - A boolean flag which indicates whether optimization is enabled or not.
    /// * `flags` - This string lists command line options. This string is directly embedded in debug info output which may be used by a tool analyzing generated debugging information.
    /// * `runtime_ver` - This indicates runtime version for languages like Objective-C.
    /// * `split_name` - The name of the file that we'll split debug info out into.
    /// * `kind` - The kind of debug information to generate.
    /// * `dwo_id` - The DWOId if this is a split skeleton compile unit.
    /// * `split_debug_inlining` - Whether to emit inline debug info.
    /// * `debug_info_for_profiling` - Whether to emit extra debug info for profile collection.
    pub fn create_compile_unit(
        &self,
        language: LLVMDWARFSourceLanguage,
        file: DIFile<'ctx>,
        producer: &str,
        is_optimized: bool,
        flags: &str,
        runtime_ver: libc::c_uint,
        split_name: &str,
        kind: LLVMDWARFEmissionKind,
        dwo_id: libc::c_uint,
        split_debug_inlining: bool,
        debug_info_for_profiling: bool,
    ) -> DICompileUnit<'ctx> {
        let producer = CString::new(producer).expect("Conversion to CString failed unexpectedly");
        let flags = CString::new(flags).expect("Conversion to CString failed unexpectedly");
        let split_name =
            CString::new(split_name).expect("Conversion to CString failed unexpectedly");
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateCompileUnit(
                self.builder,
                language,
                file.metadata_ref,
                producer.as_ptr(),
                producer.as_bytes().len(),
                is_optimized as libc::c_int,
                flags.as_ptr(),
                flags.as_bytes().len(),
                runtime_ver,
                split_name.as_ptr(),
                split_name.as_bytes().len(),
                kind,
                dwo_id,
                split_debug_inlining as libc::c_int,
                debug_info_for_profiling as libc::c_int,
            )
        };

        DICompileUnit {
            file,
            metadata_ref,
            _marker: PhantomData,
        }
        /*
                 *pub unsafe extern "C" fn LLVMDIBuilderCreateCompileUnit(
            Builder: LLVMDIBuilderRef,
            Lang: LLVMDWARFSourceLanguage,
            FileRef: LLVMMetadataRef,
            Producer: *const c_char,
            ProducerLen: size_t,
            isOptimized: LLVMBool,
            Flags: *const c_char,
            FlagsLen: size_t,
            RuntimeVer: c_uint,
            SplitName: *const c_char,
            SplitNameLen: size_t,
            Kind: LLVMDWARFEmissionKind,
            DWOId: c_uint,
            SplitDebugInlining: LLVMBool,
            DebugInfoForProfiling: LLVMBool
        ) -> LLVMMetadataRef
                */
    }

    /// A DIFunction provides an anchor for all debugging information generated for the specified subprogram.
    ///
    /// * `scope` - Function scope.
    /// * `name` - Function name.
    /// * `linkage_name` - Mangled function name, if any.
    /// * `file` - File where this variable is defined.
    /// * `line_no` - Line number.
    /// * `ty` - Function type.
    /// * `is_local_to_unit` - True if this function is not externally visible.
    /// * `is_definition` - True if this is a function definition.
    /// * `scope_line` - Set to the beginning of the scope this starts
    /// * `flags` - E.g.: LLVMDIFlagLValueReference. These flags are used to emit dwarf attributes.
    /// * `is_optimized` - True if optimization is ON.
    pub fn create_function(
        &self,
        scope: DIScope<'ctx>,
        name: &str,
        linkage_name: Option<&str>,
        file: DIFile<'ctx>,
        line_no: u32,
        ditype: DISubroutineType<'ctx>,
        is_local_to_unit: bool,
        is_definition: bool,
        scope_line: u32,
        flags: LLVMDIFlags,
        is_optimized: bool,
    ) -> DISubProgram<'ctx> {
        let linkage_name = CString::new(linkage_name.unwrap_or(name))
            .expect("Conversion to CString failed unexpectedly");
        let name = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let metadata_ref = unsafe {
            LLVMDIBuilderCreateFunction(
                self.builder,
                scope.metadata_ref,
                name.as_ptr(),
                name.as_bytes().len(),
                linkage_name.as_ptr(),
                linkage_name.as_bytes().len(),
                file.metadata_ref,
                line_no,
                ditype.metadata_ref,
                is_local_to_unit as libc::c_int,
                is_definition as libc::c_int,
                scope_line as libc::c_uint,
                flags,
                is_optimized as libc::c_int,
            )
        };
        DISubProgram {
            metadata_ref,
            _marker: PhantomData,
        }
        /*
        pub unsafe extern "C" fn LLVMDIBuilderCreateFunction(
            Builder: LLVMDIBuilderRef,
            Scope: LLVMMetadataRef,
            Name: *const c_char,
            NameLen: size_t,
            LinkageName: *const c_char,
            LinkageNameLen: size_t,
            File: LLVMMetadataRef,
            LineNo: c_uint,
            Ty: LLVMMetadataRef,
            IsLocalToUnit: LLVMBool,
            IsDefinition: LLVMBool,
            ScopeLine: c_uint,
            Flags: LLVMDIFlags,
            IsOptimized: LLVMBool
        ) -> LLVMMetadataRef
         *
         */
    }

    pub fn create_file(&self, filename: &str, directory: &str) -> DIFile<'ctx> {
        let filename = CString::new(filename).expect("Conversion to CString failed unexpectedly");
        let directory = CString::new(directory).expect("Conversion to CString failed unexpectedly");
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateFile(
                self.builder,
                filename.as_ptr(),
                filename.as_bytes().len(),
                directory.as_ptr(),
                directory.as_bytes().len(),
            )
        };
        DIFile {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    pub fn create_debug_location(
        &self,
        context: &Context,
        line: u32,
        column: u32,
        scope: DIScope<'ctx>,
    ) -> DebugLocation<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateDebugLocation(
                context.context,
                line,
                column,
                scope.metadata_ref,
                std::ptr::null_mut(),
            )
        };
        /*
        LLVMDIBuilderCreateDebugLocation(
            Ctx: LLVMContextRef,
            Line: c_uint,
            Column: c_uint,
            Scope: LLVMMetadataRef,
            InlinedAt: LLVMMetadataRef
        ) -> LLVMMetadataRef
            */
        DebugLocation {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// * `encoding` -
    /// ```ignore
    /// const DW_LANG_RUST: c_uint = 0x1c;
    /// #[allow(non_upper_case_globals)]
    /// const DW_ATE_boolean: c_uint = 0x02;
    /// #[allow(non_upper_case_globals)]
    /// const DW_ATE_float: c_uint = 0x04;
    /// #[allow(non_upper_case_globals)]
    /// const DW_ATE_signed: c_uint = 0x05;
    /// #[allow(non_upper_case_globals)]
    /// const DW_ATE_unsigned: c_uint = 0x07;
    /// #[allow(non_upper_case_globals)]
    /// const DW_ATE_unsigned_char: c_uint = 0x08;
    /// ```
    pub fn create_basic_type(
        &self,
        name: &str,
        size_in_bits: u64,
        encoding: LLVMDWARFTypeEncoding,
        flags: LLVMDIFlags,
    ) -> DIType<'ctx> {
        let name = CString::new(name).expect("Conversion to CString failed unexpectedly");
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateBasicType(
                self.builder,
                name.as_ptr(),
                name.as_bytes().len(),
                size_in_bits,
                encoding,
                flags,
            )
        };
        /*
        pub unsafe extern "C" fn LLVMDIBuilderCreateBasicType(
            Builder: LLVMDIBuilderRef,
            Name: *const c_char,
            NameLen: size_t,
            SizeInBits: u64,
            Encoding: LLVMDWARFTypeEncoding,
            Flags: LLVMDIFlags
        ) -> LLVMMetadataRef
                    */
        DIType {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    pub fn create_subroutine_type(
        &self,
        file: DIFile<'ctx>,
        return_type: DIType<'ctx>,
        parameter_types: Vec<DIType<'ctx>>,
        flags: LLVMDIFlags,
    ) -> DISubroutineType<'ctx> {
        use std::convert::TryInto;
        let mut p = [return_type]
            .iter()
            .map(|t| t.metadata_ref)
            .chain(parameter_types.into_iter().map(|t| t.metadata_ref))
            .collect::<Vec<LLVMMetadataRef>>();
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateSubroutineType(
                self.builder,
                file.metadata_ref,
                p.as_mut_ptr(),
                p.len().try_into().unwrap(),
                flags,
            )
        };
        /*
        pub unsafe extern "C" fn LLVMDIBuilderCreateSubroutineType(
            Builder: LLVMDIBuilderRef,
            File: LLVMMetadataRef,
            ParameterTypes: *mut LLVMMetadataRef,
            NumParameterTypes: c_uint,
            Flags: LLVMDIFlags
        ) -> LLVMMetadataRef
                            */
        DISubroutineType {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    pub(crate) fn finalize(self) {
        unsafe { LLVMDIBuilderFinalize(self.builder) };
        self.dispose();
    }

    fn dispose(self) {
        unsafe { LLVMDisposeDIBuilder(self.builder) }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIFile<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> AsDIScope<'ctx> for DIFile<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DICompileUnit<'ctx> {
    file: DIFile<'ctx>,
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DICompileUnit<'ctx> {
    pub fn get_file(&self) -> DIFile<'ctx> {
        self.file
    }
}

impl<'ctx> AsDIScope<'ctx> for DICompileUnit<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DISubProgram<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> AsDIScope<'ctx> for DISubProgram<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DISubroutineType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DebugLocation<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}
