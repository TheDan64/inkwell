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
use syn::{parse_quote, parse_macro_input, parenthesized};
use syn::parse::{Parse, ParseStream, Result, Error};
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{Token, LitFloat, Ident, Item, Field, Variant, Attribute};

// This array should match the LLVM features in the top level Cargo manifest
const FEATURE_VERSIONS: [&str; 14] =
    ["llvm3-6", "llvm3-7", "llvm3-8", "llvm3-9", "llvm4-0", "llvm5-0", "llvm6-0", "llvm7-0", "llvm8-0", "llvm9-0", "llvm10-0", "llvm11-0", "llvm12-0", "llvm13-0"];

/// Gets the index of the feature version that represents `latest`
fn get_latest_feature_index(features: &[&str]) -> usize {
    features.len() - 1
}

/// Gets the index of the feature version that matches the input string
fn get_feature_index(features: &[&str], feature: String, span: Span) -> Result<usize> {
    let feat = feature.as_str();
    match features.iter().position(|&s| s == feat) {
        None => Err(Error::new(span, format!("Invalid feature version: {}, not defined", feature))),
        Some(index) => Ok(index)
    }
}

/// Gets a vector of feature versions represented by the given VersionType
fn get_features(vt: VersionType) -> Result<Vec<&'static str>> {
    let features = FEATURE_VERSIONS;
    let latest = get_latest_feature_index(&features);
    match vt {
        VersionType::Specific(version, span) => {
            let feature = f64_to_feature_string(version);
            let index = get_feature_index(&features, feature, span)?;
            Ok(features[index..=index].to_vec())
        }
        VersionType::InclusiveRangeToLatest(version, span) => {
            let feature = f64_to_feature_string(version);
            let index = get_feature_index(&features, feature, span)?;
            Ok(features[index..=latest].to_vec())
        }
        VersionType::InclusiveRange((start, start_span), (end, end_span)) => {
            let start_feature = f64_to_feature_string(start);
            let end_feature = f64_to_feature_string(end);
            let start_index = get_feature_index(&features, start_feature, start_span)?;
            let end_index = get_feature_index(&features, end_feature, end_span)?;
            if end_index < start_index {
                let message = format!("Invalid version range: {} must be greater than or equal to {}", start, end);
                Err(Error::new(end_span, message))
            } else {
                Ok(features[start_index..=end_index].to_vec())
            }
        }
        VersionType::ExclusiveRangeToLatest(version, span) => {
            let feature = f64_to_feature_string(version);
            let index = get_feature_index(&features, feature, span)?;
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
            let start_index = get_feature_index(&features, start_feature, start_span)?;
            let end_index = get_feature_index(&features, end_feature, end_span)?;
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

/// Converts a version number as a float to its feature version
/// string form (e.g. 8.0 ..= llvm8-0)
fn f64_to_feature_string(float: f64) -> String {
    let int = float as u64;

    format!("llvm{}-{}", int, (float * 10.) % 10.)
}

/// This struct defines the type of version expressions parsable by `llvm_versions`
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
            let from_val = from.base10_parse().unwrap();
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
                    if to == "latest" {
                        Ok(VersionType::InclusiveRangeToLatest(from_val, from.span()))
                    } else {
                        Err(Error::new(to.span(), "expected `latest` or `X.Y`"))
                    }
                } else if lookahead.peek(LitFloat) {
                    let to = input.parse::<LitFloat>().unwrap();
                    let to_val = to.base10_parse().unwrap();
                    Ok(VersionType::InclusiveRange((from_val, from.span()), (to_val, to.span())))
                } else {
                    Err(lookahead.error())
                }
            } else if lookahead.peek(Token![..]) {
                let _: Token![..] = input.parse().unwrap();
                let lookahead = input.lookahead1();
                if lookahead.peek(Ident) {
                    let to = input.parse::<Ident>().unwrap();
                    if to == "latest" {
                        Ok(VersionType::ExclusiveRangeToLatest(from_val, from.span()))
                    } else {
                        Err(Error::new(to.span(), "expected `latest` or `X.Y`"))
                    }
                } else if lookahead.peek(LitFloat) {
                    let to = input.parse::<LitFloat>().unwrap();
                    let to_val = to.base10_parse().unwrap();
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

/// Handler for parsing of TokenStreams contained in item attributes
#[derive(Debug)]
struct ParenthesizedFeatureSet(FeatureSet);
impl Parse for ParenthesizedFeatureSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _ = parenthesized!(content in input);
        let features = content.parse::<FeatureSet>()?;
        Ok(Self(features))
    }
}

/// Handler for parsing of TokenStreams from macro input
#[derive(Clone, Debug)]
struct FeatureSet(std::vec::IntoIter<&'static str>, Option<Error>);
impl Default for FeatureSet {
    fn default() -> Self {
        // Default to all versions
        Self(FEATURE_VERSIONS.to_vec().into_iter(), None)
    }
}
impl Parse for FeatureSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let version_type = input.parse::<VersionType>()?;
        let features = get_features(version_type)?;
        Ok(Self(features.into_iter(), None))
    }
}
impl Iterator for FeatureSet {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
impl FeatureSet {
    #[inline]
    fn has_error(&self) -> bool {
        self.1.is_some()
    }

    #[inline]
    fn set_error(&mut self, err: Error) {
        self.1 = Some(err);
    }

    fn into_error(self) -> Error {
        self.1.unwrap()
    }

    fn into_compile_error(self) -> TokenStream {
        TokenStream::from(self.1.unwrap().to_compile_error())
    }

    fn expand_llvm_versions_attr(&mut self, attr: &Attribute) -> Attribute {
        // Make no modifications if we've generated an error
        if self.has_error() {
            return attr.clone();
        }

        // If this isn't an llvm_versions attribute, skip it
        if !attr.path.is_ident("llvm_versions") {
            return attr.clone();
        }

        // Expand from llvm_versions to raw cfg attribute
        match syn::parse2::<ParenthesizedFeatureSet>(attr.tokens.clone()) {
            Ok(ParenthesizedFeatureSet(features)) => {
                parse_quote! {
                    #[cfg(any(#(feature = #features),*))]
                }
            }
            Err(err) => {
                // We've hit an error, but we can't break out yet,
                // so we set the error in the FeatureSet state and
                // avoid any further modifications until we can produce
                // the error
                self.set_error(err);
                attr.clone()
            }
        }
    }
}
impl Fold for FeatureSet {
    fn fold_variant(&mut self, mut variant: Variant) -> Variant {
        if self.has_error() {
            return variant;
        }

        let attrs = variant.attrs
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

        let attrs = field.attrs
            .iter()
            .map(|attr| self.expand_llvm_versions_attr(attr))
            .collect::<Vec<_>>();
        field.attrs = attrs;
        field
    }
}

/// This macro can be used to specify version constraints for an enum/struct/union or
/// other item which can be decorated with an attribute.
///
/// To use with enum variants or struct fields, you need to decorate the parent item with
/// the `#[llvm_versioned_item]` attribute, as this is the hook we need to modify the AST
/// of those items
///
/// # Examples
///
/// ```ignore
/// // Inclusive range from 3.6 up to and including 3.9
/// #[llvm_versions(3.6..=3.9)]
///
/// // Exclusive range from 3.6 up to but not including 4.0
/// #[llvm_versions(3.6..4.0)]
///
/// // Inclusive range from 3.6 up to and including the latest release
/// #[llvm_versions(3.6..=latest)]
/// ```
#[proc_macro_attribute]
pub fn llvm_versions(attribute_args: TokenStream, attributee: TokenStream) -> TokenStream {
    let mut features = parse_macro_input!(attribute_args as FeatureSet);

    let attributee = parse_macro_input!(attributee as Item);
    let folded = features.fold_item(attributee);

    if features.has_error() {
        return features.into_compile_error();
    }

    // Add nightly only doc cfgs to improve documentation on nightly builds
    // such as our own hosted docs.
    let doc = if cfg!(feature = "nightly") {
        let features2 = features.clone();
        quote! {
            #[doc(cfg(any(#(feature = #features2),*)))]
        }
    } else {
        quote! {}
    };

    let q = quote! {
        #[cfg(any(#(feature = #features),*))]
        #doc
        #folded
    };

    q.into()
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
pub fn llvm_versioned_item(_attribute_args: TokenStream, attributee: TokenStream) -> TokenStream {
    let attributee = parse_macro_input!(attributee as Item);

    let mut features = FeatureSet::default();
    let folded = features.fold_item(attributee);

    if features.has_error() {
        return features.into_compile_error();
    }

    quote!(#folded).into()
}

/// Used to track an enum variant and its corresponding mappings (LLVM <-> Rust),
/// as well as attributes
struct EnumVariant {
    llvm_variant: Ident,
    rust_variant: Ident,
    attrs: Vec<Attribute>,
}
impl EnumVariant {
    fn new(variant: &Variant) -> Self {
        let rust_variant = variant.ident.clone();
        let llvm_variant = Ident::new(&format!("LLVM{}", rust_variant.to_string()), variant.span());
        let mut attrs = variant.attrs.clone();
        attrs.retain(|attr| !attr.path.is_ident("llvm_variant"));
        Self {
            llvm_variant,
            rust_variant,
            attrs,
        }
    }

    fn with_name(variant: &Variant, mut llvm_variant: Ident) -> Self {
        let rust_variant = variant.ident.clone();
        llvm_variant.set_span(rust_variant.span());
        let mut attrs = variant.attrs.clone();
        attrs.retain(|attr| !attr.path.is_ident("llvm_variant"));
        Self {
            llvm_variant,
            rust_variant,
            attrs,
        }
    }
}

/// Used when constructing the variants of an enum declaration.
#[derive(Default)]
struct EnumVariants {
    variants: Vec<EnumVariant>,
    error: Option<Error>,
}
impl EnumVariants {
    #[inline]
    fn len(&self) -> usize {
        self.variants.len()
    }

    #[inline]
    fn iter(&self) -> core::slice::Iter<'_, EnumVariant> {
        self.variants.iter()
    }

    #[inline]
    fn has_error(&self) -> bool {
        self.error.is_some()
    }

    #[inline]
    fn set_error(&mut self, err: &str, span: Span) {
        self.error = Some(Error::new(span, err));
    }

    fn into_error(self) -> Error {
        self.error.unwrap()
    }
}
impl Fold for EnumVariants {
    fn fold_variant(&mut self, mut variant: Variant) -> Variant {
        use syn::{Meta, NestedMeta};

        if self.has_error() {
            return variant;
        }

        // Check for llvm_variant
        if let Some(attr) = variant.attrs.iter().find(|attr| attr.path.is_ident("llvm_variant")) {
            // Extract attribute meta
            if let Ok(Meta::List(meta)) = attr.parse_meta() {
                // We should only have one element
                if meta.nested.len() == 1 {
                    let variant_meta = meta.nested.first().unwrap();
                    // The element should be an identifier
                    if let NestedMeta::Meta(Meta::Path(name)) = variant_meta {
                        self.variants.push(EnumVariant::with_name(&variant, name.get_ident().unwrap().clone()));
                        // Strip the llvm_variant attribute from the final AST
                        variant.attrs.retain(|attr| !attr.path.is_ident("llvm_variant"));
                        return variant;
                    }
                }
            }

            // If at any point we fall through to here, it is the same basic issue, invalid format
            self.set_error("expected #[llvm_variant(VARIANT_NAME)]", attr.span());
            return variant;
        }

        self.variants.push(EnumVariant::new(&variant));
        variant
    }
}

/// Used to parse an enum declaration decorated with `#[llvm_enum(..)]`
struct LLVMEnumType {
    name: Ident,
    decl: syn::ItemEnum,
    variants: EnumVariants,
}
impl Parse for LLVMEnumType {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse enum declaration
        let decl = input.parse::<syn::ItemEnum>()?;
        let name = decl.ident.clone();
        // Fold over variants and expand llvm_versions
        let mut features = FeatureSet::default();
        let decl = features.fold_item_enum(decl);
        if features.has_error() {
            return Err(features.into_error());
        }

        let mut variants = EnumVariants::default();
        let decl = variants.fold_item_enum(decl);
        if variants.has_error() {
            return Err(variants.into_error());
        }

        Ok(Self {
            name,
            decl,
            variants,
        })
    }
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
///     #[llvm_versions(3.8..=latest)]
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
pub fn llvm_enum(attribute_args: TokenStream, attributee: TokenStream) -> TokenStream {
    use syn::{Path, PatPath, Arm};

    // Expect something like #[llvm_enum(LLVMOpcode)]
    let llvm_ty = parse_macro_input!(attribute_args as Path);
    let llvm_enum_type = parse_macro_input!(attributee as LLVMEnumType);

    // Construct match arms for LLVM -> Rust enum conversion
    let mut from_arms = Vec::with_capacity(llvm_enum_type.variants.len());
    for variant in llvm_enum_type.variants.iter() {
        let src_variant = variant.llvm_variant.clone();
        // Filter out doc comments or else rustc will warn about docs on match arms in newer versions.
        let src_attrs: Vec<_> = variant
            .attrs
            .iter()
            .filter(|&attr| !attr.parse_meta().unwrap().path().is_ident("doc"))
            .collect();
        let src_ty = llvm_ty.clone();
        let dst_variant = variant.rust_variant.clone();
        let dst_ty = llvm_enum_type.name.clone();

        let pat = PatPath {
            attrs: Vec::new(),
            qself: None,
            path: parse_quote!(#src_ty::#src_variant),
        };

        let arm: Arm = parse_quote! {
            #(#src_attrs)*
            #pat => { #dst_ty::#dst_variant }
        };
        from_arms.push(arm);
    }

    // Construct match arms for Rust -> LLVM enum conversion
    let mut to_arms = Vec::with_capacity(llvm_enum_type.variants.len());
    for variant in llvm_enum_type.variants.iter() {
        let src_variant = variant.rust_variant.clone();
        // Filter out doc comments or else rustc will warn about docs on match arms in newer versions.
        let src_attrs: Vec<_> = variant
            .attrs
            .iter()
            .filter(|&attr| !attr.parse_meta().unwrap().path().is_ident("doc"))
            .collect();
        let src_ty = llvm_enum_type.name.clone();
        let dst_variant = variant.llvm_variant.clone();
        let dst_ty = llvm_ty.clone();

        let pat = PatPath {
            attrs: Vec::new(),
            qself: None,
            path: parse_quote!(#src_ty::#src_variant),
        };

        let arm: Arm = parse_quote! {
            #(#src_attrs)*
            #pat => { #dst_ty::#dst_variant }
        };
        to_arms.push(arm);
    }

    let enum_ty = llvm_enum_type.name.clone();
    let enum_decl = llvm_enum_type.decl;

    let q = quote! {
        #enum_decl

        impl #enum_ty {
            fn new(src: #llvm_ty) -> Self {
                match src {
                    #(#from_arms)*
                }
            }
        }
        impl From<#llvm_ty> for #enum_ty {
            fn from(src: #llvm_ty) -> Self {
                Self::new(src)
            }
        }
        impl Into<#llvm_ty> for #enum_ty {
            fn into(self) -> #llvm_ty {
                match self {
                    #(#to_arms),*
                }
            }
        }
    };
    q.into()
}
