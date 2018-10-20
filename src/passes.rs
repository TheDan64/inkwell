use llvm_sys::core::{LLVMDisposePassManager, LLVMInitializeFunctionPassManager, LLVMFinalizeFunctionPassManager, LLVMRunFunctionPassManager, LLVMRunPassManager, LLVMCreatePassManager, LLVMCreateFunctionPassManagerForModule, LLVMGetGlobalPassRegistry};
use llvm_sys::initialization::{LLVMInitializeCore, LLVMInitializeTransformUtils, LLVMInitializeScalarOpts, LLVMInitializeObjCARCOpts, LLVMInitializeVectorization, LLVMInitializeInstCombine, LLVMInitializeIPO, LLVMInitializeInstrumentation, LLVMInitializeAnalysis, LLVMInitializeIPA, LLVMInitializeCodeGen, LLVMInitializeTarget};
use llvm_sys::prelude::{LLVMPassManagerRef, LLVMPassRegistryRef};
use llvm_sys::transforms::ipo::{LLVMAddArgumentPromotionPass, LLVMAddConstantMergePass, LLVMAddDeadArgEliminationPass, LLVMAddFunctionAttrsPass, LLVMAddFunctionInliningPass, LLVMAddAlwaysInlinerPass, LLVMAddGlobalDCEPass, LLVMAddGlobalOptimizerPass, LLVMAddIPConstantPropagationPass, LLVMAddIPSCCPPass, LLVMAddInternalizePass, LLVMAddStripDeadPrototypesPass, LLVMAddPruneEHPass, LLVMAddStripSymbolsPass};
use llvm_sys::transforms::pass_manager_builder::{LLVMPassManagerBuilderRef, LLVMPassManagerBuilderCreate, LLVMPassManagerBuilderDispose, LLVMPassManagerBuilderSetOptLevel, LLVMPassManagerBuilderSetSizeLevel, LLVMPassManagerBuilderSetDisableUnitAtATime, LLVMPassManagerBuilderSetDisableUnrollLoops, LLVMPassManagerBuilderSetDisableSimplifyLibCalls, LLVMPassManagerBuilderUseInlinerWithThreshold, LLVMPassManagerBuilderPopulateFunctionPassManager, LLVMPassManagerBuilderPopulateModulePassManager, LLVMPassManagerBuilderPopulateLTOPassManager};
use llvm_sys::transforms::scalar::{LLVMAddAggressiveDCEPass, LLVMAddMemCpyOptPass, LLVMAddAlignmentFromAssumptionsPass, LLVMAddCFGSimplificationPass, LLVMAddDeadStoreEliminationPass, LLVMAddScalarizerPass, LLVMAddMergedLoadStoreMotionPass, LLVMAddGVNPass, LLVMAddIndVarSimplifyPass, LLVMAddInstructionCombiningPass, LLVMAddJumpThreadingPass, LLVMAddLICMPass, LLVMAddLoopDeletionPass, LLVMAddLoopIdiomPass, LLVMAddLoopRotatePass, LLVMAddLoopRerollPass, LLVMAddLoopUnrollPass, LLVMAddLoopUnswitchPass, LLVMAddPartiallyInlineLibCallsPass, LLVMAddSCCPPass, LLVMAddScalarReplAggregatesPass, LLVMAddScalarReplAggregatesPassSSA, LLVMAddScalarReplAggregatesPassWithThreshold, LLVMAddSimplifyLibCallsPass, LLVMAddTailCallEliminationPass, LLVMAddConstantPropagationPass, LLVMAddDemoteMemoryToRegisterPass, LLVMAddVerifierPass, LLVMAddCorrelatedValuePropagationPass, LLVMAddEarlyCSEPass, LLVMAddLowerExpectIntrinsicPass, LLVMAddTypeBasedAliasAnalysisPass, LLVMAddScopedNoAliasAAPass, LLVMAddBasicAliasAnalysisPass, LLVMAddReassociatePass};
#[llvm_versions(3.7 => latest)]
use llvm_sys::transforms::scalar::LLVMAddBitTrackingDCEPass;
use llvm_sys::transforms::vectorize::{LLVMAddLoopVectorizePass, LLVMAddSLPVectorizePass};

use OptimizationLevel;
use module::Module;
#[llvm_versions(3.6 => 3.8)]
use targets::TargetData;
use values::{AsValueRef, FunctionValue};

// REVIEW: Opt Level might be identical to targets::Option<CodeGenOptLevel>
#[derive(Debug)]
pub struct PassManagerBuilder {
    pass_manager_builder: LLVMPassManagerBuilderRef,
}

impl PassManagerBuilder {
    fn new(pass_manager_builder: LLVMPassManagerBuilderRef) -> Self {
        assert!(!pass_manager_builder.is_null());

        PassManagerBuilder {
            pass_manager_builder,
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

    // REVIEW: Valid input 0-2 according to llvmlite. Maybe better as an enum?
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
/// A manager for running optimization and simplification passes. Much of the
/// documenation for specific passes is directly from the [LLVM
/// documentation](https://llvm.org/docs/Passes.html).
#[derive(Debug)]
pub struct PassManager {
    pub(crate) pass_manager: LLVMPassManagerRef,
}

impl PassManager {
    pub(crate) fn new(pass_manager: LLVMPassManagerRef) -> PassManager {
        assert!(!pass_manager.is_null());

        PassManager {
            pass_manager,
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

    // SubTypes: For PassManager<FunctionValue> only, rename run_on
    pub fn run_on_function(&self, fn_value: &FunctionValue) -> bool {
        unsafe {
            LLVMRunFunctionPassManager(self.pass_manager, fn_value.as_value_ref()) == 1
        }
    }

    // SubTypes: For PassManager<Module> only, rename run_on
    pub fn run_on_module(&self, module: &Module) -> bool {
        unsafe {
            LLVMRunPassManager(self.pass_manager, module.module.get()) == 1
        }
    }

    #[llvm_versions(3.6 => 3.8)]
    pub fn add_target_data(&self, target_data: &TargetData) {
        use llvm_sys::target::LLVMAddTargetData;

        unsafe {
            LLVMAddTargetData(target_data.target_data, self.pass_manager)
        }
    }

    /// This pass promotes "by reference" arguments to be "by value" arguments.
    /// In practice, this means looking for internal functions that have pointer
    /// arguments. If it can prove, through the use of alias analysis, that an
    /// argument is only loaded, then it can pass the value into the function
    /// instead of the address of the value. This can cause recursive simplification
    /// of code and lead to the elimination of allocas (especially in C++ template
    /// code like the STL).
    ///
    /// This pass also handles aggregate arguments that are passed into a function,
    /// scalarizing them if the elements of the aggregate are only loaded. Note that
    /// it refuses to scalarize aggregates which would require passing in more than
    /// three operands to the function, because passing thousands of operands for a
    /// large array or structure is unprofitable!
    ///
    /// Note that this transformation could also be done for arguments that are
    /// only stored to (returning the value instead), but does not currently.
    /// This case would be best handled when and if LLVM starts supporting multiple
    /// return values from functions.
    pub fn add_argument_promotion_pass(&self) {
        unsafe {
            LLVMAddArgumentPromotionPass(self.pass_manager)
        }
    }

    /// Merges duplicate global constants together into a single constant that is
    /// shared. This is useful because some passes (i.e., TraceValues) insert a lot
    /// of string constants into the program, regardless of whether or not an existing
    /// string is available.
    pub fn add_constant_merge_pass(&self) {
        unsafe {
            LLVMAddConstantMergePass(self.pass_manager)
        }
    }

    /// This pass deletes dead arguments from internal functions. Dead argument
    /// elimination removes arguments which are directly dead, as well as arguments
    /// only passed into function calls as dead arguments of other functions. This
    /// pass also deletes dead arguments in a similar way.
    ///
    /// This pass is often useful as a cleanup pass to run after aggressive
    /// interprocedural passes, which add possibly-dead arguments.
    pub fn add_dead_arg_elimination_pass(&self) {
        unsafe {
            LLVMAddDeadArgEliminationPass(self.pass_manager)
        }
    }

    /// A simple interprocedural pass which walks the call-graph, looking for
    /// functions which do not access or only read non-local memory, and marking
    /// them readnone/readonly. In addition, it marks function arguments (of
    /// pointer type) “nocapture” if a call to the function does not create
    /// any copies of the pointer value that outlive the call. This more or
    /// less means that the pointer is only dereferenced, and not returned
    /// from the function or stored in a global. This pass is implemented
    /// as a bottom-up traversal of the call-graph.
    pub fn add_function_attrs_pass(&self) {
        unsafe {
            LLVMAddFunctionAttrsPass(self.pass_manager)
        }
    }

    /// Bottom-up inlining of functions into callees.
    pub fn add_function_inlining_pass(&self) {
        unsafe {
            LLVMAddFunctionInliningPass(self.pass_manager)
        }
    }

    /// A custom inliner that handles only functions that are marked as “always inline”.
    pub fn add_always_inliner_pass(&self) {
        unsafe {
            LLVMAddAlwaysInlinerPass(self.pass_manager)
        }
    }

    /// This transform is designed to eliminate unreachable internal
    /// globals from the program. It uses an aggressive algorithm,
    /// searching out globals that are known to be alive. After it
    /// finds all of the globals which are needed, it deletes
    /// whatever is left over. This allows it to delete recursive
    /// chunks of the program which are unreachable.
    pub fn add_global_dce_pass(&self) {
        unsafe {
            LLVMAddGlobalDCEPass(self.pass_manager)
        }
    }

    /// This pass transforms simple global variables that never have
    /// their address taken. If obviously true, it marks read/write
    /// globals as constant, deletes variables only stored to, etc.
    pub fn add_global_optimizer_pass(&self) {
        unsafe {
            LLVMAddGlobalOptimizerPass(self.pass_manager)
        }
    }

    /// This pass implements an extremely simple interprocedural
    /// constant propagation pass. It could certainly be improved
    /// in many different ways, like using a worklist. This pass
    /// makes arguments dead, but does not remove them. The existing
    /// dead argument elimination pass should be run after this to
    /// clean up the mess.
    pub fn add_ip_constant_propagation_pass(&self) {
        unsafe {
            LLVMAddIPConstantPropagationPass(self.pass_manager)
        }
    }

    /// This file implements a simple interprocedural pass which
    /// walks the call-graph, turning invoke instructions into
    /// call instructions if and only if the callee cannot throw
    /// an exception. It implements this as a bottom-up traversal
    /// of the call-graph.
    pub fn add_prune_eh_pass(&self) {
        unsafe {
            LLVMAddPruneEHPass(self.pass_manager)
        }
    }

    /// An interprocedural variant of [Sparse Conditional Constant
    /// Propagation](https://llvm.org/docs/Passes.html#passes-sccp).
    pub fn add_ipsccp_pass(&self) {
        unsafe {
            LLVMAddIPSCCPPass(self.pass_manager)
        }
    }

    /// This pass loops over all of the functions in the input module,
    /// looking for a main function. If a main function is found, all
    /// other functions and all global variables with initializers are
    /// marked as internal.
    pub fn add_internalize_pass(&self, all_but_main: bool) {
        unsafe {
            LLVMAddInternalizePass(self.pass_manager, all_but_main as u32)
        }
    }

    /// This pass loops over all of the functions in the input module,
    /// looking for dead declarations and removes them. Dead declarations
    /// are declarations of functions for which no implementation is available
    /// (i.e., declarations for unused library functions).
    pub fn add_strip_dead_prototypes_pass(&self) {
        unsafe {
            LLVMAddStripDeadPrototypesPass(self.pass_manager)
        }
    }

    /// Performs code stripping. This transformation can delete:
    ///
    /// * Names for virtual registers
    /// * Symbols for internal globals and functions
    /// * Debug information
    ///
    /// Note that this transformation makes code much less readable,
    /// so it should only be used in situations where the strip utility
    /// would be used, such as reducing code size or making it harder
    /// to reverse engineer code.
    pub fn add_strip_symbol_pass(&self) {
        unsafe {
            LLVMAddStripSymbolsPass(self.pass_manager)
        }
    }

    /// This pass combines instructions inside basic blocks to form
    /// vector instructions. It iterates over each basic block,
    /// attempting to pair compatible instructions, repeating this
    /// process until no additional pairs are selected for vectorization.
    /// When the outputs of some pair of compatible instructions are
    /// used as inputs by some other pair of compatible instructions,
    /// those pairs are part of a potential vectorization chain.
    /// Instruction pairs are only fused into vector instructions when
    /// they are part of a chain longer than some threshold length.
    /// Moreover, the pass attempts to find the best possible chain
    /// for each pair of compatible instructions. These heuristics
    /// are intended to prevent vectorization in cases where it would
    /// not yield a performance increase of the resulting code.
    #[llvm_versions(3.6 => 6.0)]
    pub fn add_bb_vectorize_pass(&self) {
        use llvm_sys::transforms::vectorize::LLVMAddBBVectorizePass;

        unsafe {
            LLVMAddBBVectorizePass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_loop_vectorize_pass(&self) {
        unsafe {
            LLVMAddLoopVectorizePass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_slp_vectorize_pass(&self) {
        unsafe {
            LLVMAddSLPVectorizePass(self.pass_manager)
        }
    }

    /// ADCE aggressively tries to eliminate code. This pass is similar
    /// to [DCE](https://llvm.org/docs/Passes.html#passes-dce) but it
    /// assumes that values are dead until proven otherwise. This is
    /// similar to [SCCP](https://llvm.org/docs/Passes.html#passes-sccp),
    /// except applied to the liveness of values.
    pub fn add_aggressive_dce_pass(&self) {
        unsafe {
            LLVMAddAggressiveDCEPass(self.pass_manager)
        }
    }

    #[llvm_versions(3.7 => latest)]
    /// No LLVM documentation is available at this time.
    pub fn add_bit_tracking_dce_pass(&self) {
        unsafe {
            LLVMAddBitTrackingDCEPass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_alignment_from_assumptions_pass(&self) {
        unsafe {
            LLVMAddAlignmentFromAssumptionsPass(self.pass_manager)
        }
    }

    /// Performs dead code elimination and basic block merging. Specifically:
    ///
    /// * Removes basic blocks with no predecessors.
    /// * Merges a basic block into its predecessor if there is only one and the predecessor only has one successor.
    /// * Eliminates PHI nodes for basic blocks with a single predecessor.
    /// * Eliminates a basic block that only contains an unconditional branch.
    pub fn add_cfg_simplification_pass(&self) {
        unsafe {
            LLVMAddCFGSimplificationPass(self.pass_manager)
        }
    }

    /// A trivial dead store elimination that only considers basic-block local redundant stores.
    pub fn add_dead_store_elimination_pass(&self) {
        unsafe {
            LLVMAddDeadStoreEliminationPass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_scalarizer_pass(&self) {
        unsafe {
            LLVMAddScalarizerPass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_merged_load_store_motion_pass(&self) {
        unsafe {
            LLVMAddMergedLoadStoreMotionPass(self.pass_manager)
        }
    }

    /// This pass performs global value numbering to eliminate
    /// fully and partially redundant instructions. It also
    /// performs redundant load elimination.
    pub fn add_gvn_pass(&self) {
        unsafe {
            LLVMAddGVNPass(self.pass_manager)
        }
    }

    /// This pass performs global value numbering to eliminate
    /// fully and partially redundant instructions. It also
    /// performs redundant load elimination.
    // REVIEW: Is `LLVMAddGVNPass` deprecated? Should we just seemlessly replace
    // the old one with this one in 4.0+?
    #[llvm_versions(4.0 => latest)]
    pub fn add_new_gvn_pass(&self) {
        use llvm_sys::transforms::scalar::LLVMAddNewGVNPass;

        unsafe {
            LLVMAddNewGVNPass(self.pass_manager)
        }
    }

    /// This transformation analyzes and transforms the induction variables (and
    /// computations derived from them) into simpler forms suitable for subsequent
    /// analysis and transformation.
    ///
    /// This transformation makes the following changes to each loop with an
    /// identifiable induction variable:
    ///
    /// * All loops are transformed to have a single canonical induction variable
    /// which starts at zero and steps by one.
    ///
    /// * The canonical induction variable is guaranteed to be the first PHI node
    /// in the loop header block.
    ///
    /// * Any pointer arithmetic recurrences are raised to use array subscripts.
    ///
    /// If the trip count of a loop is computable, this pass also makes the
    /// following changes:
    ///
    /// * The exit condition for the loop is canonicalized to compare the induction
    /// value against the exit value. This turns loops like:
    ///
    /// ```c
    /// for (i = 7; i*i < 1000; ++i)
    /// ```
    /// into
    /// ```c
    /// for (i = 0; i != 25; ++i)
    /// ```
    ///
    /// * Any use outside of the loop of an expression derived from the indvar is
    /// changed to compute the derived value outside of the loop, eliminating the
    /// dependence on the exit value of the induction variable. If the only purpose
    /// of the loop is to compute the exit value of some derived expression, this
    /// transformation will make the loop dead.
    ///
    /// This transformation should be followed by strength reduction after all of
    /// the desired loop transformations have been performed. Additionally, on
    /// targets where it is profitable, the loop could be transformed to count
    /// down to zero (the "do loop" optimization).
    pub fn add_ind_var_simplify_pass(&self) {
        unsafe {
            LLVMAddIndVarSimplifyPass(self.pass_manager)
        }
    }

    /// Combine instructions to form fewer, simple instructions. This pass
    /// does not modify the CFG. This pass is where algebraic simplification happens.
    ///
    /// This pass combines things like:
    ///
    /// ```c
    /// %Y = add i32 %X, 1
    /// %Z = add i32 %Y, 1
    /// ```
    /// into:
    /// ```c
    /// %Z = add i32 %X, 2
    /// ```
    ///
    /// This is a simple worklist driven algorithm.
    ///
    /// This pass guarantees that the following canonicalizations are performed
    /// on the program:
    ///
    /// 1. If a binary operator has a constant operand, it is moved to the
    /// right-hand side.
    ///
    /// 2. Bitwise operators with constant operands are always grouped so that
    /// shifts are performed first, then ors, then ands, then xors.
    ///
    /// 3. Compare instructions are converted from <, >, ≤, or ≥ to = or ≠ if possible.
    ///
    /// 4. All cmp instructions on boolean values are replaced with logical operations.
    ///
    /// 5. add X, X is represented as mul X, 2 ⇒ shl X, 1
    ///
    /// 6. Multiplies with a constant power-of-two argument are transformed into shifts.
    ///
    /// 7. ... etc.
    ///
    /// This pass can also simplify calls to specific well-known function calls
    /// (e.g. runtime library functions). For example, a call exit(3) that occurs within
    /// the main() function can be transformed into simply return 3. Whether or not library
    /// calls are simplified is controlled by the [-functionattrs](https://llvm.org/docs/Passes.html#passes-functionattrs)
    /// pass and LLVM’s knowledge of library calls on different targets.
    pub fn add_instruction_combining_pass(&self) {
        unsafe {
            LLVMAddInstructionCombiningPass(self.pass_manager)
        }
    }

    /// Jump threading tries to find distinct threads of control flow
    /// running through a basic block. This pass looks at blocks that
    /// have multiple predecessors and multiple successors. If one or
    /// more of the predecessors of the block can be proven to always
    /// cause a jump to one of the successors, we forward the edge from
    /// the predecessor to the successor by duplicating the contents of
    /// this block.
    ///
    /// An example of when this can occur is code like this:
    ///
    /// ```c
    /// if () { ...
    ///   X = 4;
    /// }
    /// if (X < 3) {
    /// ```
    ///
    /// In this case, the unconditional branch at the end of the first
    /// if can be revectored to the false side of the second if.
    pub fn add_jump_threading_pass(&self) {
        unsafe {
            LLVMAddJumpThreadingPass(self.pass_manager)
        }
    }

    /// This pass performs loop invariant code motion,
    /// attempting to remove as much code from the body of
    /// a loop as possible. It does this by either hoisting
    /// code into the preheader block, or by sinking code to
    /// the exit blocks if it is safe. This pass also promotes
    /// must-aliased memory locations in the loop to live in
    /// registers, thus hoisting and sinking “invariant” loads
    /// and stores.
    ///
    /// This pass uses alias analysis for two purposes:
    ///
    /// 1. Moving loop invariant loads and calls out of loops.
    /// If we can determine that a load or call inside of a
    /// loop never aliases anything stored to, we can hoist
    /// it or sink it like any other instruction.
    ///
    /// 2. Scalar Promotion of Memory. If there is a store
    /// instruction inside of the loop, we try to move the
    /// store to happen AFTER the loop instead of inside of
    /// the loop. This can only happen if a few conditions
    /// are true:
    ///
    ///     1. The pointer stored through is loop invariant.
    ///
    ///     2. There are no stores or loads in the loop
    /// which may alias the pointer. There are no calls in
    /// the loop which mod/ref the pointer.
    ///
    /// If these conditions are true, we can promote the loads
    /// and stores in the loop of the pointer to use a temporary
    /// alloca'd variable. We then use the mem2reg functionality
    /// to construct the appropriate SSA form for the variable.
    pub fn add_licm_pass(&self) {
        unsafe {
            LLVMAddLICMPass(self.pass_manager)
        }
    }

    /// This file implements the Dead Loop Deletion Pass.
    /// This pass is responsible for eliminating loops with
    /// non-infinite computable trip counts that have no side
    /// effects or volatile instructions, and do not contribute
    /// to the computation of the function’s return value.
    pub fn add_loop_deletion_pass(&self) {
        unsafe {
            LLVMAddLoopDeletionPass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_loop_idiom_pass(&self) {
        unsafe {
            LLVMAddLoopIdiomPass(self.pass_manager)
        }
    }

    /// A simple loop rotation transformation.
    pub fn add_loop_rotate_pass(&self) {
        unsafe {
            LLVMAddLoopRotatePass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_loop_reroll_pass(&self) {
        unsafe {
            LLVMAddLoopRerollPass(self.pass_manager)
        }
    }

    /// This pass implements a simple loop unroller.
    /// It works best when loops have been canonicalized
    /// by the [indvars](https://llvm.org/docs/Passes.html#passes-indvars)
    /// pass, allowing it to determine the trip counts
    /// of loops easily.
    pub fn add_loop_unroll_pass(&self) {
        unsafe {
            LLVMAddLoopUnrollPass(self.pass_manager)
        }
    }

    /// This pass transforms loops that contain branches on
    /// loop-invariant conditions to have multiple loops.
    /// For example, it turns the left into the right code:
    ///
    /// ```c
    /// for (...)                  if (lic)
    ///     A                          for (...)
    ///     if (lic)                       A; B; C
    ///         B                  else
    ///     C                          for (...)
    ///                                    A; C
    /// ```
    ///
    /// This can increase the size of the code exponentially
    /// (doubling it every time a loop is unswitched) so we
    /// only unswitch if the resultant code will be smaller
    /// than a threshold.
    ///
    /// This pass expects [LICM](https://llvm.org/docs/Passes.html#passes-licm)
    /// to be run before it to hoist invariant conditions
    /// out of the loop, to make the unswitching opportunity
    /// obvious.
    pub fn add_loop_unswitch_pass(&self) {
        unsafe {
            LLVMAddLoopUnswitchPass(self.pass_manager)
        }
    }

    /// This pass performs various transformations related
    /// to eliminating memcpy calls, or transforming sets
    /// of stores into memsets.
    pub fn add_memcpy_optimize_pass(&self) {
        unsafe {
            LLVMAddMemCpyOptPass(self.pass_manager)
        }
    }

    /// This pass performs partial inlining, typically by inlining
    /// an if statement that surrounds the body of the function.
    pub fn add_partially_inline_lib_calls_pass(&self) {
        unsafe {
            LLVMAddPartiallyInlineLibCallsPass(self.pass_manager)
        }
    }

    /// Rewrites switch instructions with a sequence of branches,
    /// which allows targets to get away with not implementing the
    /// switch instruction until it is convenient.
    pub fn add_lower_switch_pass(&self) {
        #[llvm_versions(3.6 => 6.0)]
        use llvm_sys::transforms::scalar::LLVMAddLowerSwitchPass;
        #[llvm_versions(7.0 => latest)]
        use llvm_sys::transforms::util::LLVMAddLowerSwitchPass;

        unsafe {
            LLVMAddLowerSwitchPass(self.pass_manager)
        }
    }

    /// This file promotes memory references to be register references.
    /// It promotes alloca instructions which only have loads and stores
    /// as uses. An alloca is transformed by using dominator frontiers
    /// to place phi nodes, then traversing the function in depth-first
    /// order to rewrite loads and stores as appropriate. This is just
    /// the standard SSA construction algorithm to construct "pruned" SSA form.
    pub fn add_promote_memory_to_register_pass(&self) {
        #[llvm_versions(3.6 => 6.0)]
        use llvm_sys::transforms::scalar::LLVMAddPromoteMemoryToRegisterPass;
        #[llvm_versions(7.0 => latest)]
        use llvm_sys::transforms::util::LLVMAddPromoteMemoryToRegisterPass;

        unsafe {
            LLVMAddPromoteMemoryToRegisterPass(self.pass_manager)
        }
    }

    /// This pass reassociates commutative expressions in an order that is designed
    /// to promote better constant propagation, GCSE, LICM, PRE, etc.
    ///
    /// For example: 4 + (x + 5) ⇒ x + (4 + 5)
    ///
    /// In the implementation of this algorithm, constants are assigned rank = 0,
    /// function arguments are rank = 1, and other values are assigned ranks
    /// corresponding to the reverse post order traversal of current function
    /// (starting at 2), which effectively gives values in deep loops higher
    /// rank than values not in loops.
    pub fn add_reassociate_pass(&self) {
        unsafe {
            LLVMAddReassociatePass(self.pass_manager)
        }
    }

    /// Sparse conditional constant propagation and merging, which can
    /// be summarized as:
    ///
    /// * Assumes values are constant unless proven otherwise
    /// * Assumes BasicBlocks are dead unless proven otherwise
    /// * Proves values to be constant, and replaces them with constants
    /// * Proves conditional branches to be unconditional
    ///
    /// Note that this pass has a habit of making definitions be dead.
    /// It is a good idea to run a DCE pass sometime after running this pass.
    pub fn add_sccp_pass(&self) {
        unsafe {
            LLVMAddSCCPPass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_scalar_repl_aggregates_pass(&self) {
        unsafe {
            LLVMAddScalarReplAggregatesPass(self.pass_manager)
        }
    }

    /// The well-known scalar replacement of aggregates transformation.
    /// This transform breaks up alloca instructions of aggregate type
    /// (structure or array) into individual alloca instructions for each
    /// member if possible. Then, if possible, it transforms the individual
    /// alloca instructions into nice clean scalar SSA form.
    pub fn add_scalar_repl_aggregates_pass_ssa(&self) {
        unsafe {
            LLVMAddScalarReplAggregatesPassSSA(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_scalar_repl_aggregates_pass_with_threshold(&self, threshold: i32) {
        unsafe {
            LLVMAddScalarReplAggregatesPassWithThreshold(self.pass_manager, threshold)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_simplify_lib_calls_pass(&self) {
        unsafe {
            LLVMAddSimplifyLibCallsPass(self.pass_manager)
        }
    }

    /// This file transforms calls of the current function (self recursion) followed
    /// by a return instruction with a branch to the entry of the function, creating
    /// a loop. This pass also implements the following extensions to the basic algorithm:
    ///
    /// 1. Trivial instructions between the call and return do not prevent the
    /// transformation from taking place, though currently the analysis cannot support
    /// moving any really useful instructions (only dead ones).
    ///
    /// 2. This pass transforms functions that are prevented from being tail
    /// recursive by an associative expression to use an accumulator variable, thus
    /// compiling the typical naive factorial or fib implementation into efficient code.
    ///
    /// 3. TRE is performed if the function returns void, if the return returns
    /// the result returned by the call, or if the function returns a run-time constant
    /// on all exits from the function. It is possible, though unlikely, that the return
    /// returns something else (like constant 0), and can still be TRE’d. It can be
    /// TRE'd if all other return instructions in the function return the exact same value.
    ///
    /// 4. If it can prove that callees do not access theier caller stack frame,
    /// they are marked as eligible for tail call elimination (by the code generator).
    pub fn add_tail_call_elimination_pass(&self) {
        unsafe {
            LLVMAddTailCallEliminationPass(self.pass_manager)
        }
    }

    /// This pass implements constant propagation and merging. It looks for instructions
    /// involving only constant operands and replaces them with a constant value instead
    /// of an instruction. For example:
    ///
    /// ```ir
    /// add i32 1, 2
    /// ```
    ///
    /// becomes
    ///
    /// ```ir
    /// i32 3
    /// ```
    ///
    /// NOTE: this pass has a habit of making definitions be dead. It is a good idea to
    /// run a Dead Instruction Elimination pass sometime after running this pass.
    pub fn add_constant_propagation_pass(&self) {
        unsafe {
            LLVMAddConstantPropagationPass(self.pass_manager)
        }
    }

    /// This file promotes memory references to be register references.
    /// It promotes alloca instructions which only have loads and stores
    /// as uses. An alloca is transformed by using dominator frontiers to
    /// place phi nodes, then traversing the function in depth-first order to
    /// rewrite loads and stores as appropriate. This is just the standard SSA
    /// construction algorithm to construct “pruned” SSA form.
    pub fn add_demote_memory_to_register_pass(&self) {
        unsafe {
            LLVMAddDemoteMemoryToRegisterPass(self.pass_manager)
        }
    }

    /// Verifies an LLVM IR code. This is useful to run after an optimization
    /// which is undergoing testing. Note that llvm-as verifies its input before
    /// emitting bitcode, and also that malformed bitcode is likely to make
    /// LLVM crash. All language front-ends are therefore encouraged to verify
    /// their output before performing optimizing transformations.
    ///
    /// 1. Both of a binary operator’s parameters are of the same type.
    ///
    /// 2. Verify that the indices of mem access instructions match other operands.
    ///
    /// 3. Verify that arithmetic and other things are only performed on
    /// first-class types. Verify that shifts and logicals only happen on
    /// integrals f.e.
    ///
    /// 4. All of the constants in a switch statement are of the correct type.
    ///
    /// 5. The code is in valid SSA form.
    ///
    /// 6. It is illegal to put a label into any other type (like a structure)
    /// or to return one.
    ///
    /// 7. Only phi nodes can be self referential: %x = add i32 %x, %x is invalid.
    ///
    /// 8. PHI nodes must have an entry for each predecessor, with no extras.
    ///
    /// 9. PHI nodes must be the first thing in a basic block, all grouped together.
    ///
    /// 10. PHI nodes must have at least one entry.
    ///
    /// 11. All basic blocks should only end with terminator insts, not contain them.
    ///
    /// 12. The entry node to a function must not have predecessors.
    ///
    /// 13. All Instructions must be embedded into a basic block.
    ///
    /// 14. Functions cannot take a void-typed parameter.
    ///
    /// 15. Verify that a function’s argument list agrees with its declared type.
    ///
    /// 16. It is illegal to specify a name for a void value.
    ///
    /// 17. It is illegal to have an internal global value with no initializer.
    ///
    /// 18. It is illegal to have a ret instruction that returns a value that does
    /// not agree with the function return value type.
    ///
    /// 19. Function call argument types match the function prototype.
    ///
    /// 20. All other things that are tested by asserts spread about the code.
    ///
    /// Note that this does not provide full security verification (like Java), but instead just tries to ensure that code is well-formed.
    pub fn add_verifier_pass(&self) {
        unsafe {
            LLVMAddVerifierPass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_correlated_value_propagation_pass(&self) {
        unsafe {
            LLVMAddCorrelatedValuePropagationPass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_early_cse_pass(&self) {
        unsafe {
            LLVMAddEarlyCSEPass(self.pass_manager)
        }
    }

    #[llvm_versions(4.0 => latest)]
    /// No LLVM documentation is available at this time.
    pub fn add_early_cse_mem_ssa_pass(&self) {
        use llvm_sys::transforms::scalar::LLVMAddEarlyCSEMemSSAPass;

        unsafe {
            LLVMAddEarlyCSEMemSSAPass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_lower_expect_intrinsic_pass(&self) {
        unsafe {
            LLVMAddLowerExpectIntrinsicPass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_type_based_alias_analysis_pass(&self) {
        unsafe {
            LLVMAddTypeBasedAliasAnalysisPass(self.pass_manager)
        }
    }

    /// No LLVM documentation is available at this time.
    pub fn add_scoped_no_alias_aa_pass(&self) {
        unsafe {
            LLVMAddScopedNoAliasAAPass(self.pass_manager)
        }
    }

    /// A basic alias analysis pass that implements identities
    /// (two different globals cannot alias, etc), but does no
    /// stateful analysis.
    pub fn add_basic_alias_analysis_pass(&self) {
        unsafe {
            LLVMAddBasicAliasAnalysisPass(self.pass_manager)
        }
    }

    #[llvm_versions(7.0 => latest)]
    pub fn add_aggressive_inst_combiner_pass(&self) {
        use llvm_sys::transforms::scalar::LLVMAddAggressiveInstCombinerPass;

        unsafe {
            LLVMAddAggressiveInstCombinerPass(self.pass_manager)
        }
    }

    #[llvm_versions(7.0 => latest)]
    pub fn add_loop_unroll_and_jam_pass(&self) {
        use llvm_sys::transforms::scalar::LLVMAddLoopUnrollAndJamPass;

        unsafe {
            LLVMAddLoopUnrollAndJamPass(self.pass_manager)
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

#[derive(Debug)]
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

    #[llvm_versions(7.0 => latest)]
    pub fn initialize_aggressive_inst_combiner(&self) {
        use llvm_sys::initialization::LLVMInitializeAggressiveInstCombiner;

        unsafe {
            LLVMInitializeAggressiveInstCombiner(self.pass_registry)
        }
    }
}
