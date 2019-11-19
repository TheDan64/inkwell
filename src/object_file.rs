use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(
        feature = "llvm3-6",
        feature = "llvm3-7",
        feature = "llvm3-8",
        feature = "llvm3-9",
        feature = "llvm4-0",
        feature = "llvm5-0",
        feature = "llvm6-0",
        feature = "llvm7-0",
        feature = "llvm8-0",
    ))] {
        mod object_file;
        pub use self::object_file::{Binary, BinaryRef, ObjectFile};
    } else {
        mod binary;
        pub use self::binary::{Binary, BinaryRef, BinaryType};
    }
}
mod section;
mod relocation;
mod symbol;

pub use self::section::{Section, SectionIterator};
pub use self::relocation::{Relocation, RelocationIterator};
pub use self::symbol::{Symbol, SymbolIterator};
