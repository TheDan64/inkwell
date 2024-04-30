use inkwell::context::Context;
use inkwell::passes::{PassManager, PassManagerBuilder, PassRegistry};

#[llvm_versions(13..)]
use inkwell::passes::PassBuilderOptions;
#[llvm_versions(13..)]
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;

#[test]
fn test_init_all_passes_for_module() {
    let context = Context::create();
    let module = context.create_module("my_module");
    let pass_manager = PassManager::create(());

    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    pass_manager.add_argument_promotion_pass();
    pass_manager.add_constant_merge_pass();
    #[cfg(not(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0"
    )))]
    pass_manager.add_merge_functions_pass();
    pass_manager.add_dead_arg_elimination_pass();
    pass_manager.add_function_attrs_pass();
    pass_manager.add_function_inlining_pass();
    pass_manager.add_always_inliner_pass();
    pass_manager.add_global_dce_pass();
    pass_manager.add_global_optimizer_pass();
    #[cfg(not(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0"
    )))]
    pass_manager.add_ip_constant_propagation_pass();
    #[cfg(not(feature = "llvm16-0"))]
    pass_manager.add_prune_eh_pass();
    pass_manager.add_ipsccp_pass();
    pass_manager.add_internalize_pass(true);
    pass_manager.add_strip_dead_prototypes_pass();
    pass_manager.add_strip_symbol_pass();
    #[cfg(feature = "llvm4-0")]
    pass_manager.add_bb_vectorize_pass();
    pass_manager.add_loop_vectorize_pass();
    pass_manager.add_slp_vectorize_pass();
    pass_manager.add_aggressive_dce_pass();
    pass_manager.add_bit_tracking_dce_pass();
    pass_manager.add_alignment_from_assumptions_pass();
    pass_manager.add_cfg_simplification_pass();
    pass_manager.add_dead_store_elimination_pass();
    pass_manager.add_scalarizer_pass();
    pass_manager.add_merged_load_store_motion_pass();
    pass_manager.add_gvn_pass();
    pass_manager.add_ind_var_simplify_pass();
    pass_manager.add_instruction_combining_pass();
    pass_manager.add_jump_threading_pass();
    pass_manager.add_licm_pass();
    pass_manager.add_loop_deletion_pass();
    pass_manager.add_loop_idiom_pass();
    pass_manager.add_loop_rotate_pass();
    pass_manager.add_loop_reroll_pass();
    pass_manager.add_loop_unroll_pass();
    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    pass_manager.add_loop_unswitch_pass();
    pass_manager.add_memcpy_optimize_pass();
    pass_manager.add_partially_inline_lib_calls_pass();
    pass_manager.add_lower_switch_pass();
    pass_manager.add_promote_memory_to_register_pass();
    pass_manager.add_reassociate_pass();
    pass_manager.add_sccp_pass();
    pass_manager.add_scalar_repl_aggregates_pass();
    pass_manager.add_scalar_repl_aggregates_pass_ssa();
    pass_manager.add_scalar_repl_aggregates_pass_with_threshold(1);
    pass_manager.add_simplify_lib_calls_pass();
    pass_manager.add_tail_call_elimination_pass();
    #[cfg(not(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0"
    )))]
    pass_manager.add_constant_propagation_pass();
    #[cfg(any(
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0",
        feature = "llvm15-0",
        feature = "llvm16-0"
    ))]
    pass_manager.add_instruction_simplify_pass();
    pass_manager.add_demote_memory_to_register_pass();
    pass_manager.add_verifier_pass();
    pass_manager.add_correlated_value_propagation_pass();
    pass_manager.add_early_cse_pass();
    pass_manager.add_lower_expect_intrinsic_pass();
    pass_manager.add_type_based_alias_analysis_pass();
    pass_manager.add_scoped_no_alias_aa_pass();
    pass_manager.add_basic_alias_analysis_pass();
    pass_manager.add_early_cse_mem_ssa_pass();
    pass_manager.add_new_gvn_pass();

    #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0", feature = "llvm16-0")))]
    pass_manager.add_aggressive_inst_combiner_pass();
    #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0")))]
    pass_manager.add_loop_unroll_and_jam_pass();

    #[cfg(not(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm15-0",
        feature = "llvm16-0"
    )))]
    {
        pass_manager.add_coroutine_early_pass();
        pass_manager.add_coroutine_split_pass();
        pass_manager.add_coroutine_elide_pass();
        pass_manager.add_coroutine_cleanup_pass();
    }

    pass_manager.run_on(&module);
}

#[test]
fn test_pass_manager_builder() {
    let pass_manager_builder = PassManagerBuilder::create();

    pass_manager_builder.set_optimization_level(OptimizationLevel::Aggressive);
    pass_manager_builder.set_size_level(2);
    pass_manager_builder.set_inliner_with_threshold(42);
    pass_manager_builder.set_disable_unit_at_a_time(true);
    pass_manager_builder.set_disable_unroll_loops(true);
    pass_manager_builder.set_disable_simplify_lib_calls(true);

    let context = Context::create();
    let module = context.create_module("my_module");

    let fn_pass_manager = PassManager::create(&module);

    pass_manager_builder.populate_function_pass_manager(&fn_pass_manager);

    let void_type = context.void_type();
    let fn_type = void_type.fn_type(&[], false);
    let fn_value = module.add_function("my_fn", fn_type, None);
    let builder = context.create_builder();
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);
    builder.build_return(None).unwrap();

    assert!(!fn_pass_manager.initialize());

    // TODO: Test with actual changes? Would be true in that case
    // REVIEW: Segfaults in 4.0
    #[cfg(not(feature = "llvm4-0"))]
    assert!(!fn_pass_manager.run_on(&fn_value));

    assert!(!fn_pass_manager.finalize());

    let module_pass_manager = PassManager::create(());

    pass_manager_builder.populate_module_pass_manager(&module_pass_manager);

    #[cfg(any(
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
        feature = "llvm9-0",
        feature = "llvm10-0",
        feature = "llvm11-0",
        feature = "llvm12-0",
        feature = "llvm13-0",
        feature = "llvm14-0"
    ))]
    {
        let module2 = module.clone();

        let lto_pass_manager = PassManager::create(());
        pass_manager_builder.populate_lto_pass_manager(&lto_pass_manager, false, false);

        assert!(lto_pass_manager.run_on(&module2));
    }

    #[cfg(any(feature = "llvm4-0", feature = "llvm5-0"))]
    assert!(!module_pass_manager.run_on(&module));
    #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0")))]
    assert!(module_pass_manager.run_on(&module));
}

#[test]
fn test_pass_registry() {
    let pass_registry = PassRegistry::get_global();

    pass_registry.initialize_core();
    pass_registry.initialize_transform_utils();
    pass_registry.initialize_scalar_opts();
    #[cfg(not(feature = "llvm16-0"))]
    pass_registry.initialize_obj_carc_opts();
    pass_registry.initialize_vectorization();
    pass_registry.initialize_inst_combine();
    pass_registry.initialize_ipo();
    #[cfg(not(feature = "llvm16-0"))]
    pass_registry.initialize_instrumentation();
    pass_registry.initialize_analysis();
    pass_registry.initialize_ipa();
    pass_registry.initialize_codegen();
    pass_registry.initialize_target();
    #[cfg(not(any(feature = "llvm4-0", feature = "llvm5-0", feature = "llvm6-0", feature = "llvm16-0")))]
    pass_registry.initialize_aggressive_inst_combiner();
}

#[llvm_versions(13..)]
#[test]
fn test_run_passes() {
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
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).unwrap();
    let machine = target
        .create_target_machine(
            &triple,
            //TODO : Add cpu features as optionals
            "generic", //TargetMachine::get_host_cpu_name().to_string().as_str(),
            "",        //TargetMachine::get_host_cpu_features().to_string().as_str(),
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    module.run_passes("default<O2>", &machine, pass_options).unwrap();
}

#[llvm_versions(13..)]
#[test]
fn test_run_passes_invalid() {
    let pass_options = PassBuilderOptions::create();

    let initialization_config = &InitializationConfig::default();
    Target::initialize_all(initialization_config);
    let context = Context::create();
    let module = context.create_module("my_module");
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).unwrap();
    let machine = target
        .create_target_machine(
            &triple,
            //TODO : Add cpu features as optionals
            TargetMachine::get_host_cpu_name().to_string().as_str(),
            TargetMachine::get_host_cpu_features().to_string().as_str(),
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    let res = module.run_passes("invalid_pass", &machine, pass_options);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_str().unwrap(), "unknown pass name 'invalid_pass'");
}
