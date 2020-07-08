use inkwell::context::Context;
use inkwell::debug_info::{
    AsDIScope, DIFlags, DISubprogram, DWARFEmissionKind, DWARFSourceLanguage, DebugInfoBuilder,
};
use inkwell::module::FlagBehavior;

#[test]
fn test_smoke() {
    let context = Context::create();
    let module = context.create_module("bin");

    let debug_metadata_version = context.i32_type().const_int(3, false);
    module.add_basic_value_flag(
        "Debug Info Version",
        FlagBehavior::Warning,
        debug_metadata_version,
    );
    let builder = context.create_builder();
    let mut dibuilder = module.create_debug_info_builder(true);

    let compile_unit = dibuilder.create_compile_unit(
        DWARFSourceLanguage::C,
        dibuilder.create_file("source_file", "."),
        "my llvm compiler frontend",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
    );

    let ditype = dibuilder.create_basic_type("type_name", 0_u64, 0x00, DIFlags::Public);
    let subroutine_type = dibuilder.create_subroutine_type(
        compile_unit.get_file(),
        Some(ditype.as_type()),
        &[],
        DIFlags::Public,
    );
    let func_scope: DISubprogram<'_> = dibuilder.create_function(
        compile_unit.as_debug_info_scope(),
        "main",
        None,
        compile_unit.get_file(),
        0,
        subroutine_type,
        true,
        true,
        0,
        DIFlags::Public,
        false,
    );

    let fn_type = context.i64_type().fn_type(&[], false);
    let fn_val = module.add_function("fn_name_str", fn_type, None);
    fn_val.set_subprogram(func_scope);

    let lexical_block = dibuilder.create_lexical_block(
        func_scope.as_debug_info_scope(),
        compile_unit.get_file(),
        0,
        0,
    );

    let loc =
        dibuilder.create_debug_location(&context, 0, 0, lexical_block.as_debug_info_scope(), None);
    builder.set_current_debug_location(&context, loc);

    dibuilder.finalize();
}
