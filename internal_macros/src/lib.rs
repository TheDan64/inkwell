//! These macros are only intended to be used by inkwell internally
//! and should not be expected to have public support nor stability.
//! Here be dragons 🐉

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod cfg;
mod r#enum;

/// This macro can be used to specify version constraints for an enum/struct/union or
/// other item which can be decorated with an attribute.
///
/// It takes one argument which is any range of major or `major.minor` LLVM versions.
///
/// To use with enum variants or struct fields, you need to decorate the parent item with
/// the `#[llvm_versioned_item]` attribute, as this is the hook we need to modify the AST
/// of those items.
///
/// # Examples
///
/// ```ignore
/// // Inclusive range from 15 up to and including 18.
/// #[llvm_versions(15..=18)]
///
/// // Exclusive range from 15 up to but not including 18.
/// #[llvm_versions(15..18)]
///
/// // Inclusive range from 15.1 up to and including the latest release.
/// #[llvm_versions(15.1..)]
/// ```
#[proc_macro_attribute]
pub fn llvm_versions(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as cfg::VersionRange);
    cfg::expand(Some(args), input)
}

/// This attribute is used to decorate enums, structs, or unions which may contain
/// variants/fields which make use of `#[llvm_versions(..)]`
///
/// # Examples
///
/// ```ignore
/// #[llvm_versioned_item]
/// enum InstructionOpcode {
///     Call,
///     #[llvm_versions(3.8..=latest)]
///     CatchPad,
///     ...
/// }
/// ```
#[proc_macro_attribute]
pub fn llvm_versioned_item(args: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(args as syn::parse::Nothing);
    cfg::expand(None, input)
}

/// This attribute macro allows you to decorate an enum declaration which represents
/// an LLVM enum with versioning constraints and/or custom variant names. There are
/// a few expectations around the LLVM and Rust enums:
///
/// - Both enums have the same number of variants
/// - The name of the LLVM variant can be derived by appending 'LLVM' to the Rust variant
///
/// The latter can be worked around manually with `#[llvm_variant]` if desired.
///
/// # Examples
///
/// ```ignore
/// #[llvm_enum(LLVMOpcode)]
/// enum InstructionOpcode {
///     Call,
///     #[llvm_versions(3.8..)]
///     CatchPad,
///     ...,
///     #[llvm_variant(LLVMRet)]
///     Return,
///     ...
/// }
/// ```
///
/// The use of `#[llvm_variant(NAME)]` allows you to override the default
/// naming scheme by providing the variant name which the source enum maps
/// to. In the above example, `Ret` was deemed unnecessarily concise, so the
/// source variant is named `Return` and mapped manually to `LLVMRet`.
#[proc_macro_attribute]
pub fn llvm_enum(args: TokenStream, input: TokenStream) -> TokenStream {
    // Expect something like #[llvm_enum(LLVMOpcode)]
    let llvm_ty = parse_macro_input!(args as syn::Path);
    let llvm_enum_type = parse_macro_input!(input as r#enum::LLVMEnumType);
    r#enum::llvm_enum(llvm_ty, llvm_enum_type).into()
}
