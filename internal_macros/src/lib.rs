//! These macros are only intended to be used by inkwell internally
//! and should not be expected to have public support nor stability.
//! Here be dragons 🐉

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use std::iter::IntoIterator;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse_macro_input;
use syn::parse::{Parse, ParseStream, Result, Error};
use syn::{Token, LitFloat, Ident, Item};

const FEATURES_ENV: &'static str = env!("INKWELL_FEATURES");

// Fetches a vector of feature version strings, e.g. llvm8-0
fn get_feature_versions() -> Vec<&'static str> {
    FEATURES_ENV
        .split(',')
        .collect()
}

// Gets the index of the feature version that represents `latest`
fn get_latest_feature_index(features: &[&str]) -> usize {
    features.len() - 1
}

// Gets the index of the feature version that matches the input string
fn get_feature_index(features: &[&str], feature: String, span: Span) -> Result<usize> {
    let feat = feature.as_str();
    match features.iter().position(|&s| s == feat) {
        None => Err(Error::new(span, format!("Invalid feature version: {}, not defined", feature))),
        Some(index) => Ok(index)
    }
}

// Gets a vector of feature versions represented by the given VersionType
fn get_features(vt: VersionType) -> Result<Vec<&'static str>> {
    let features = get_feature_versions();
    let latest = get_latest_feature_index(&features);
    match vt {
        VersionType::Specific(version, span) => {
            let feature = f64_to_feature_string(version);
            let index = get_feature_index(&features, feature, span.clone())?;
            Ok(features[index..=index].to_vec())
        }
        VersionType::InclusiveRangeToLatest(version, span) => {
            let feature = f64_to_feature_string(version);
            let index = get_feature_index(&features, feature, span.clone())?;
            Ok(features[index..=latest].to_vec())
        }
        VersionType::InclusiveRange((start, start_span), (end, end_span)) => {
            let start_feature = f64_to_feature_string(start);
            let end_feature = f64_to_feature_string(end);
            let start_index = get_feature_index(&features, start_feature, start_span.clone())?;
            let end_index = get_feature_index(&features, end_feature, end_span.clone())?;
            if end_index < start_index {
                let message = format!("Invalid version range: {} must be greater than or equal to {}", start, end);
                Err(Error::new(end_span, message))
            } else {
                Ok(features[start_index..=end_index].to_vec())
            }
        }
        VersionType::ExclusiveRangeToLatest(version, span) => {
            let feature = f64_to_feature_string(version);
            let index = get_feature_index(&features, feature, span.clone())?;
            if latest == index {
                let message = format!("Invalid version range: {}..latest produces an empty feature set", version);
                Err(Error::new(span, message))
            } else {
                Ok(features[index..latest].to_vec())
            }
        }
        VersionType::ExclusiveRange((start, start_span), (end, end_span)) => {
            let start_feature = f64_to_feature_string(start);
            let end_feature = f64_to_feature_string(end);
            let start_index = get_feature_index(&features, start_feature, start_span.clone())?;
            let end_index = get_feature_index(&features, end_feature, end_span.clone())?;
            if end_index == start_index {
                let message = format!("Invalid version range: {}..{} produces an empty feature set", start, end);
                Err(Error::new(start_span, message))
            } else if end_index < start_index {
                let message = format!("Invalid version range: {} must be greater than {}", start, end);
                Err(Error::new(end_span, message))
            } else {
                Ok(features[start_index..end_index].to_vec())
            }
        }
    }
}

// Converts a version number as a float to its feature version 
// string form (e.g. 8.0 => llvm8-0)
fn f64_to_feature_string(float: f64) -> String {
    let int = float as u64;

    format!("llvm{}-{}", int, (float * 10.) % 10.)
}

// This struct defines the type of version expressions parsable by `llvm_versions`
#[derive(Debug)]
enum VersionType {
    Specific(f64, Span),
    InclusiveRange((f64, Span), (f64, Span)),
    InclusiveRangeToLatest(f64, Span),
    ExclusiveRange((f64, Span), (f64, Span)),
    ExclusiveRangeToLatest(f64, Span),
}
impl Parse for VersionType {
    fn parse(input: ParseStream) -> Result<Self> {
        // We use lookahead to produce better syntax errors
        let lookahead = input.lookahead1();
        // All version specifiers begin with a float
        if lookahead.peek(LitFloat) {
            let from = input.parse::<LitFloat>().unwrap();
            let from_val = from.value();
            // If that's the end of the input, this was a specific version string
            if input.is_empty() {
                return Ok(VersionType::Specific(from_val, from.span()));
            }
            // If the next token is ..= it is an inclusive range, .. is exclusive
            // In both cases the right-hand operand must be either a float or an ident, `latest`
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![..=]) {
                let _: Token![..=] = input.parse().unwrap();
                let lookahead = input.lookahead1();
                if lookahead.peek(Ident) {
                    let to = input.parse::<Ident>().unwrap();
                    if to.to_string() == "latest" {
                        Ok(VersionType::InclusiveRangeToLatest(from_val, from.span()))
                    } else {
                        Err(Error::new(to.span(), "expected `latest` or `X.Y`"))
                    }
                } else if lookahead.peek(LitFloat) {
                    let to = input.parse::<LitFloat>().unwrap();
                    let to_val = to.value();
                    Ok(VersionType::InclusiveRange((from_val, from.span()), (to_val, to.span())))
                } else {
                    Err(lookahead.error())
                }
            } else if lookahead.peek(Token![..]) {
                let _: Token![..] = input.parse().unwrap();
                let lookahead = input.lookahead1();
                if lookahead.peek(Ident) {
                    let to = input.parse::<Ident>().unwrap();
                    if to.to_string() == "latest" {
                        Ok(VersionType::ExclusiveRangeToLatest(from_val, from.span()))
                    } else {
                        Err(Error::new(to.span(), "expected `latest` or `X.Y`"))
                    }
                } else if lookahead.peek(LitFloat) {
                    let to = input.parse::<LitFloat>().unwrap();
                    let to_val = to.value();
                    Ok(VersionType::ExclusiveRange((from_val, from.span()), (to_val, to.span())))
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

#[derive(Debug)]
struct FeatureSet(Vec<&'static str>);
impl Parse for FeatureSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let version_type = input.parse::<VersionType>()?;
        let features = get_features(version_type)?;
        Ok(Self(features))
    }
}
impl IntoIterator for FeatureSet {
    type Item = &'static str;
    type IntoIter = std::vec::IntoIter<&'static str>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[proc_macro_attribute]
pub fn llvm_versions(attribute_args: TokenStream, attributee: TokenStream) -> TokenStream {
    let features = parse_macro_input!(attribute_args as FeatureSet);

    let attributee: Item = syn::parse(attributee).expect("Could not parse attributed item");

    let q = quote! {
        #[cfg(any(#(feature = #features),*))]
        #attributee
    };

    q.into()
}
