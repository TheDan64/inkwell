use llvm_sys::core::{LLVMDisposePassManager, LLVMInitializeFunctionPassManager, LLVMFinalizeFunctionPassManager, LLVMRunFunctionPassManager, LLVMRunPassManager, LLVMCreatePassManager, LLVMCreateFunctionPassManagerForModule, LLVMGetGlobalPassRegistry};
use llvm_sys::initialization::{LLVMInitializeCore, LLVMInitializeTransformUtils, LLVMInitializeScalarOpts, LLVMInitializeObjCARCOpts, LLVMInitializeVectorization, LLVMInitializeInstCombine, LLVMInitializeIPO, LLVMInitializeInstrumentation, LLVMInitializeAnalysis, LLVMInitializeIPA, LLVMInitializeCodeGen, LLVMInitializeTarget};
use llvm_sys::prelude::{LLVMPassManagerRef, LLVMPassRegistryRef};
use llvm_sys::target::LLVMAddTargetData;
use llvm_sys::transforms::ipo::{LLVMAddArgumentPromotionPass, LLVMAddConstantMergePass, LLVMAddDeadArgEliminationPass, LLVMAddFunctionAttrsPass, LLVMAddFunctionInliningPass, LLVMAddAlwaysInlinerPass, LLVMAddGlobalDCEPass, LLVMAddGlobalOptimizerPass, LLVMAddIPConstantPropagationPass, LLVMAddIPSCCPPass, LLVMAddInternalizePass, LLVMAddStripDeadPrototypesPass, LLVMAddPruneEHPass, LLVMAddStripSymbolsPass};
use llvm_sys::transforms::pass_manager_builder::{LLVMPassManagerBuilderRef, LLVMPassManagerBuilderCreate, LLVMPassManagerBuilderDispose, LLVMPassManagerBuilderSetOptLevel, LLVMPassManagerBuilderSetSizeLevel, LLVMPassManagerBuilderSetDisableUnitAtATime, LLVMPassManagerBuilderSetDisableUnrollLoops, LLVMPassManagerBuilderSetDisableSimplifyLibCalls, LLVMPassManagerBuilderUseInlinerWithThreshold, LLVMPassManagerBuilderPopulateFunctionPassManager, LLVMPassManagerBuilderPopulateModulePassManager, LLVMPassManagerBuilderPopulateLTOPassManager};
use llvm_sys::transforms::scalar::{LLVMAddAggressiveDCEPass, LLVMAddMemCpyOptPass, LLVMAddAlignmentFromAssumptionsPass, LLVMAddCFGSimplificationPass, LLVMAddDeadStoreEliminationPass, LLVMAddScalarizerPass, LLVMAddMergedLoadStoreMotionPass, LLVMAddGVNPass, LLVMAddIndVarSimplifyPass, LLVMAddInstructionCombiningPass, LLVMAddJumpThreadingPass, LLVMAddLICMPass, LLVMAddLoopDeletionPass, LLVMAddLoopIdiomPass, LLVMAddLoopRotatePass, LLVMAddLoopRerollPass, LLVMAddLoopUnrollPass, LLVMAddLoopUnswitchPass, LLVMAddPartiallyInlineLibCallsPass, LLVMAddLowerSwitchPass, LLVMAddPromoteMemoryToRegisterPass, LLVMAddSCCPPass, LLVMAddScalarReplAggregatesPass, LLVMAddScalarReplAggregatesPassSSA, LLVMAddScalarReplAggregatesPassWithThreshold, LLVMAddSimplifyLibCallsPass, LLVMAddTailCallEliminationPass, LLVMAddConstantPropagationPass, LLVMAddDemoteMemoryToRegisterPass, LLVMAddVerifierPass, LLVMAddCorrelatedValuePropagationPass, LLVMAddEarlyCSEPass, LLVMAddLowerExpectIntrinsicPass, LLVMAddTypeBasedAliasAnalysisPass, LLVMAddScopedNoAliasAAPass, LLVMAddBasicAliasAnalysisPass, LLVMAddReassociatePass};
#[cfg(not(feature = "llvm3-6"))]
use llvm_sys::transforms::scalar::LLVMAddBitTrackingDCEPass;
use llvm_sys::transforms::vectorize::{LLVMAddBBVectorizePass, LLVMAddLoopVectorizePass, LLVMAddSLPVectorizePass};

use OptimizationLevel;
use module::Module;
use targets::TargetData;
use values::{AsValueRef, FunctionValue};

// REVIEW: Opt Level might be identical to targets::Option<CodeGenOptLevel>
// REVIEW: size_level 0-2 according to llvmlite
pub struct PassManagerBuilder {
    pass_manager_builder: LLVMPassManagerBuilderRef,
}

impl PassManagerBuilder {
    fn new(pass_manager_builder: LLVMPassManagerBuilderRef) -> Self {
        assert!(!pass_manager_builder.is_null());

        PassManagerBuilder {
            pass_manager_builder: pass_manager_builder,
        }
    }

    pub fn create() -> Self {
        let pass_manager_builder = unsafe {
            LLVMPassManagerBuilderCreate()
        };

        PassManagerBuilder::new(pass_manager_builder)
    }

    pub fn set_optimization_level(&self, opt_level: OptimizationLevel) {
        unsafe {
            LLVMPassManagerBuilderSetOptLevel(self.pass_manager_builder, opt_level as u32)
        }
    }

    // REVIEW: Valid input 0-2 according to llvmlite
    pub fn set_size_level(&self, size_level: u32) {
        unsafe {
            LLVMPassManagerBuilderSetSizeLevel(self.pass_manager_builder, size_level)
        }
    }

    pub fn set_disable_unit_at_a_time(&self, disable: bool) {
        unsafe {
            LLVMPassManagerBuilderSetDisableUnitAtATime(self.pass_manager_builder, disable as i32)
        }
    }

    pub fn set_disable_unroll_loops(&self, disable: bool) {
        unsafe {
            LLVMPassManagerBuilderSetDisableUnrollLoops(self.pass_manager_builder, disable as i32)
        }
    }

    pub fn set_disable_simplify_lib_calls(&self, disable: bool) {
        unsafe {
            LLVMPassManagerBuilderSetDisableSimplifyLibCalls(self.pass_manager_builder, disable as i32)
        }
    }

    pub fn set_inliner_with_threshold(&self, threshold: u32) {
        unsafe {
            LLVMPassManagerBuilderUseInlinerWithThreshold(self.pass_manager_builder, threshold)
        }
    }

    // SubType: pass_manager: &PassManager<FunctionValue>
    pub fn populate_function_pass_manager(&self, pass_manager: &PassManager) {
        unsafe {
            LLVMPassManagerBuilderPopulateFunctionPassManager(self.pass_manager_builder, pass_manager.pass_manager)
        }
    }

    // SubType: pass_manager: &PassManager<Module>
    pub fn populate_module_pass_manager(&self, pass_manager: &PassManager) {
        unsafe {
            LLVMPassManagerBuilderPopulateModulePassManager(self.pass_manager_builder, pass_manager.pass_manager)
        }
    }

    // SubType: Need LTO subtype?
    pub fn populate_lto_pass_manager(&self, pass_manager: &PassManager, internalize: bool, run_inliner: bool) {
        unsafe {
            LLVMPassManagerBuilderPopulateLTOPassManager(self.pass_manager_builder, pass_manager.pass_manager, internalize as i32, run_inliner as i32)
        }
    }
}

impl Drop for PassManagerBuilder {
    fn drop(&mut self) {
        unsafe {
            LLVMPassManagerBuilderDispose(self.pass_manager_builder)
        }
    }
}

// SubTypes: PassManager<Module>, PassManager<FunctionValue>
pub struct PassManager {
    pub(crate) pass_manager: LLVMPassManagerRef,
}

impl PassManager {
    pub(crate) fn new(pass_manager: LLVMPassManagerRef) -> PassManager {
        assert!(!pass_manager.is_null());

        PassManager {
            pass_manager: pass_manager
        }
    }

    // SubTypes: PassManager<Module>::create()
    pub fn create_for_module() -> Self {
        let pass_manager = unsafe {
            LLVMCreatePassManager()
        };

        PassManager::new(pass_manager)
    }

    // SubTypes: PassManager<FunctionValue>::create()
    pub fn create_for_function(module: &Module) -> Self {
        let pass_manager = unsafe {
            LLVMCreateFunctionPassManagerForModule(module.module.get())
        };

        PassManager::new(pass_manager)
    }

    // return true means some pass modified the module, not an error occurred
    pub fn initialize(&self) -> bool {
        unsafe {
            LLVMInitializeFunctionPassManager(self.pass_manager) == 1
        }
    }

    pub fn finalize(&self) -> bool {
        unsafe {
            LLVMFinalizeFunctionPassManager(self.pass_manager) == 1
        }
    }

    pub fn run_on_function(&self, fn_value: &FunctionValue) -> bool {
        unsafe {
            LLVMRunFunctionPassManager(self.pass_manager, fn_value.as_value_ref()) == 1
        }
    }

    pub fn run_on_module(&self, module: &Module) -> bool {
        unsafe {
            LLVMRunPassManager(self.pass_manager, module.module.get()) == 1
        }
    }

    pub fn add_target_data(&self, target_data: &TargetData) {
        unsafe {
            LLVMAddTargetData(target_data.target_data, self.pass_manager)
        }
    }

    pub fn add_argument_promotion_pass(&self) {
        unsafe {
            LLVMAddArgumentPromotionPass(self.pass_manager)
        }
    }

    pub fn add_constant_merge_pass(&self) {
        unsafe {
            LLVMAddConstantMergePass(self.pass_manager)
        }
    }

    pub fn add_dead_arg_elimination_pass(&self) {
        unsafe {
            LLVMAddDeadArgEliminationPass(self.pass_manager)
        }
    }

    pub fn add_function_attrs_pass(&self) {
        unsafe {
            LLVMAddFunctionAttrsPass(self.pass_manager)
        }
    }

    pub fn add_function_inlining_pass(&self) {
        unsafe {
            LLVMAddFunctionInliningPass(self.pass_manager)
        }
    }

    pub fn add_always_inliner_pass(&self) {
        unsafe {
            LLVMAddAlwaysInlinerPass(self.pass_manager)
        }
    }

    pub fn add_global_dce_pass(&self) {
        unsafe {
            LLVMAddGlobalDCEPass(self.pass_manager)
        }
    }

    pub fn add_global_optimizer_pass(&self) {
        unsafe {
            LLVMAddGlobalOptimizerPass(self.pass_manager)
        }
    }

    pub fn add_ip_constant_propagation_pass(&self) {
        unsafe {
            LLVMAddIPConstantPropagationPass(self.pass_manager)
        }
    }

    pub fn add_prune_eh_pass(&self) {
        unsafe {
            LLVMAddPruneEHPass(self.pass_manager)
        }
    }

    pub fn add_ipsccp_pass(&self) {
        unsafe {
            LLVMAddIPSCCPPass(self.pass_manager)
        }
    }

    // REVIEW: Would it be ok to take a bool instead and cast it?
    pub fn add_interinalize_pass(&self, all_but_main: u32) {
        unsafe {
            LLVMAddInternalizePass(self.pass_manager, all_but_main)
        }
    }

    pub fn add_strip_dead_prototypes_pass(&self) {
        unsafe {
            LLVMAddStripDeadPrototypesPass(self.pass_manager)
        }
    }

    pub fn add_strip_symbol_pass(&self) {
        unsafe {
            LLVMAddStripSymbolsPass(self.pass_manager)
        }
    }

    pub fn add_bb_vectorize_pass(&self) {
        unsafe {
            LLVMAddBBVectorizePass(self.pass_manager)
        }
    }

    pub fn add_loop_vectorize_pass(&self) {
        unsafe {
            LLVMAddLoopVectorizePass(self.pass_manager)
        }
    }

    pub fn add_slp_vectorize_pass(&self) {
        unsafe {
            LLVMAddSLPVectorizePass(self.pass_manager)
        }
    }

    pub fn add_aggressive_dce_pass(&self) {
        unsafe {
            LLVMAddAggressiveDCEPass(self.pass_manager)
        }
    }

    #[cfg(not(feature = "llvm3-6"))]
    pub fn add_bit_tracking_dce_pass(&self) {
        unsafe {
            LLVMAddBitTrackingDCEPass(self.pass_manager)
        }
    }

    pub fn add_alignment_from_assumptions_pass(&self) {
        unsafe {
            LLVMAddAlignmentFromAssumptionsPass(self.pass_manager)
        }
    }

    pub fn add_cfg_simplification_pass(&self) {
        unsafe {
            LLVMAddCFGSimplificationPass(self.pass_manager)
        }
    }

    pub fn add_dead_store_elimination_pass(&self) {
        unsafe {
            LLVMAddDeadStoreEliminationPass(self.pass_manager)
        }
    }

    pub fn add_scalarizer_pass(&self) {
        unsafe {
            LLVMAddScalarizerPass(self.pass_manager)
        }
    }

    pub fn add_merged_load_store_motion_pass(&self) {
        unsafe {
            LLVMAddMergedLoadStoreMotionPass(self.pass_manager)
        }
    }

    pub fn add_gvn_pass(&self) {
        unsafe {
            LLVMAddGVNPass(self.pass_manager)
        }
    }

    // TODO: LLVM 4.0+
    // pub fn add_new_gvn_pass(&self) {
    //     unsafe {
    //         LLVMAddNewGVNPass(self.pass_manager)
    //     }
    // }

    pub fn add_ind_var_simplify_pass(&self) {
        unsafe {
            LLVMAddIndVarSimplifyPass(self.pass_manager)
        }
    }

    pub fn add_instruction_combining_pass(&self) {
        unsafe {
            LLVMAddInstructionCombiningPass(self.pass_manager)
        }
    }

    pub fn add_jump_threading_pass(&self) {
        unsafe {
            LLVMAddJumpThreadingPass(self.pass_manager)
        }
    }

    pub fn add_licm_pass(&self) {
        unsafe {
            LLVMAddLICMPass(self.pass_manager)
        }
    }

    pub fn add_loop_deletion_pass(&self) {
        unsafe {
            LLVMAddLoopDeletionPass(self.pass_manager)
        }
    }

    pub fn add_loop_idiom_pass(&self) {
        unsafe {
            LLVMAddLoopIdiomPass(self.pass_manager)
        }
    }

    pub fn add_loop_rotate_pass(&self) {
        unsafe {
            LLVMAddLoopRotatePass(self.pass_manager)
        }
    }

    pub fn add_loop_reroll_pass(&self) {
        unsafe {
            LLVMAddLoopRerollPass(self.pass_manager)
        }
    }

    pub fn add_loop_unroll_pass(&self) {
        unsafe {
            LLVMAddLoopUnrollPass(self.pass_manager)
        }
    }

    pub fn add_loop_unswitch_pass(&self) {
        unsafe {
            LLVMAddLoopUnswitchPass(self.pass_manager)
        }
    }

    pub fn add_memcpy_optimize_pass(&self) {
        unsafe {
            LLVMAddMemCpyOptPass(self.pass_manager)
        }
    }

    pub fn add_partially_inline_lib_calls_pass(&self) {
        unsafe {
            LLVMAddPartiallyInlineLibCallsPass(self.pass_manager)
        }
    }

    pub fn add_lower_switch_pass(&self) {
        unsafe {
            LLVMAddLowerSwitchPass(self.pass_manager)
        }
    }

    pub fn add_promote_memory_to_register_pass(&self) {
        unsafe {
            LLVMAddPromoteMemoryToRegisterPass(self.pass_manager)
        }
    }

    pub fn add_reassociate_pass(&self) {
        unsafe {
            LLVMAddReassociatePass(self.pass_manager)
        }
    }

    pub fn add_sccp_pass(&self) {
        unsafe {
            LLVMAddSCCPPass(self.pass_manager)
        }
    }

    pub fn add_scalar_repl_aggregates_pass(&self) {
        unsafe {
            LLVMAddScalarReplAggregatesPass(self.pass_manager)
        }
    }

    pub fn add_scalar_repl_aggregates_pass_ssa(&self) {
        unsafe {
            LLVMAddScalarReplAggregatesPassSSA(self.pass_manager)
        }
    }

    pub fn add_scalar_repl_aggregates_pass_with_threshold(&self, threshold: i32) {
        unsafe {
            LLVMAddScalarReplAggregatesPassWithThreshold(self.pass_manager, threshold)
        }
    }

    pub fn add_simplify_lib_calls_pass(&self) {
        unsafe {
            LLVMAddSimplifyLibCallsPass(self.pass_manager)
        }
    }

    pub fn add_tail_call_elimination_pass(&self) {
        unsafe {
            LLVMAddTailCallEliminationPass(self.pass_manager)
        }
    }

    pub fn add_constant_propagation_pass(&self) {
        unsafe {
            LLVMAddConstantPropagationPass(self.pass_manager)
        }
    }

    pub fn add_demote_memory_to_register_pass(&self) {
        unsafe {
            LLVMAddDemoteMemoryToRegisterPass(self.pass_manager)
        }
    }

    pub fn add_verifier_pass(&self) {
        unsafe {
            LLVMAddVerifierPass(self.pass_manager)
        }
    }

    pub fn add_correlated_value_propagation_pass(&self) {
        unsafe {
            LLVMAddCorrelatedValuePropagationPass(self.pass_manager)
        }
    }

    pub fn add_early_cse_pass(&self) {
        unsafe {
            LLVMAddEarlyCSEPass(self.pass_manager)
        }
    }

    // TODO: LLVM 4.0+
    // pub fn add_early_cse_mem_ssa_pass(&self) {
    //     unsafe {
    //         LLVMAddEarlyCSEMemSSAPass(self.pass_manager)
    //     }
    // }

    pub fn add_lower_expect_intrinsic_pass(&self) {
        unsafe {
            LLVMAddLowerExpectIntrinsicPass(self.pass_manager)
        }
    }

    pub fn add_type_based_alias_analysis_pass(&self) {
        unsafe {
            LLVMAddTypeBasedAliasAnalysisPass(self.pass_manager)
        }
    }

    pub fn add_scoped_no_alias_aa_pass(&self) {
        unsafe {
            LLVMAddScopedNoAliasAAPass(self.pass_manager)
        }
    }

    pub fn add_basic_alias_analysis_pass(&self) {
        unsafe {
            LLVMAddBasicAliasAnalysisPass(self.pass_manager)
        }
    }
}

impl Drop for PassManager {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposePassManager(self.pass_manager)
        }
    }
}

pub struct PassRegistry {
    pass_registry: LLVMPassRegistryRef,
}

impl PassRegistry {
    pub fn new(pass_registry: LLVMPassRegistryRef) -> PassRegistry {
        assert!(!pass_registry.is_null());

        PassRegistry {
            pass_registry,
        }
    }

    pub fn get_global() -> PassRegistry {
        let pass_registry = unsafe {
            LLVMGetGlobalPassRegistry()
        };

        PassRegistry::new(pass_registry)
    }

    pub fn initialize_core(&self) {
        unsafe {
            LLVMInitializeCore(self.pass_registry)
        }
    }

    pub fn initialize_transform_utils(&self) {
        unsafe {
            LLVMInitializeTransformUtils(self.pass_registry)
        }
    }

    pub fn initialize_scalar_opts(&self) {
        unsafe {
            LLVMInitializeScalarOpts(self.pass_registry)
        }
    }

    pub fn initialize_obj_carc_opts(&self) {
        unsafe {
            LLVMInitializeObjCARCOpts(self.pass_registry)
        }
    }

    pub fn initialize_vectorization(&self) {
        unsafe {
            LLVMInitializeVectorization(self.pass_registry)
        }
    }

    pub fn initialize_inst_combine(&self) {
        unsafe {
            LLVMInitializeInstCombine(self.pass_registry)
        }
    }

    // Let us begin our initial public offering
    pub fn initialize_ipo(&self) {
        unsafe {
            LLVMInitializeIPO(self.pass_registry)
        }
    }

    pub fn initialize_instrumentation(&self) {
        unsafe {
            LLVMInitializeInstrumentation(self.pass_registry)
        }
    }

    pub fn initialize_analysis(&self) {
        unsafe {
            LLVMInitializeAnalysis(self.pass_registry)
        }
    }

    pub fn initialize_ipa(&self) {
        unsafe {
            LLVMInitializeIPA(self.pass_registry)
        }
    }

    pub fn initialize_codegen(&self) {
        unsafe {
            LLVMInitializeCodeGen(self.pass_registry)
        }
    }

    pub fn initialize_target(&self) {
        unsafe {
            LLVMInitializeTarget(self.pass_registry)
        }
    }
}
