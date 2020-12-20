//! Debug symbols - `DebugInfoBuilder` interface
//!
//! # Example usage
//!
//! ## Setting up the module for holding debug info:
//! ```ignore
//! let context = Context::create();
//! let module = context.create_module("bin");
//!
//! let debug_metadata_version = context.i32_type().const_int(3, false);
//! module.add_basic_value_flag(
//!     "Debug Info Version",
//!     inkwell::module::FlagBehavior::Warning,
//!     debug_metadata_version,
//! );
//! let builder = context.create_builder();
//! let (dibuilder, compile_unit) = module.create_debug_info_builder(
//!     true,
//!     /* language */ inkwell::debug_info::DWARFSourceLanguage::C,
//!     /* filename */ "source_file",
//!     /* directory */ ".",
//!     /* producer */ "my llvm compiler frontend",
//!     /* is_optimized */ false,
//!     /* compiler command line flags */ "",
//!     /* runtime_ver */ 0,
//!     /* split_name */ "",
//!     /* kind */ inkwell::debug_info::DWARFEmissionKind::Full,
//!     /* dwo_id */ 0,
//!     /* split_debug_inling */ false,
//!     /* debug_info_for_profiling */ false,
//! );
//! ```
//! ## Creating function debug info
//! ```ignore
//!  let ditype = dibuilder.create_basic_type(
//!      "type_name",
//!      0_u64,
//!      0x00,
//!      inkwell::debug_info::DIFlags::Public,
//!  ).unwrap();
//!  let subroutine_type = dibuilder.create_subroutine_type(
//!      compile_unit.get_file(),
//!      /* return type */ Some(ditype.as_type()),
//!      /* parameter types */ &[],
//!      inkwell::debug_info::DIFlags::Public,
//!  );
//!  let func_scope: DISubprogram<'_> = dibuilder.create_function(
//!      /* scope */ compile_unit.as_debug_info_scope(),
//!      /* func name */ "main",
//!      /* linkage_name */ None,
//!      /* file */ compile_unit.get_file(),
//!      /* line_no */ 0,
//!      /* DIType */ subroutine_type,
//!      /* is_local_to_unit */ true,
//!      /* is_definition */ true,
//!      /* scope_line */ 0,
//!      /* flags */ inkwell::debug_info::DIFlags::Public,
//!      /* is_optimized */ false,
//!  );
//! ```
//! The `DISubprogram` value must be attached to the generated `FunctionValue`:
//! ```ignore
//! /* after creating function: */
//!     let fn_val = module.add_function(fn_name_str, fn_type, None);
//!     fn_val.set_subprogram(func_scope);
//! ```
//!
//! ## Setting debug locations
//! ```ignore
//! let lexical_block = dibuilder.create_lexical_block(
//!         /* scope */ func_scope.as_debug_info_scope(),
//!         /* file */ compile_unit.get_file(),
//!         /* line_no */ 0,
//!         /* column_no */ 0);
//!
//! let loc = dibuilder
//!     .create_debug_location(&context, /* line */ 0, /* column */ 0,
//!     /* current_scope */ lexical_block.as_debug_info_scope(),
//!     /* inlined_at */ None);
//! builder.set_current_debug_location(&context, loc);
//!
//! // Create global variable
//! let gv = module.add_global(context.i64_type(), Some(inkwell::AddressSpace::Global), "gv");
//!
//!
//! let const_v = di.create_constant_expression(10);
//!
//! let gv_debug = di.create_global_variable_expression(cu.get_file().as_debug_info_scope(), "gv", "", cu.get_file(), 1, ditype.as_type(), true, Some(const_v), None, 8);
//!
//! let meta_value: inkwell::values::BasicMetadataValueEnum = gv_debug.as_metadata_value(&context).into();
//! let metadata = context.metadata_node(&[meta_value]);
//! gv.set_metadata(metadata, 0);//dbg
//!
//! ```
//!
//! ## Finalize debug info
//! Before any kind of code generation (including verification passes; they generate code and
//! validate debug info), do:
//! ```ignore
//! dibuilder.finalize();
//! ```

use crate::basic_block::BasicBlock;
use crate::context::Context;
pub use crate::debug_info::flags::{DIFlags, DIFlagsConstants};
use crate::module::Module;
use crate::values::{AsValueRef, BasicValueEnum, InstructionValue, PointerValue, MetadataValue};

#[llvm_versions(8.0..=latest)]
use llvm_sys::debuginfo::LLVMDIBuilderCreateTypedef;
pub use llvm_sys::debuginfo::LLVMDWARFTypeEncoding;
use llvm_sys::debuginfo::LLVMDebugMetadataVersion;
use llvm_sys::debuginfo::LLVMDisposeDIBuilder;
use llvm_sys::debuginfo::LLVMMetadataReplaceAllUsesWith;
use llvm_sys::debuginfo::LLVMTemporaryMDNode;
use llvm_sys::debuginfo::{LLVMCreateDIBuilder, LLVMCreateDIBuilderDisallowUnresolved};
use llvm_sys::debuginfo::{
    LLVMDIBuilderCreateAutoVariable, LLVMDIBuilderCreateBasicType, LLVMDIBuilderCreateCompileUnit,
    LLVMDIBuilderCreateDebugLocation, LLVMDIBuilderCreateExpression, LLVMDIBuilderCreateFile,
    LLVMDIBuilderCreateFunction, LLVMDIBuilderCreateLexicalBlock, LLVMDIBuilderCreateMemberType,
    LLVMDIBuilderCreateNameSpace, LLVMDIBuilderCreateParameterVariable,
    LLVMDIBuilderCreateStructType, LLVMDIBuilderCreateSubroutineType, LLVMDIBuilderCreateUnionType,
    LLVMDIBuilderFinalize, LLVMDIBuilderInsertDbgValueBefore, LLVMDIBuilderInsertDeclareAtEnd,
    LLVMDIBuilderInsertDeclareBefore, LLVMDILocationGetColumn, LLVMDILocationGetLine,
    LLVMDILocationGetScope, LLVMDITypeGetAlignInBits, LLVMDITypeGetOffsetInBits,
    LLVMDITypeGetSizeInBits,
};
#[llvm_versions(8.0..=latest)]
use llvm_sys::debuginfo::{LLVMDIBuilderCreateGlobalVariableExpression,LLVMDIBuilderCreateConstantValueExpression};
use llvm_sys::prelude::{LLVMDIBuilderRef, LLVMMetadataRef};
use llvm_sys::core::LLVMMetadataAsValue;
use std::convert::TryInto;
use std::marker::PhantomData;

/// Gets the version of debug metadata produced by the current LLVM version.
pub fn debug_metadata_version() -> libc::c_uint {
    unsafe { LLVMDebugMetadataVersion() }
}

/// A builder object to create debug info metadata. Used along with `Builder` while producing
/// IR. Created by `Module::create_debug_info_builder`. See `debug_info` module level
/// documentation for more.
#[derive(Debug, PartialEq, Eq)]
pub struct DebugInfoBuilder<'ctx> {
    pub(crate) builder: LLVMDIBuilderRef,
    _marker: PhantomData<&'ctx Context>,
}

/// Any kind of debug information scope (i.e. visibility of a source code symbol). Scopes are
/// created by special `DebugInfoBuilder` methods (eg `create_lexical_block`) and can be turned
/// into a `DIScope` with the `AsDIScope::as_debug_info_scope` trait method.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DIScope<'ctx> {
    metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

/// Specific scopes (i.e. `DILexicalBlock`) can be turned into a `DIScope` with the
/// `AsDIScope::as_debug_info_scope` trait method.
pub trait AsDIScope<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx>;
}

impl<'ctx> DebugInfoBuilder<'ctx> {
    #[llvm_versions(3.6..=10.0)]
    pub(crate) fn new(
        module: &Module,
        allow_unresolved: bool,
        language: DWARFSourceLanguage,
        filename: &str,
        directory: &str,
        producer: &str,
        is_optimized: bool,
        flags: &str,
        runtime_ver: libc::c_uint,
        split_name: &str,
        kind: DWARFEmissionKind,
        dwo_id: libc::c_uint,
        split_debug_inlining: bool,
        debug_info_for_profiling: bool,
    ) -> (Self, DICompileUnit<'ctx>) {
        let builder = unsafe {
            if allow_unresolved {
                LLVMCreateDIBuilder(module.module.get())
            } else {
                LLVMCreateDIBuilderDisallowUnresolved(module.module.get())
            }
        };

        let builder = DebugInfoBuilder {
            builder,
            _marker: PhantomData,
        };

        let file = builder.create_file(filename, directory);

        let cu = builder.create_compile_unit(
            language,
            file,
            producer,
            is_optimized,
            flags,
            runtime_ver,
            split_name,
            kind,
            dwo_id,
            split_debug_inlining,
            debug_info_for_profiling,
        );

        (builder, cu)
    }

    #[llvm_versions(11.0)]
    pub(crate) fn new(
        module: &Module,
        allow_unresolved: bool,
        language: DWARFSourceLanguage,
        filename: &str,
        directory: &str,
        producer: &str,
        is_optimized: bool,
        flags: &str,
        runtime_ver: libc::c_uint,
        split_name: &str,
        kind: DWARFEmissionKind,
        dwo_id: libc::c_uint,
        split_debug_inlining: bool,
        debug_info_for_profiling: bool,
        sysroot: &str,
        sdk: &str,
    ) -> (Self, DICompileUnit<'ctx>) {
        let builder = unsafe {
            if allow_unresolved {
                LLVMCreateDIBuilder(module.module.get())
            } else {
                LLVMCreateDIBuilderDisallowUnresolved(module.module.get())
            }
        };

        let builder = DebugInfoBuilder {
            builder,
            _marker: PhantomData,
        };

        let file = builder.create_file(filename, directory);

        let cu = builder.create_compile_unit(
            language,
            file,
            producer,
            is_optimized,
            flags,
            runtime_ver,
            split_name,
            kind,
            dwo_id,
            split_debug_inlining,
            debug_info_for_profiling,
            sysroot,
            sdk
        );

        (builder, cu)
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
    #[llvm_versions(3.6..=10.0)]
    fn create_compile_unit(
        &self,
        language: DWARFSourceLanguage,
        file: DIFile<'ctx>,
        producer: &str,
        is_optimized: bool,
        flags: &str,
        runtime_ver: libc::c_uint,
        split_name: &str,
        kind: DWARFEmissionKind,
        dwo_id: libc::c_uint,
        split_debug_inlining: bool,
        debug_info_for_profiling: bool,
    ) -> DICompileUnit<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateCompileUnit(
                self.builder,
                language.into(),
                file.metadata_ref,
                producer.as_ptr() as _,
                producer.len(),
                is_optimized as _,
                flags.as_ptr() as _,
                flags.len(),
                runtime_ver,
                split_name.as_ptr() as _,
                split_name.len(),
                kind.into(),
                dwo_id,
                split_debug_inlining as _,
                debug_info_for_profiling as _,
            )
        };

        DICompileUnit {
            file,
            metadata_ref,
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
    /// * `sysroot` The clang system root (value of -isysroot).
    /// * `sdk` The SDK name. On Darwin, this is the last component of the sysroot.
    #[llvm_versions(11.0)]
    fn create_compile_unit(
        &self,
        language: DWARFSourceLanguage,
        file: DIFile<'ctx>,
        producer: &str,
        is_optimized: bool,
        flags: &str,
        runtime_ver: libc::c_uint,
        split_name: &str,
        kind: DWARFEmissionKind,
        dwo_id: libc::c_uint,
        split_debug_inlining: bool,
        debug_info_for_profiling: bool,
        sysroot: &str,
        sdk: &str,
    ) -> DICompileUnit<'ctx> {

        let metadata_ref = unsafe {
            LLVMDIBuilderCreateCompileUnit(
                self.builder,
                language.into(),
                file.metadata_ref,
                producer.as_ptr() as _,
                producer.len(),
                is_optimized as _,
                flags.as_ptr() as _,
                flags.len(),
                runtime_ver,
                split_name.as_ptr() as _,
                split_name.len(),
                kind.into(),
                dwo_id,
                split_debug_inlining as _,
                debug_info_for_profiling as _,
                sysroot.as_ptr() as _,
                sysroot.len(),
                sdk.as_ptr() as _,
                sdk.len(),
            )
        };

        DICompileUnit {
            file,
            metadata_ref,
            _marker: PhantomData,
        }
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
    /// * `is_definition` - True if this is a function definition ("When isDefinition: false,
    /// subprograms describe a declaration in the type tree as opposed to a definition of a
    /// function").
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
        flags: DIFlags,
        is_optimized: bool,
    ) -> DISubprogram<'ctx> {
        let linkage_name = linkage_name.unwrap_or(name);

        let metadata_ref = unsafe {
            LLVMDIBuilderCreateFunction(
                self.builder,
                scope.metadata_ref,
                name.as_ptr() as _,
                name.len(),
                linkage_name.as_ptr() as _,
                linkage_name.len(),
                file.metadata_ref,
                line_no,
                ditype.metadata_ref,
                is_local_to_unit as _,
                is_definition as _,
                scope_line as libc::c_uint,
                flags,
                is_optimized as _,
            )
        };
        DISubprogram {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Create a lexical block scope.
    pub fn create_lexical_block(
        &self,
        parent_scope: DIScope<'ctx>,
        file: DIFile<'ctx>,
        line: u32,
        column: u32,
    ) -> DILexicalBlock<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateLexicalBlock(
                self.builder,
                parent_scope.metadata_ref,
                file.metadata_ref,
                line as libc::c_uint,
                column as libc::c_uint,
            )
        };
        DILexicalBlock {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Create a file scope.
    pub fn create_file(&self, filename: &str, directory: &str) -> DIFile<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateFile(
                self.builder,
                filename.as_ptr() as _,
                filename.len(),
                directory.as_ptr() as _,
                directory.len(),
            )
        };
        DIFile {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Create a debug location.
    pub fn create_debug_location(
        &self,
        context: &Context,
        line: u32,
        column: u32,
        scope: DIScope<'ctx>,
        inlined_at: Option<DILocation<'ctx>>,
    ) -> DILocation<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateDebugLocation(
                context.context,
                line,
                column,
                scope.metadata_ref,
                inlined_at
                    .map(|l| l.metadata_ref)
                    .unwrap_or(std::ptr::null_mut()),
            )
        };
        DILocation {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Create a primitive basic type. `encoding` is an unsigned int flag (`DW_ATE_*`
    /// enum) defined by the chosen DWARF standard.
    #[llvm_versions(7.0..=latest)]
    pub fn create_basic_type(
        &self,
        name: &str,
        size_in_bits: u64,
        encoding: LLVMDWARFTypeEncoding,
        #[cfg(not(feature = "llvm7-0"))]
        flags: DIFlags,
    ) -> Result<DIBasicType<'ctx>, &'static str> {
        if name.is_empty() {
            // Also, LLVM returns the same type if you ask for the same
            // (name, size_in_bits, encoding).
            return Err("basic types must have names");
        }
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateBasicType(
                self.builder,
                name.as_ptr() as _,
                name.len(),
                size_in_bits,
                encoding,
                #[cfg(not(feature = "llvm7-0"))]
                flags,
            )
        };
        Ok(DIBasicType {
            metadata_ref,
            _marker: PhantomData,
        })
    }

    /// Create a typedef (alias) of `ditype`
    #[llvm_versions(8.0..=latest)]
    pub fn create_typedef(
        &self,
        ditype: DIType<'ctx>,
        name: &str,
        file: DIFile<'ctx>,
        line_no: u32,
        scope: DIScope<'ctx>,
        #[cfg(not(any(feature = "llvm8-0", feature = "llvm9-0")))]
        align_in_bits: u32,
    ) -> DIDerivedType<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateTypedef(
                self.builder,
                ditype.metadata_ref,
                name.as_ptr() as _,
                name.len(),
                file.metadata_ref,
                line_no,
                scope.metadata_ref,
                #[cfg(not(any(feature = "llvm8-0", feature = "llvm9-0")))]
                align_in_bits,
            )
        };
        DIDerivedType {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Create union type of multiple types.
    pub fn create_union_type(
        &self,
        scope: DIScope<'ctx>,
        name: &str,
        file: DIFile<'ctx>,
        line_no: u32,
        size_in_bits: u64,
        align_in_bits: u32,
        flags: DIFlags,
        elements: &[DIType<'ctx>],
        runtime_language: u32,
        unique_id: &str,
    ) -> DICompositeType<'ctx> {
        let mut elements: Vec<LLVMMetadataRef> =
            elements.into_iter().map(|dt| dt.metadata_ref).collect();
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateUnionType(
                self.builder,
                scope.metadata_ref,
                name.as_ptr() as _,
                name.len(),
                file.metadata_ref,
                line_no,
                size_in_bits,
                align_in_bits,
                flags.into(),
                elements.as_mut_ptr(),
                elements.len().try_into().unwrap(),
                runtime_language,
                unique_id.as_ptr() as _,
                unique_id.len(),
            )
        };
        DICompositeType {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Create a type for a non-static member.
    pub fn create_member_type(
        &self,
        scope: DIScope<'ctx>,
        name: &str,
        file: DIFile<'ctx>,
        line_no: libc::c_uint,
        size_in_bits: u64,
        align_in_bits: u32,
        offset_in_bits: u64,
        flags: DIFlags,
        ty: DIType<'ctx>,
    ) -> DIDerivedType<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateMemberType(
                self.builder,
                scope.metadata_ref,
                name.as_ptr() as _,
                name.len(),
                file.metadata_ref,
                line_no,
                size_in_bits,
                align_in_bits,
                offset_in_bits,
                flags,
                ty.metadata_ref,
            )
        };
        DIDerivedType {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Create a struct type.
    pub fn create_struct_type(
        &self,
        scope: DIScope<'ctx>,
        name: &str,
        file: DIFile<'ctx>,
        line_no: libc::c_uint,
        size_in_bits: u64,
        align_in_bits: u32,
        flags: DIFlags,
        derived_from: Option<DIType<'ctx>>,
        elements: &[DIType<'ctx>],
        runtime_language: libc::c_uint,
        vtable_holder: Option<DIType<'ctx>>,
        unique_id: &str,
    ) -> DICompositeType<'ctx> {
        let mut elements: Vec<LLVMMetadataRef> =
            elements.iter().map(|dt| dt.metadata_ref).collect();
        let derived_from = derived_from.map_or(std::ptr::null_mut(), |dt| dt.metadata_ref);
        let vtable_holder = vtable_holder.map_or(std::ptr::null_mut(), |dt| dt.metadata_ref);
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateStructType(
                self.builder,
                scope.metadata_ref,
                name.as_ptr() as _,
                name.len(),
                file.metadata_ref,
                line_no,
                size_in_bits,
                align_in_bits,
                flags,
                derived_from,
                elements.as_mut_ptr(),
                elements.len().try_into().unwrap(),
                runtime_language,
                vtable_holder,
                unique_id.as_ptr() as _,
                unique_id.len(),
            )
        };
        DICompositeType {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Create a function type
    pub fn create_subroutine_type(
        &self,
        file: DIFile<'ctx>,
        return_type: Option<DIType<'ctx>>,
        parameter_types: &[DIType<'ctx>],
        flags: DIFlags,
    ) -> DISubroutineType<'ctx> {
        let mut p = vec![return_type.map_or(std::ptr::null_mut(), |t| t.metadata_ref)];
        p.append(
            &mut parameter_types
                .iter()
                .map(|t| t.metadata_ref)
                .collect::<Vec<LLVMMetadataRef>>(),
        );
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateSubroutineType(
                self.builder,
                file.metadata_ref,
                p.as_mut_ptr(),
                p.len().try_into().unwrap(),
                flags,
            )
        };
        DISubroutineType {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    #[llvm_versions(8.0..=latest)]
    pub fn create_global_variable_expression(
        &self,
        scope: DIScope<'ctx>,
        name: &str,
        linkage: &str,
        file: DIFile<'ctx>,
        line_no: u32,
        ty: DIType<'ctx>,
        local_to_unit: bool,
        expression: Option<DIExpression>,
        declaration: Option<DIScope>,
        align_in_bits: u32,
    ) -> DIGlobalVariableExpression<'ctx> {
        let expression_ptr = expression.map_or(std::ptr::null_mut(), |dt| dt.metadata_ref);
        let decl_ptr = declaration.map_or(std::ptr::null_mut(), |dt| dt.metadata_ref);
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateGlobalVariableExpression(
                self.builder,
                scope.metadata_ref,
                name.as_ptr() as _,
                name.len(),
                linkage.as_ptr() as _,
                linkage.len(),
                file.metadata_ref,
                line_no,
                ty.metadata_ref,
                local_to_unit as _,
                expression_ptr,
                decl_ptr,
                align_in_bits,
            )
        };
        DIGlobalVariableExpression {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    #[llvm_versions(8.0..=latest)]
    pub fn create_constant_expression(
        &self,
        value : i64,
    ) -> DIExpression<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateConstantValueExpression(
                self.builder,
                value,
            )
        };

        DIExpression {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Create function parameter variable.
    pub fn create_parameter_variable(
        &self,
        scope: DIScope<'ctx>,
        name: &str,
        arg_no: u32,
        file: DIFile<'ctx>,
        line_no: u32,
        ty: DIType<'ctx>,
        always_preserve: bool,
        flags: DIFlags,
    ) -> DILocalVariable<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateParameterVariable(
                self.builder,
                scope.metadata_ref,
                name.as_ptr() as _,
                name.len(),
                arg_no,
                file.metadata_ref,
                line_no,
                ty.metadata_ref,
                always_preserve as _,
                flags,
            )
        };
        DILocalVariable {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Create local automatic storage variable.
    pub fn create_auto_variable(
        &self,
        scope: DIScope<'ctx>,
        name: &str,
        file: DIFile<'ctx>,
        line_no: u32,
        ty: DIType<'ctx>,
        always_preserve: bool,
        flags: DIFlags,
        align_in_bits: u32,
    ) -> DILocalVariable<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateAutoVariable(
                self.builder,
                scope.metadata_ref,
                name.as_ptr() as _,
                name.len(),
                file.metadata_ref,
                line_no,
                ty.metadata_ref,
                always_preserve as _,
                flags,
                align_in_bits,
            )
        };
        DILocalVariable {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    pub fn create_namespace(
        &self,
        scope: DIScope<'ctx>,
        name: &str,
        export_symbols: bool
    ) -> DINamespace<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateNameSpace(
                self.builder,
                scope.metadata_ref,
                name.as_ptr() as _,
                name.len(),
                export_symbols as _
            )
        };
        DINamespace {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Insert a variable declaration (`llvm.dbg.declare`) before a specified instruction.
    pub fn insert_declare_before_instruction(
        &self,
        storage: PointerValue<'ctx>,
        var_info: Option<DILocalVariable<'ctx>>,
        expr: Option<DIExpression<'ctx>>,
        debug_loc: DILocation<'ctx>,
        instruction: InstructionValue<'ctx>,
    ) -> InstructionValue<'ctx> {
        let value_ref = unsafe {
            LLVMDIBuilderInsertDeclareBefore(
                self.builder,
                storage.as_value_ref(),
                var_info
                    .map(|v| v.metadata_ref)
                    .unwrap_or(std::ptr::null_mut()),
                expr.unwrap_or_else(|| self.create_expression(vec![])).metadata_ref,
                debug_loc.metadata_ref,
                instruction.as_value_ref(),
            )
        };
        InstructionValue::new(value_ref)
    }

    /// Insert a variable declaration (`llvm.dbg.declare` intrinsic) at the end of `block`
    pub fn insert_declare_at_end(
        &self,
        storage: PointerValue<'ctx>,
        var_info: Option<DILocalVariable<'ctx>>,
        expr: Option<DIExpression<'ctx>>,
        debug_loc: DILocation<'ctx>,
        block: BasicBlock<'ctx>,
    ) -> InstructionValue<'ctx> {
        let value_ref = unsafe {
            LLVMDIBuilderInsertDeclareAtEnd(
                self.builder,
                storage.as_value_ref(),
                var_info
                    .map(|v| v.metadata_ref)
                    .unwrap_or(std::ptr::null_mut()),
                expr.unwrap_or_else(|| self.create_expression(vec![])).metadata_ref,
                debug_loc.metadata_ref,
                block.basic_block,
            )
        };
        InstructionValue::new(value_ref)
    }

    /// Create an expression
    pub fn create_expression(&self, mut address_operations: Vec<i64>) -> DIExpression<'ctx> {
        let metadata_ref = unsafe {
            LLVMDIBuilderCreateExpression(
                self.builder,
                address_operations.as_mut_ptr(),
                address_operations.len(),
            )
        };
        DIExpression {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Insert a new llvm.dbg.value intrinsic call before an instruction.
    pub fn insert_dbg_value_before(
        &self,
        value: BasicValueEnum<'ctx>,
        var_info: DILocalVariable<'ctx>,
        expr: Option<DIExpression<'ctx>>,
        debug_loc: DILocation<'ctx>,
        instruction: InstructionValue<'ctx>,
    ) -> InstructionValue<'ctx> {
        let value_ref = unsafe {
            LLVMDIBuilderInsertDbgValueBefore(
                self.builder,
                value.as_value_ref(),
                var_info.metadata_ref,
                expr.unwrap_or_else(|| self.create_expression(vec![])).metadata_ref,
                debug_loc.metadata_ref,
                instruction.as_value_ref(),
            )
        };
        InstructionValue::new(value_ref)
    }

    /// Construct a placeholders derived type to be used when building debug info with circular references.
    ///
    /// All placeholders must be replaced before calling finalize().
    pub unsafe fn create_placeholder_derived_type(&self, context: &Context) -> DIDerivedType<'ctx> {
        let metadata_ref = LLVMTemporaryMDNode(context.context, std::ptr::null_mut(), 0);
        DIDerivedType {
            metadata_ref,
            _marker: PhantomData,
        }
    }

    /// Deletes a placeholder, replacing all uses of it with another derived type.
    ///
    /// # Safety:
    /// This and any other copies of this placeholder made by Copy or Clone
    /// become dangling pointers after calling this method.
    pub unsafe fn replace_placeholder_derived_type(
        &self,
        placeholder: DIDerivedType<'ctx>,
        other: DIDerivedType<'ctx>,
    ) {
        LLVMMetadataReplaceAllUsesWith(placeholder.metadata_ref, other.metadata_ref);
    }

    /// Construct any deferred debug info descriptors. May generate invalid metadata if debug info
    /// is incomplete. Module/function verification can then fail.
    ///
    /// Call before any kind of code generation (including verification). Can be called more than once.
    pub fn finalize(&self) {
        unsafe { LLVMDIBuilderFinalize(self.builder) };
    }
}

impl<'ctx> Drop for DebugInfoBuilder<'ctx> {
    fn drop(&mut self) {
        self.finalize();
        unsafe { LLVMDisposeDIBuilder(self.builder) }
    }
}

/// Source file scope for debug info
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

/// Compilation unit scope for debug info
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

/// Namespace scope for debug info
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DINamespace<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> AsDIScope<'ctx> for DINamespace<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

/// Function body scope for debug info
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DISubprogram<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    pub(crate) _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> AsDIScope<'ctx> for DISubprogram<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

/// Any kind of debug info type
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DIType<'ctx> {
    pub fn get_size_in_bits(&self) -> u64 {
        unsafe { LLVMDITypeGetSizeInBits(self.metadata_ref) }
    }

    pub fn get_align_in_bits(&self) -> u32 {
        unsafe { LLVMDITypeGetAlignInBits(self.metadata_ref) }
    }

    pub fn get_offset_in_bits(&self) -> u64 {
        unsafe { LLVMDITypeGetOffsetInBits(self.metadata_ref) }
    }
}

impl<'ctx> AsDIScope<'ctx> for DIType<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

/// A wrapper around a single type, such as a typedef or member type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIDerivedType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DIDerivedType<'ctx> {
    pub fn as_type(&self) -> DIType<'ctx> {
        DIType {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

impl<'ctx> AsDIScope<'ctx> for DIDerivedType<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

/// A primitive debug info type created by `create_basic_type` method of `DebugInfoBuilder`
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIBasicType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DIBasicType<'ctx> {
    pub fn as_type(&self) -> DIType<'ctx> {
        DIType {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

impl<'ctx> AsDIScope<'ctx> for DIBasicType<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}
/// A wrapper around an array of types, such as a union or struct.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DICompositeType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DICompositeType<'ctx> {
    pub fn as_type(&self) -> DIType<'ctx> {
        DIType {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

impl<'ctx> AsDIScope<'ctx> for DICompositeType<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

/// Metadata representing the type of a function
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DISubroutineType<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

/// Lexical block scope for debug info
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DILexicalBlock<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> AsDIScope<'ctx> for DILexicalBlock<'ctx> {
    fn as_debug_info_scope(self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: self.metadata_ref,
            _marker: PhantomData,
        }
    }
}

/// A debug location within the source code. Contains the following information:
///
/// - line, column
/// - scope
/// - inlined at
///
/// Created by `create_debug_location` of `DebugInfoBuilder` and consumed by
/// `set_current_debug_location` of `Builder`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DILocation<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    pub(crate) _marker: PhantomData<&'ctx Context>,
}

impl<'ctx> DILocation<'ctx> {
    pub fn get_line(&self) -> u32 {
        unsafe { LLVMDILocationGetLine(self.metadata_ref) }
    }

    pub fn get_column(&self) -> u32 {
        unsafe { LLVMDILocationGetColumn(self.metadata_ref) }
    }

    pub fn get_scope(&self) -> DIScope<'ctx> {
        DIScope {
            metadata_ref: unsafe { LLVMDILocationGetScope(self.metadata_ref) },
            _marker: PhantomData,
        }
    }
}

/// Metadata representing a variable inside a scope
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DILocalVariable<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIGlobalVariableExpression<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

impl <'ctx> DIGlobalVariableExpression<'ctx>  {
    pub fn as_metadata_value(&self, context: &Context) -> MetadataValue<'ctx> {
        let value = unsafe {
            LLVMMetadataAsValue(context.context, self.metadata_ref)
        };
        MetadataValue::new(value)
    }
}

/// https://llvm.org/docs/LangRef.html#diexpression
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DIExpression<'ctx> {
    pub(crate) metadata_ref: LLVMMetadataRef,
    _marker: PhantomData<&'ctx Context>,
}

pub use flags::*;
mod flags {
    use llvm_sys::debuginfo::{LLVMDWARFEmissionKind, LLVMDWARFSourceLanguage};
    pub use llvm_sys::debuginfo::LLVMDIFlags as DIFlags;

    pub trait DIFlagsConstants {
        const ZERO: Self;
        const PRIVATE: Self;
        const PROTECTED: Self;
        const PUBLIC: Self;
        const FWD_DECL: Self;
        const APPLE_BLOCK: Self;
        //#[llvm_versions(7.0..=9.0)]
        //const BLOCK_BYREF_STRUCT: Self;
        const VIRTUAL: Self;
        const ARTIFICIAL: Self;
        const EXPLICIT: Self;
        const PROTOTYPED: Self;
        const OBJC_CLASS_COMPLETE: Self;
        const OBJECT_POINTER: Self;
        const VECTOR: Self;
        const STATIC_MEMBER: Self;
        const LVALUE_REFERENCE: Self;
        const RVALUE_REFERENCE: Self;
        const RESERVED: Self;
        const SINGLE_INHERITANCE: Self;
        const MULTIPLE_INHERITANCE: Self;
        const VIRTUAL_INHERITANCE: Self;
        const INTRODUCED_VIRTUAL: Self;
        const BIT_FIELD: Self;
        const NO_RETURN: Self;
        //#[llvm_versions(7.0..=8.0)]
        //const MAIN_SUBPROGRAM: Self;
        const TYPE_PASS_BY_VALUE: Self;
        const TYPE_PASS_BY_REFERENCE: Self;
        //#[llvm_versions(7.0)]
        //const FIXED_ENUM: Self;
        //#[llvm_versions(8.0..=latest)]
        //const ENUM_CLASS: Self;
        const THUNK: Self;
        //#[llvm_versions(7.0..=8.0)]
        //const TRIVIAL: Self;
        //#[llvm_versions(9.0..=latest)]
        //const NON_TRIVIAL: Self;
        //#[llvm_versions(10.0)]
        //const RESERVED_BIT4: Self;
        //#[llvm_versions(8.0..=latest)]
        //const BIGE_NDIAN: Self;
        //#[llvm_versions(8.0..=latest)]
        //const LITTLE_ENDIAN: Self;
        const INDIRECT_VIRTUAL_BASE: Self;
    }
    impl DIFlagsConstants for DIFlags {
        const ZERO: DIFlags = llvm_sys::debuginfo::LLVMDIFlagZero;
        const PRIVATE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagPrivate;
        const PROTECTED: DIFlags = llvm_sys::debuginfo::LLVMDIFlagProtected;
        const PUBLIC: DIFlags = llvm_sys::debuginfo::LLVMDIFlagPublic;
        const FWD_DECL: DIFlags = llvm_sys::debuginfo::LLVMDIFlagFwdDecl;
        const APPLE_BLOCK: DIFlags = llvm_sys::debuginfo::LLVMDIFlagAppleBlock;
        //#[llvm_versions(7.0..=9.0)]
        //const BLOCK_BYREF_STRUCT: DIFlags = llvm_sys::debuginfo::LLVMDIFlagBlockByrefStruct;
        const VIRTUAL: DIFlags = llvm_sys::debuginfo::LLVMDIFlagVirtual;
        const ARTIFICIAL: DIFlags = llvm_sys::debuginfo::LLVMDIFlagArtificial;
        const EXPLICIT: DIFlags = llvm_sys::debuginfo::LLVMDIFlagExplicit;
        const PROTOTYPED: DIFlags = llvm_sys::debuginfo::LLVMDIFlagPrototyped;
        const OBJC_CLASS_COMPLETE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagObjcClassComplete;
        const OBJECT_POINTER: DIFlags = llvm_sys::debuginfo::LLVMDIFlagObjectPointer;
        const VECTOR: DIFlags = llvm_sys::debuginfo::LLVMDIFlagVector;
        const STATIC_MEMBER: DIFlags = llvm_sys::debuginfo::LLVMDIFlagStaticMember;
        const LVALUE_REFERENCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagLValueReference;
        const RVALUE_REFERENCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagRValueReference;
        const RESERVED: DIFlags = llvm_sys::debuginfo::LLVMDIFlagReserved;
        const SINGLE_INHERITANCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagSingleInheritance;
        const MULTIPLE_INHERITANCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagMultipleInheritance;
        const VIRTUAL_INHERITANCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagVirtualInheritance;
        const INTRODUCED_VIRTUAL: DIFlags = llvm_sys::debuginfo::LLVMDIFlagIntroducedVirtual;
        const BIT_FIELD: DIFlags = llvm_sys::debuginfo::LLVMDIFlagBitField;
        const NO_RETURN: DIFlags = llvm_sys::debuginfo::LLVMDIFlagNoReturn;
        //#[llvm_versions(7.0..=8.0)]
        //const MAIN_SUBPROGRAM: DIFlags = llvm_sys::debuginfo::LLVMDIFlagMainSubprogram;
        const TYPE_PASS_BY_VALUE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagTypePassByValue;
        const TYPE_PASS_BY_REFERENCE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagTypePassByReference;
        //#[llvm_versions(7.0)]
        //const FIXED_ENUM: DIFlags = llvm_sys::debuginfo::LLVMDIFlagFixedEnum;
        //#[llvm_versions(8.0..=latest)]
        //const ENUM_CLASS: DIFlags = llvm_sys::debuginfo::LLVMDIFlagEnumClass;
        const THUNK: DIFlags = llvm_sys::debuginfo::LLVMDIFlagThunk;
        //#[llvm_versions(7.0..=8.0)]
        //const TRIVIAL: DIFlags = llvm_sys::debuginfo::LLVMDIFlagTrivial;
        //#[llvm_versions(9.0..=latest)]
        //const NON_TRIVIAL: DIFlags = llvm_sys::debuginfo::LLVMDIFlagNonTrivial;
        //#[llvm_versions(10.0)]
        //const RESERVED_BIT4: DIFlags = llvm_sys::debuginfo::LLVMDIFlagReservedBit4;
        //#[llvm_versions(8.0..=latest)]
        //const BIG_ENDIAN: DIFlags = llvm_sys::debuginfo::LLVMDIFlagBigEndian;
        //#[llvm_versions(8.0..=latest)]
        //const LITTLE_ENDIAN: DIFlags = llvm_sys::debuginfo::LLVMDIFlagLittleEndian;
        const INDIRECT_VIRTUAL_BASE: DIFlags = llvm_sys::debuginfo::LLVMDIFlagIndirectVirtualBase;
    }

    /// The amount of debug information to emit. Corresponds to `LLVMDWARFEmissionKind` enum from LLVM.
    #[llvm_enum(LLVMDWARFEmissionKind)]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum DWARFEmissionKind {
        #[llvm_variant(LLVMDWARFEmissionKindNone)]
        None,
        #[llvm_variant(LLVMDWARFEmissionKindFull)]
        Full,
        #[llvm_variant(LLVMDWARFEmissionKindLineTablesOnly)]
        LineTablesOnly,
    }

    /// Source languages known by DWARF. Corresponds to `LLVMDWARFSourceLanguage` enum from LLVM.
    #[llvm_enum(LLVMDWARFSourceLanguage)]
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum DWARFSourceLanguage {
        #[llvm_variant(LLVMDWARFSourceLanguageC89)]
        C89,
        #[llvm_variant(LLVMDWARFSourceLanguageC)]
        C,
        #[llvm_variant(LLVMDWARFSourceLanguageAda83)]
        Ada83,
        #[llvm_variant(LLVMDWARFSourceLanguageC_plus_plus)]
        CPlusPlus,
        #[llvm_variant(LLVMDWARFSourceLanguageCobol74)]
        Cobol74,
        #[llvm_variant(LLVMDWARFSourceLanguageCobol85)]
        Cobol85,
        #[llvm_variant(LLVMDWARFSourceLanguageFortran77)]
        Fortran77,
        #[llvm_variant(LLVMDWARFSourceLanguageFortran90)]
        Fortran90,
        #[llvm_variant(LLVMDWARFSourceLanguagePascal83)]
        Pascal83,
        #[llvm_variant(LLVMDWARFSourceLanguageModula2)]
        Modula2,
        #[llvm_variant(LLVMDWARFSourceLanguageJava)]
        Java,
        #[llvm_variant(LLVMDWARFSourceLanguageC99)]
        C99,
        #[llvm_variant(LLVMDWARFSourceLanguageAda95)]
        Ada95,
        #[llvm_variant(LLVMDWARFSourceLanguageFortran95)]
        Fortran95,
        #[llvm_variant(LLVMDWARFSourceLanguagePLI)]
        PLI,
        #[llvm_variant(LLVMDWARFSourceLanguageObjC)]
        ObjC,
        #[llvm_variant(LLVMDWARFSourceLanguageObjC_plus_plus)]
        ObjCPlusPlus,
        #[llvm_variant(LLVMDWARFSourceLanguageUPC)]
        UPC,
        #[llvm_variant(LLVMDWARFSourceLanguageD)]
        D,
        #[llvm_variant(LLVMDWARFSourceLanguagePython)]
        Python,
        #[llvm_variant(LLVMDWARFSourceLanguageOpenCL)]
        OpenCL,
        #[llvm_variant(LLVMDWARFSourceLanguageGo)]
        Go,
        #[llvm_variant(LLVMDWARFSourceLanguageModula3)]
        Modula3,
        #[llvm_variant(LLVMDWARFSourceLanguageHaskell)]
        Haskell,
        #[llvm_variant(LLVMDWARFSourceLanguageC_plus_plus_03)]
        CPlusPlus03,
        #[llvm_variant(LLVMDWARFSourceLanguageC_plus_plus_11)]
        CPlusPlus11,
        #[llvm_variant(LLVMDWARFSourceLanguageOCaml)]
        OCaml,
        #[llvm_variant(LLVMDWARFSourceLanguageRust)]
        Rust,
        #[llvm_variant(LLVMDWARFSourceLanguageC11)]
        C11,
        #[llvm_variant(LLVMDWARFSourceLanguageSwift)]
        Swift,
        #[llvm_variant(LLVMDWARFSourceLanguageJulia)]
        Julia,
        #[llvm_variant(LLVMDWARFSourceLanguageDylan)]
        Dylan,
        #[llvm_variant(LLVMDWARFSourceLanguageC_plus_plus_14)]
        CPlusPlus14,
        #[llvm_variant(LLVMDWARFSourceLanguageFortran03)]
        Fortran03,
        #[llvm_variant(LLVMDWARFSourceLanguageFortran08)]
        Fortran08,
        #[llvm_variant(LLVMDWARFSourceLanguageRenderScript)]
        RenderScript,
        #[llvm_variant(LLVMDWARFSourceLanguageBLISS)]
        BLISS,
        #[llvm_variant(LLVMDWARFSourceLanguageMips_Assembler)]
        MipsAssembler,
        #[llvm_variant(LLVMDWARFSourceLanguageGOOGLE_RenderScript)]
        GOOGLERenderScript,
        #[llvm_variant(LLVMDWARFSourceLanguageBORLAND_Delphi)]
        BORLANDDelphi,
    }
}
