//! These macros are only intended to be used by inkwell internally
//! and should not be expected to have public support nor stability
//! Here be dragons

extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use syn::{Arm, Expr, ExprLit, Item, Lit, punctuated::Pair, Pat, PatLit};

// We could include_str! inkwell's Cargo.toml and extract these features
// so that they don't need to also be managed here...
const ALL_FEATURE_VERSIONS: [&str; 8] = [
    "llvm3-6",
    "llvm3-7",
    "llvm3-8",
    "llvm3-9",
    "llvm4-0",
    "llvm5-0",
    "llvm6-0",
    "llvm7-0",
];

fn panic_with_usage() -> ! {
    panic!("feature_version must take the form: \"llvmX-Y\" => (\"llvmX-Y\" || latest)");
}

fn get_latest_feature() -> String {
    ALL_FEATURE_VERSIONS[ALL_FEATURE_VERSIONS.len() - 1].into()
}

#[proc_macro_attribute]
pub fn feature_versions(attribute_args: TokenStream, attributee: TokenStream) -> TokenStream {
    let arm = parse_macro_input!(attribute_args as Arm);

    if arm.pats.len() != 1 {
        panic_with_usage();
    }

    let start_feature = match arm.pats.first().unwrap() {
        Pair::End(Pat::Lit(PatLit { expr })) => match **expr {
            Expr::Lit(ExprLit { lit: Lit::Str(ref literal_str), .. }) => literal_str.value(),
            _ => panic_with_usage(),
        },
        _ => panic_with_usage(),
    };
    let end_feature = match *arm.body {
        Expr::Lit(ExprLit { lit: Lit::Str(literal_str), .. }) => Some(literal_str.value()),
        // We could assert the path is just "latest" but this seems like a lot of extra work...
        Expr::Path(_) => None,
        _ => panic_with_usage(),
    }.unwrap_or_else(get_latest_feature);

    let start_index = ALL_FEATURE_VERSIONS.iter().position(|&str| str == start_feature).unwrap();
    let end_index = ALL_FEATURE_VERSIONS.iter().position(|&str| str == end_feature).unwrap();
    let features = &ALL_FEATURE_VERSIONS[start_index..=end_index];
    let attributee: Item = syn::parse(attributee).expect("Could not parse attributed item");

    let q = quote! {
        #[cfg(any(#(feature = #features),*))]
        #attributee
    };

    q.into()
}
