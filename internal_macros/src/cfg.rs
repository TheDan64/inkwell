use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::cmp::Ordering;
use syn::fold::Fold;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Attribute, Field, Item, Token, Variant};
use syn::{Lit, RangeLimits};

// This array should match the LLVM features in the top level Cargo manifest
const FEATURE_VERSIONS: &[&str] = &[
    "llvm8-0", "llvm9-0", "llvm10-0", "llvm11-0", "llvm12-0", "llvm13-0", "llvm14-0", "llvm15-0", "llvm16-0",
    "llvm17-0", "llvm18-1",
];

pub struct VersionRange {
    start: Option<Version>,
    limits: RangeLimits,
    end: Option<Version>,
    features: &'static [&'static str],
}

impl Parse for VersionRange {
    fn parse(input: ParseStream) -> Result<Self> {
        let start = if input.peek(Token![..]) || input.peek(Token![..=]) {
            None
        } else {
            Some(input.parse()?)
        };
        let limits = input.parse::<RangeLimits>()?;
        let end = if matches!(limits, RangeLimits::HalfOpen(_)) && (input.is_empty() || input.peek(Token![,])) {
            None
        } else {
            Some(input.parse()?)
        };
        let mut this = Self {
            start,
            limits,
            end,
            features: &[],
        };
        this.features = this.get_features()?;
        Ok(this)
    }
}

impl VersionRange {
    fn doc_cfg(&self) -> Option<TokenStream> {
        if cfg!(feature = "nightly") {
            let features = self.features;
            Some(quote! {
                #[doc(cfg(any(#(feature = #features),*)))]
            })
        } else {
            None
        }
    }

    fn cfg(&self) -> TokenStream {
        let features = self.features;
        quote! {
            #[cfg(any(#(feature = #features),*))]
        }
    }

    fn get_features(&self) -> Result<&'static [&'static str]> {
        let features = FEATURE_VERSIONS;
        let start = self.start.as_ref().map(|v| v.get_index(features)).transpose()?;
        if let Some(0) = start {
            let min = features[0];
            return Err(Error::new(
                self.start.as_ref().unwrap().span,
                format!("start version is the same as the minimum version ({min})"),
            ));
        }
        let start = start.unwrap_or(0);

        let end = self.end.as_ref().map(|v| v.get_index(features)).transpose()?;
        if let Some(end) = end {
            let span = || self.end.as_ref().unwrap().span;
            match end.cmp(&start) {
                Ordering::Less => return Err(Error::new(span(), "end version is before start version")),
                Ordering::Equal => return Err(Error::new(span(), "start and end versions are the same")),
                Ordering::Greater => {},
            }
        }
        let selected = match (self.limits, end) {
            (RangeLimits::Closed(_), end) => &features[start..=end.expect("already checked")],
            (RangeLimits::HalfOpen(_), None) => &features[start..],
            (RangeLimits::HalfOpen(_), Some(end)) => &features[start..end],
        };
        if selected.len() == features.len() {
            return Err(Error::new(self.span(), "selected all features, remove this attribute"));
        }
        Ok(selected)
    }

    fn span(&self) -> Span {
        match (&self.start, &self.end) {
            (Some(start), Some(end)) => start.span.join(end.span).unwrap(),
            (Some(start), None) => start.span,
            (None, Some(end)) => end.span,
            (None, None) => self.limits.span(),
        }
    }
}

struct Version {
    major: u32,
    minor: u32,
    span: Span,
}

fn default_minor(major: u32) -> u32 {
    if major >= 18 {
        1
    } else {
        0
    }
}

impl Parse for Version {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit = input.parse::<Lit>()?;
        let (major, minor) = match &lit {
            Lit::Int(int) => {
                let major = int.base10_parse()?;
                (major, default_minor(major))
            },
            Lit::Float(float) => {
                let s = float.base10_digits();
                let mut parts = s.split('.');
                let major = parts
                    .next()
                    .unwrap()
                    .parse()
                    .map_err(|e| syn::Error::new(float.span(), e))?;
                let minor = if let Some(minor) = parts.next() {
                    minor.parse().map_err(|e| syn::Error::new(float.span(), e))?
                } else {
                    default_minor(major)
                };
                (major, minor)
            },
            _ => return Err(Error::new(lit.span(), "expected integer or float")),
        };
        Ok(Self {
            major,
            minor,
            span: lit.span(),
        })
    }
}

impl Version {
    fn get_index(&self, features: &[&str]) -> Result<usize> {
        let feature = self.as_feature();
        match features.iter().position(|&s| s == feature) {
            None => Err(Error::new(self.span, format!("undefined feature version: {feature:?}"))),
            Some(index) => Ok(index),
        }
    }

    fn as_feature(&self) -> String {
        format!("llvm{}-{}", self.major, self.minor)
    }
}

/// Folder for expanding `llvm_versions` attributes.
pub struct VersionFolder {
    result: Result<()>,
}

impl VersionFolder {
    pub fn new() -> Self {
        Self { result: Ok(()) }
    }

    pub fn fold_any<T>(f: impl FnOnce(&mut Self, T) -> T, t: T) -> Result<T> {
        let mut folder = VersionFolder::new();
        let t = f(&mut folder, t);
        folder.result?;
        Ok(t)
    }

    fn has_error(&self) -> bool {
        self.result.is_err()
    }

    fn expand_llvm_versions_attr(&mut self, attr: &Attribute) -> Attribute {
        // Make no modifications if we've generated an error
        if self.has_error() {
            return attr.clone();
        }

        // If this isn't an llvm_versions attribute, skip it
        if !attr.path().is_ident("llvm_versions") {
            return attr.clone();
        }

        // Expand from llvm_versions to raw cfg attribute
        match attr.parse_args::<VersionRange>() {
            Ok(version_range) => {
                let cfg = version_range.cfg();
                syn::parse_quote!(#cfg)
            },
            Err(err) => {
                self.result = Err(err);
                attr.clone()
            },
        }
    }
}

impl Fold for VersionFolder {
    fn fold_variant(&mut self, mut variant: Variant) -> Variant {
        if self.has_error() {
            return variant;
        }

        let attrs = variant
            .attrs
            .iter()
            .map(|attr| self.expand_llvm_versions_attr(attr))
            .collect::<Vec<_>>();
        variant.attrs = attrs;
        variant
    }

    fn fold_field(&mut self, mut field: Field) -> Field {
        if self.has_error() {
            return field;
        }

        let attrs = field
            .attrs
            .iter()
            .map(|attr| self.expand_llvm_versions_attr(attr))
            .collect::<Vec<_>>();
        field.attrs = attrs;
        field
    }
}

pub fn expand(args: Option<VersionRange>, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_clone = input.clone();
    let item = syn::parse_macro_input!(input as Item);
    let args = match args {
        Some(args) => {
            let cfg = args.cfg();
            let doc_cfg = args.doc_cfg();
            quote! { #cfg #doc_cfg }
        },
        None => quote! {},
    };
    match VersionFolder::fold_any(Fold::fold_item, item) {
        Ok(item) => quote! { #args #item },
        Err(err) => {
            let err = err.into_compile_error();
            let input = proc_macro2::TokenStream::from(input_clone);
            quote! { #err #args #input }
        },
    }
    .into()
}
