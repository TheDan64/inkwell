use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::fold::Fold;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::parse_quote;
use syn::spanned::Spanned;
use syn::{Arm, PatPath, Path};
use syn::{Attribute, Ident, Variant};

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
        let llvm_variant = Ident::new(&format!("LLVM{}", rust_variant), variant.span());
        let mut attrs = variant.attrs.clone();
        attrs.retain(|attr| !attr.path().is_ident("llvm_variant"));
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
        attrs.retain(|attr| !attr.path().is_ident("llvm_variant"));
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
        use syn::Meta;

        if self.has_error() {
            return variant;
        }

        // Check for llvm_variant
        if let Some(attr) = variant.attrs.iter().find(|attr| attr.path().is_ident("llvm_variant")) {
            // Extract attribute meta
            if let Meta::List(meta) = &attr.meta {
                // We should only have one element

                if let Ok(Meta::Path(name)) = meta.parse_args() {
                    self.variants
                        .push(EnumVariant::with_name(&variant, name.get_ident().unwrap().clone()));
                    // Strip the llvm_variant attribute from the final AST
                    variant.attrs.retain(|attr| !attr.path().is_ident("llvm_variant"));
                    return variant;
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
pub struct LLVMEnumType {
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
        let decl = crate::cfg::VersionFolder::fold_any(Fold::fold_item_enum, decl)?;

        let mut variants = EnumVariants::default();
        let decl = variants.fold_item_enum(decl);
        if variants.has_error() {
            return Err(variants.into_error());
        }

        Ok(Self { name, decl, variants })
    }
}

pub fn llvm_enum(llvm_ty: Path, llvm_enum_type: LLVMEnumType) -> TokenStream {
    // Construct match arms for LLVM -> Rust enum conversion
    let mut from_arms = Vec::with_capacity(llvm_enum_type.variants.len());
    for variant in llvm_enum_type.variants.iter() {
        let src_variant = variant.llvm_variant.clone();
        // Filter out doc comments or else rustc will warn about docs on match arms in newer versions.
        let src_attrs: Vec<_> = variant
            .attrs
            .iter()
            .filter(|&attr| !attr.meta.path().is_ident("doc"))
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
            .filter(|&attr| !attr.meta.path().is_ident("doc"))
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

    quote! {
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
    }
}
