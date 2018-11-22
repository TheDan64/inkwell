//! These macros are only intended to be used by inkwell internally
//! and should not be expected to have public support nor stability.
//! Here be dragons 🐉

extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Arm, Expr, ExprLit, Item, Lit, punctuated::Pair, Pat, PatLit, parse_macro_input};

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
    panic!("llvm_versions must take the form: X.Y => (X.Y || latest)");
}

fn get_latest_feature() -> String {
    ALL_FEATURE_VERSIONS[ALL_FEATURE_VERSIONS.len() - 1].into()
}

fn get_feature_slice(start_feature: &str, end_feature: &str) -> Option<&'static [&'static str]> {
    let start_index = ALL_FEATURE_VERSIONS.iter().position(|&str| str == start_feature)?;
    let end_index = ALL_FEATURE_VERSIONS.iter().position(|&str| str == end_feature)?;

    if end_index < start_index {
        return None;
    }

    Some(&ALL_FEATURE_VERSIONS[start_index..=end_index])
}

fn f64_to_feature_string(float: f64) -> String {
    let int = float as u64;

    format!("llvm{}-{}", int, (float * 10.) % 10.)
}

#[proc_macro_attribute]
pub fn llvm_versions(attribute_args: TokenStream, attributee: TokenStream) -> TokenStream {
    let arm = parse_macro_input!(attribute_args as Arm);

    if arm.pats.len() != 1 {
        panic_with_usage();
    }

    let start_feature = match arm.pats.first().unwrap() {
        Pair::End(Pat::Lit(PatLit { expr })) => match **expr {
            Expr::Lit(ExprLit { lit: Lit::Float(ref literal_float), .. }) => f64_to_feature_string(literal_float.value()),
            _ => panic_with_usage(),
        },
        _ => panic_with_usage(),
    };
    let end_feature = match *arm.body {
        Expr::Lit(ExprLit { lit: Lit::Float(literal_float), .. }) => Some(f64_to_feature_string(literal_float.value())),
        // We could assert the path is just "latest" but this seems like a lot of extra work...
        Expr::Path(_) => None,
        _ => panic_with_usage(),
    }.unwrap_or_else(get_latest_feature);

    let features = get_feature_slice(&start_feature, &end_feature).unwrap_or_else(|| panic_with_usage());
    let attributee: Item = syn::parse(attributee).expect("Could not parse attributed item");

    let q = quote! {
        #[cfg(any(#(feature = #features),*))]
        #attributee
    };

    q.into()
}
