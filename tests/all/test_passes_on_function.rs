#[llvm_versions(20..)]
use inkwell::OptimizationLevel;
#[llvm_versions(20..)]
use inkwell::context::Context;
#[llvm_versions(20..)]
use inkwell::passes::PassBuilderOptions;
#[llvm_versions(20..)]
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};

#[llvm_versions(20..)]
#[test]
fn test_run_passes_on_function() {
    let pass_options = PassBuilderOptions::create();
    pass_options.set_verify_each(true);
    pass_options.set_debug_logging(true);
    pass_options.set_loop_interleaving(true);
    pass_options.set_loop_vectorization(true);
    pass_options.set_loop_slp_vectorization(true);
    pass_options.set_loop_unrolling(true);
    pass_options.set_forget_all_scev_in_loop_unroll(true);
    pass_options.set_licm_mssa_opt_cap(1);
    pass_options.set_licm_mssa_no_acc_for_promotion_cap(10);
    pass_options.set_call_graph_profile(true);
    pass_options.set_merge_functions(true);

    let initialization_config = &InitializationConfig::default();
    Target::initialize_all(initialization_config);
    let context = Context::create();
    let module = context.create_module("my_module");
    let builder = context.create_builder();

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).unwrap();
    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);

    let function = module.add_function("test_function_passes", fn_type, None);

    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);
    let _ = builder.build_return(Some(&i32_type.const_int(3, false)));

    assert!(
        function
            .run_passes("instcombine,reassociate,gvn,simplifycfg", &machine, pass_options)
            .is_ok()
    );
}

#[llvm_versions(20..)]
#[test]
fn test_run_passes_on_function_invalid() {
    let pass_options = PassBuilderOptions::create();

    let initialization_config = &InitializationConfig::default();
    Target::initialize_all(initialization_config);
    let context = Context::create();
    let module = context.create_module("my_module");
    let builder = context.create_builder();

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).unwrap();
    let machine = target
        .create_target_machine(
            &triple,
            TargetMachine::get_host_cpu_name().to_string().as_str(),
            TargetMachine::get_host_cpu_features().to_string().as_str(),
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);

    let function = module.add_function("test_function_passes", fn_type, None);

    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(basic_block);
    let _ = builder.build_return(Some(&i32_type.const_int(3, false)));

    let res = function.run_passes("invalid_pass", &machine, pass_options);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_str().unwrap(),
        "unknown function pass 'invalid_pass' in pipeline 'invalid_pass'"
    );
}
