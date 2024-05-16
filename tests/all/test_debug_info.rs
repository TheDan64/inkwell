use inkwell::context::Context;
use inkwell::debug_info::{AsDIScope, DIFlags, DIFlagsConstants, DISubprogram, DWARFEmissionKind, DWARFSourceLanguage};
use inkwell::module::FlagBehavior;
use inkwell::values::AnyValue;

#[test]
fn test_smoke() {
    let context = Context::create();
    let module = context.create_module("bin");

    let debug_metadata_version = context.i32_type().const_int(3, false);
    module.add_basic_value_flag("Debug Info Version", FlagBehavior::Warning, debug_metadata_version);
    let builder = context.create_builder();
    let (dibuilder, compile_unit) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        "source_file",
        ".",
        "my llvm compiler frontend",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
    );

    let ditype = dibuilder
        .create_basic_type(
            "type_name",
            0_u64,
            0x00,
            #[cfg(not(feature = "llvm7-0"))]
            DIFlags::PUBLIC,
        )
        .unwrap();
    let subroutine_type =
        dibuilder.create_subroutine_type(compile_unit.get_file(), Some(ditype.as_type()), &[], DIFlags::PUBLIC);
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
        DIFlags::PUBLIC,
        false,
    );

    let fn_type = context.i64_type().fn_type(&[], false);
    let fn_val = module.add_function("fn_name_str", fn_type, None);
    fn_val.set_subprogram(func_scope);

    let basic_block = context.append_basic_block(fn_val, "entry");
    builder.position_at_end(basic_block);
    builder.build_return(Some(&context.i64_type().const_zero())).unwrap();

    let lexical_block = dibuilder.create_lexical_block(func_scope.as_debug_info_scope(), compile_unit.get_file(), 0, 0);

    let loc = dibuilder.create_debug_location(&context, 0, 0, lexical_block.as_debug_info_scope(), None);

    #[cfg(any(feature = "llvm7-0", feature = "llvm8-0",))]
    builder.set_current_debug_location(&context, loc);
    #[cfg(not(any(feature = "llvm7-0", feature = "llvm8-0",)))]
    builder.set_current_debug_location(loc);

    dibuilder.finalize();

    assert!(module.verify().is_ok());
}

#[test]
fn test_struct_with_placeholders() {
    let context = Context::create();
    let module = context.create_module("");

    let (dibuilder, compile_unit) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        "source_file",
        ".",
        "my llvm compiler frontend",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
    );

    // Some byte aligned integer types.
    let i32ty = dibuilder
        .create_basic_type(
            "i32",
            32,
            0x07,
            #[cfg(not(feature = "llvm7-0"))]
            DIFlags::PUBLIC,
        )
        .unwrap();
    let i64ty = dibuilder
        .create_basic_type(
            "i64",
            64,
            0x07,
            #[cfg(not(feature = "llvm7-0"))]
            DIFlags::PUBLIC,
        )
        .unwrap();
    let f32ty = dibuilder
        .create_basic_type(
            "f32",
            32,
            0x04,
            #[cfg(not(feature = "llvm7-0"))]
            DIFlags::PUBLIC,
        )
        .unwrap();
    let f64ty = dibuilder
        .create_basic_type(
            "f64",
            64,
            0x04,
            #[cfg(not(feature = "llvm7-0"))]
            DIFlags::PUBLIC,
        )
        .unwrap();

    let member_sizes = [32, 64, 32, 64];
    let member_types = [i32ty, i64ty, f32ty, f64ty];
    let member_placeholders = member_types
        .iter()
        .map(|_ty| unsafe { dibuilder.create_placeholder_derived_type(&context) })
        .collect::<Vec<_>>();
    let member_placeholders_as_ditype = member_types.iter().map(|ty| ty.as_type()).collect::<Vec<_>>();

    let structty = dibuilder.create_struct_type(
        compile_unit.get_file().as_debug_info_scope(),
        "",
        compile_unit.get_file(),
        0,
        192,
        8,
        DIFlags::PUBLIC,
        None,
        member_placeholders_as_ditype.as_slice(),
        0,
        None,
        "",
    );

    let mut offset = 0;
    for ((placeholder, ty), size) in member_placeholders
        .iter()
        .zip(member_types.iter())
        .zip(member_sizes.iter())
    {
        let member = dibuilder.create_member_type(
            structty.as_debug_info_scope(),
            "",
            compile_unit.get_file(),
            0,
            *size,
            8,
            offset,
            DIFlags::PUBLIC,
            ty.as_type(),
        );
        unsafe {
            dibuilder.replace_placeholder_derived_type(*placeholder, member);
        }
        offset += size;
    }

    dibuilder.finalize();

    assert!(module.verify().is_ok());
}

#[test]
fn test_no_explicit_finalize() {
    let context = Context::create();
    let module = context.create_module("");

    let (dibuilder, _compile_unit) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        "source_file",
        ".",
        "my llvm compiler frontend",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
    );

    drop(dibuilder);

    assert!(module.verify().is_ok());
}

#[llvm_versions(8..)]
#[test]
fn test_replacing_placeholder_with_placeholder() {
    let context = Context::create();
    let module = context.create_module("");

    let (dibuilder, compile_unit) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        "source_file",
        ".",
        "my llvm compiler frontend",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
    );

    let i32ty = dibuilder.create_basic_type("i32", 32, 0x07, DIFlags::PUBLIC).unwrap();
    let typedefty = dibuilder.create_typedef(
        i32ty.as_type(),
        "",
        compile_unit.get_file(),
        0,
        compile_unit.get_file().as_debug_info_scope(),
        #[cfg(not(any(feature = "llvm8-0", feature = "llvm9-0")))]
        32,
    );

    unsafe {
        let ph1 = dibuilder.create_placeholder_derived_type(&context);
        let ph2 = dibuilder.create_placeholder_derived_type(&context);

        dibuilder.replace_placeholder_derived_type(ph2, ph1);
        dibuilder.replace_placeholder_derived_type(ph1, typedefty);
    }
}

#[test]
fn test_anonymous_basic_type() {
    let context = Context::create();
    let module = context.create_module("bin");

    let (dibuilder, _compile_unit) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        "source_file",
        ".",
        "my llvm compiler frontend",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
    );

    assert_eq!(
        dibuilder.create_basic_type(
            "",
            0_u64,
            0x00,
            #[cfg(not(feature = "llvm7-0"))]
            DIFlags::ZERO
        ),
        Err("basic types must have names")
    );
}

#[llvm_versions(8..)]
#[test]
fn test_global_expressions() {
    let context = Context::create();
    let module = context.create_module("bin");

    let (dibuilder, compile_unit) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        "source_file",
        ".",
        "my llvm compiler frontend",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
    );

    let di_type = dibuilder.create_basic_type("type_name", 0_u64, 0x00, DIFlags::ZERO);
    let gv = module.add_global(context.i64_type(), Some(inkwell::AddressSpace::from(1u16)), "gv");

    let const_v = dibuilder.create_constant_expression(10);

    let gv_debug = dibuilder.create_global_variable_expression(
        compile_unit.as_debug_info_scope(),
        "gv",
        "",
        compile_unit.get_file(),
        1,
        di_type.unwrap().as_type(),
        true,
        Some(const_v),
        None,
        8,
    );

    let metadata = context.metadata_node(&[gv_debug.as_metadata_value(&context).into()]);

    gv.set_metadata(metadata, 0);

    // TODO: Metadata set on the global values cannot be retrieved using the C api,
    // therefore, it's currently not possible to test that the data was set without generating the IR
    assert!(
        gv.print_to_string().to_string().contains("!dbg"),
        "expected !dbg but generated gv was {}",
        gv.print_to_string()
    );
}

#[test]
fn test_pointer_types() {
    let context = Context::create();
    let module = context.create_module("bin");

    let (dibuilder, _) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        "source_file",
        ".",
        "my llvm compiler frontend",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
    );

    let di_type = dibuilder
        .create_basic_type(
            "type_name",
            8_u64,
            0x00,
            #[cfg(not(feature = "llvm7-0"))]
            DIFlags::ZERO,
        )
        .unwrap()
        .as_type();

    //Smoke test that the pointer gets created
    dibuilder.create_pointer_type("pointer_name", di_type, 64, 64, inkwell::AddressSpace::from(1u16));
}

#[test]
fn test_reference_types() {
    let context = Context::create();
    let module = context.create_module("bin");

    let (dibuilder, _) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        "source_file",
        ".",
        "my llvm compiler frontend",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
    );

    let di_type = dibuilder
        .create_basic_type(
            "type_name",
            8_u64,
            0x00,
            #[cfg(not(feature = "llvm7-0"))]
            DIFlags::ZERO,
        )
        .unwrap()
        .as_type();

    //Smoke test that the pointer gets created
    dibuilder.create_reference_type(di_type, 0x1000);
}

#[test]
#[allow(clippy::single_range_in_vec_init)]
fn test_array_type() {
    let context = Context::create();
    let module = context.create_module("bin");

    let (dibuilder, _) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        "source_file",
        ".",
        "my llvm compiler frontend",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
        #[cfg(any(
            feature = "llvm11-0",
            feature = "llvm12-0",
            feature = "llvm13-0",
            feature = "llvm14-0",
            feature = "llvm15-0",
            feature = "llvm16-0",
            feature = "llvm17-0",
            feature = "llvm18-0"
        ))]
        "",
    );

    let di_type = dibuilder
        .create_basic_type(
            "type_name",
            8_u64,
            0x00,
            #[cfg(not(feature = "llvm7-0"))]
            DIFlags::ZERO,
        )
        .unwrap()
        .as_type();

    // Smoke test that the array gets created.
    dibuilder.create_array_type(di_type, 160, 64, &[(0..20)]);

    dibuilder.create_array_type(di_type, 160, 64, &[(0..20), (-1..30), (20..55)]);
}
