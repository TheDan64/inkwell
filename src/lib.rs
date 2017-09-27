extern crate either;
#[macro_use]
extern crate enum_methods;
extern crate libc;
extern crate llvm_sys;

pub mod basic_block;
pub mod builder;
pub mod context;
pub mod data_layout;
pub mod execution_engine;
pub mod memory_buffer;
pub mod module;
pub mod object_file;
pub mod passes;
pub mod targets;
pub mod types;
pub mod values;

use llvm_sys::{LLVMIntPredicate, LLVMRealPredicate};

// TODO: Probably move into error handling module
pub fn enable_llvm_pretty_stack_trace() {
    // use llvm_sys::error_handling::LLVMEnablePrettyStackTrace; // v3.8
    use llvm_sys::core::LLVMEnablePrettyStackTrace;

    unsafe {
        LLVMEnablePrettyStackTrace()
    }
}

// TODO: Move
pub fn is_multithreaded() -> bool {
    use llvm_sys::core::LLVMIsMultithreaded;

    unsafe {
        LLVMIsMultithreaded() == 1
    }
}

// TODO: Move to better location?
// REVIEW: Not sure how safe this is. What happens when you make an llvm call after
// shutdown_llvm has been called?
pub fn shutdown_llvm() {
    use llvm_sys::core::LLVMShutdown;

    unsafe {
        LLVMShutdown()
    }
}

// REVIEW: Maybe this belongs in some sort of prelude?
pub enum IntPredicate {
    EQ,
    NE,
    UGT,
    UGE,
    ULT,
    ULE,
    SGT,
    SGE,
    SLT,
    SLE,
}

impl IntPredicate {
    pub(crate) fn as_llvm_predicate(&self) -> LLVMIntPredicate {
        match *self {
            IntPredicate::EQ => LLVMIntPredicate::LLVMIntEQ,
            IntPredicate::NE => LLVMIntPredicate::LLVMIntNE,
            IntPredicate::UGT => LLVMIntPredicate::LLVMIntUGT,
            IntPredicate::UGE => LLVMIntPredicate::LLVMIntUGE,
            IntPredicate::ULT => LLVMIntPredicate::LLVMIntULT,
            IntPredicate::ULE => LLVMIntPredicate::LLVMIntULE,
            IntPredicate::SGT => LLVMIntPredicate::LLVMIntSGT,
            IntPredicate::SGE => LLVMIntPredicate::LLVMIntSGE,
            IntPredicate::SLT => LLVMIntPredicate::LLVMIntSLT,
            IntPredicate::SLE => LLVMIntPredicate::LLVMIntSLE,
        }
    }
}

// REVIEW: Maybe this belongs in some sort of prelude?
pub enum FloatPredicate {
    PredicateFalse,
    OEQ,
    OGT,
    OGE,
    OLT,
    OLE,
    ONE,
    ORD,
    UNO,
    UEQ,
    UGT,
    UGE,
    ULT,
    ULE,
    UNE,
    PredicateTrue,
}

impl FloatPredicate {
    pub(crate) fn as_llvm_predicate(&self) -> LLVMRealPredicate {
        match *self {
            FloatPredicate::PredicateFalse => LLVMRealPredicate::LLVMRealPredicateFalse,
            FloatPredicate::OEQ => LLVMRealPredicate::LLVMRealOEQ,
            FloatPredicate::OGT => LLVMRealPredicate::LLVMRealOGT,
            FloatPredicate::OGE => LLVMRealPredicate::LLVMRealOGE,
            FloatPredicate::OLT => LLVMRealPredicate::LLVMRealOLT,
            FloatPredicate::OLE => LLVMRealPredicate::LLVMRealOLE,
            FloatPredicate::ONE => LLVMRealPredicate::LLVMRealONE,
            FloatPredicate::ORD => LLVMRealPredicate::LLVMRealORD,
            FloatPredicate::UNO => LLVMRealPredicate::LLVMRealUNO,
            FloatPredicate::UEQ => LLVMRealPredicate::LLVMRealUEQ,
            FloatPredicate::UGT => LLVMRealPredicate::LLVMRealUGT,
            FloatPredicate::UGE => LLVMRealPredicate::LLVMRealUGE,
            FloatPredicate::ULT => LLVMRealPredicate::LLVMRealULT,
            FloatPredicate::ULE => LLVMRealPredicate::LLVMRealULE,
            FloatPredicate::UNE => LLVMRealPredicate::LLVMRealUNE,
            FloatPredicate::PredicateTrue => LLVMRealPredicate::LLVMRealPredicateTrue,
        }
    }
}


// Misc Notes
// Always pass a c_string.as_ptr() call into the function call directly and never
// before hand. Seems to make a huge difference (stuff stops working) otherwise

// Initializer (new) strategy:
// assert!(!val.is_null()); where null is not expected to ever occur, but Option<Self>
// when null is expected to be passed at some point
