//! These macros are only intended to be used by inkwell internally
//! and should not be expected to have public support nor stability.
//! Here be dragons 🐉

extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::parse::{Parse, ParseStream, Result, Error};
use syn::{Token, LitFloat, Ident, Item};

// We could include_str! inkwell's Cargo.toml and extract these features
// so that they don't need to also be managed here...
const ALL_FEATURE_VERSIONS: [&str; 9] = [
    "llvm3-6",
    "llvm3-7",
    "llvm3-8",
    "llvm3-9",
    "llvm4-0",
    "llvm5-0",
    "llvm6-0",
    "llvm7-0",
    "llvm8-0",
];

fn panic_with_usage() -> ! {
    panic!("llvm_versions must take the form: X.Y or X.Y => (X.Y || latest)");
}

fn get_latest_feature_index() -> usize {
    ALL_FEATURE_VERSIONS.len() - 1
}

fn get_feature_slice(vt: VersionType) -> Option<&'static [&'static str]> {
    match vt {
        VersionType::Specific(version) => {
            let feature = f64_to_feature_string(version);
            let index = ALL_FEATURE_VERSIONS.iter().position(|&s| s == feature)?;
            Some(&ALL_FEATURE_VERSIONS[index..index])
        }
        VersionType::RangeToLatest(version) => {
            let latest = get_latest_feature_index();
            let feature = f64_to_feature_string(version);
            let index = ALL_FEATURE_VERSIONS.iter().position(|&s| s == feature)?;
            Some(&ALL_FEATURE_VERSIONS[index..=latest])
        }
        VersionType::Range(start, end) => {
            let start_feature = f64_to_feature_string(start);
            let end_feature = f64_to_feature_string(end);
            let start_index = ALL_FEATURE_VERSIONS.iter().position(|&s| s == start_feature)?;
            let end_index = ALL_FEATURE_VERSIONS.iter().position(|&s| s == end_feature)?;
            if end_index < start_index {
                return None;
            }
            Some(&ALL_FEATURE_VERSIONS[start_index..=end_index])
        }
    }
}

fn f64_to_feature_string(float: f64) -> String {
    let int = float as u64;

    format!("llvm{}-{}", int, (float * 10.) % 10.)
}

enum VersionType {
    Specific(f64),
    RangeToLatest(f64),
    Range(f64, f64),
}
impl Parse for VersionType {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitFloat) {
            let from = input.parse::<LitFloat>().unwrap().value();
            if input.is_empty() {
                return Ok(VersionType::Specific(from));
            }
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![=>]) {
                let _: Token![=>] = input.parse().unwrap();
                let lookahead = input.lookahead1();
                if lookahead.peek(Ident) {
                    let to = input.parse::<Ident>().unwrap();
                    if to.to_string() == "latest" {
                        Ok(VersionType::RangeToLatest(from))
                    } else {
                        Err(Error::new(to.span(), "expected `latest`"))
                    }
                } else if lookahead.peek(LitFloat) {
                    let to = input.parse::<LitFloat>().unwrap().value();
                    Ok(VersionType::Range(from, to))
                } else {
                    Err(lookahead.error())
                }
            } else {
                Err(lookahead.error())
            }
        } else {
            Err(lookahead.error())
        }
    }
}

#[proc_macro_attribute]
pub fn llvm_versions(attribute_args: TokenStream, attributee: TokenStream) -> TokenStream {
    let vt = parse_macro_input!(attribute_args as VersionType);

    let features = get_feature_slice(vt)
        .unwrap_or_else(|| panic_with_usage());

    let attributee: Item = syn::parse(attributee).expect("Could not parse attributed item");

    let q = quote! {
        #[cfg(any(#(feature = #features),*))]
        #attributee
    };

    q.into()
}
