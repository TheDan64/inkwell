
/// Errors for operations involving alignment.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AlignmentError {
    #[error("{0} is not a power of two and cannot be used for alignment")]
    NonPowerOfTwo(u32),
    #[error("The src_align_bytes argument was not a power of two.")]
    SrcNonPowerOfTwo(u32),
    #[error("The dest_align_bytes argument was not a power of two.")]
    DestNonPowerOfTwo(u32),
    #[error(
        "Type is unsized and cannot be aligned. \
    Suggestion: Align memory manually."
    )]
    Unsized,
    #[error("Value is not an alloca, load, or store instruction.")]
    UnalignedInstruction,
}

/// The top-level Error type for the inkwell crate.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("Builder Error: {0}")]
    BuilderError(#[from] crate::builder::BuilderError),
    #[error("InstructionValue Error: {0}")]
    InstructionValueError(#[from] crate::values::InstructionValueError),
    #[error("Basic types must have names.")]
    EmptyNameError,
    #[error("Metadata is expected to be a node.")]
    GlobalMetadataError,
}
