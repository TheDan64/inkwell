extern crate inkwell;

use self::inkwell::context::Context;
use self::inkwell::pass_manager::{PassManagerBuilder, PassManager};
use self::inkwell::targets::CodeGenOptLevel::Aggressive;

#[test]
fn test_init_all_passes_for_module() {
    let context = Context::create();
    let module = context.create_module("my_module");
    let pass_manager = PassManager::create_for_module();

    pass_manager.add_argument_promotion_pass();
    pass_manager.add_constant_merge_pass();
    pass_manager.add_dead_arg_elimination_pass();
    pass_manager.add_function_attrs_pass();
    pass_manager.add_function_inlining_pass();
    pass_manager.add_always_inliner_pass();
    pass_manager.add_global_dce_pass();
    pass_manager.add_global_optimizer_pass();
    pass_manager.add_ip_constant_propagation_pass();
    pass_manager.add_prune_eh_pass();
    pass_manager.add_ipsccp_pass();
    pass_manager.add_interinalize_pass(1);
    pass_manager.add_strip_dead_prototypes_pass();
    pass_manager.add_strip_symbol_pass();
    pass_manager.add_bb_vectorize_pass();
    pass_manager.add_loop_vectorize_pass();
    pass_manager.add_slp_vectorize_pass();
    pass_manager.add_aggressive_dce_pass();
    pass_manager.add_bit_tracking_dce_pass(); // TODO: 3.7+ only
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
    pass_manager.add_constant_propagation_pass();
    pass_manager.add_demote_memory_to_register_pass();
    pass_manager.add_verifier_pass();
    pass_manager.add_correlated_value_propagation_pass();
    pass_manager.add_early_cse_pass();
    pass_manager.add_lower_expect_intrinsic_pass();
    pass_manager.add_type_based_alias_analysis_pass();
    pass_manager.add_scoped_no_alias_aa_pass();
    pass_manager.add_basic_alias_analysis_pass();

    assert!(!pass_manager.initialize());
    assert!(!pass_manager.finalize());

    pass_manager.run_on_module(&module);

    assert!(!pass_manager.initialize());
    assert!(!pass_manager.finalize());

    // TODO: Test when initialize and finalize are true
}

#[test]
fn test_pass_manager_builder() {
    let builder = PassManagerBuilder::create();

    builder.set_optimization_level(Some(&Aggressive));
    builder.set_size_level(2);
    builder.set_inliner_with_threshold(42);
    builder.set_disable_unit_at_a_time(true);
    builder.set_disable_unroll_loops(true);
    builder.set_disable_simplify_lib_calls(true);

    // TODO: Run on various type of pass managers
}
