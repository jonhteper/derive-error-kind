//! # derive-error-kind
//!
//! A Rust procedural macro for implementing the ErrorKind pattern that simplifies error classification and handling in complex applications.
//!
//! ## Motivation
//!
//! The ErrorKind pattern is a common technique in Rust for separating:
//! - The **kind** of an error (represented by a simple enum)
//! - The **details** of the error (contained in the error structure)
//!
//! This allows developers to handle errors more granularly without losing context.
//!
//! Rust's standard library uses this pattern in `std::io::ErrorKind`, and many other libraries have adopted it due to its flexibility. However, manually implementing this pattern can be repetitive and error-prone, especially in applications with multiple nested error types.
//!
//! This crate solves this problem by providing a derive macro that automates the implementation of the ErrorKind pattern.
//!
//! ## Overview
//!
//! The `ErrorKind` macro allows you to associate error types with a specific kind from an enum. This creates a clean and consistent way to categorize errors in your application, enabling more precise error handling.
//!
//! Key features:
//! - Automatically implements a `.kind()` method that returns a categorized error type
//! - Supports nested error types via the `transparent` attribute
//! - Works with unit variants, named fields, and tuple variants
//! - Enables transparent error propagation through error hierarchies
//!
//! ## Basic Usage
//!
//! First, define an enum for your error kinds:
//!
//! ```rust
//! #[derive(Copy, Clone, Debug, Eq, PartialEq)]
//! pub enum ErrorKind {
//!     NotFound,
//!     InvalidInput,
//!     InternalError,
//! }
//! ```
//!
//! Then, use the `ErrorKind` derive macro on your error enums:
//!
//! ```rust
//! use derive_error_kind::ErrorKind;
//!
//! #[derive(Debug, ErrorKind)]
//! #[error_kind(ErrorKind)]
//! pub enum MyError {
//!     #[error_kind(ErrorKind, NotFound)]
//!     ResourceNotFound,
//!
//!     #[error_kind(ErrorKind, InvalidInput)]
//!     BadRequest { details: String },
//!
//!     #[error_kind(ErrorKind, InternalError)]
//!     ServerError(String),
//! }
//!
//! // Now you can use the .kind() method
//! let error = MyError::ResourceNotFound;
//! assert_eq!(error.kind(), ErrorKind::NotFound);
//! ```
//!
//! ## Attribute Reference
//!
//! - `#[error_kind(KindEnum)]`: Top-level attribute that specifies which enum to use for error kinds
//! - `#[error_kind(KindEnum, Variant)]`: Variant-level attribute that specifies which variant of the kind enum to return
//! - `#[error_kind(transparent)]`: Variant-level attribute for nested errors, indicating that the inner error's kind should be used
//!
//! ## Requirements
//!
//! - The macro can only be applied to enums
//! - Each variant must have an `error_kind` attribute
//! - The kind enum must be in scope and accessible

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, DeriveInput, Meta, MetaList, NestedMeta, Path,
};

/// Create a kind method for struct
/// # Examples
/// ```
/// use derive_error_kind::ErrorKind;
///#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// enum ErrorType {
///     A,
///     B,
///     C,
/// }
///
/// #[derive(ErrorKind)]
/// #[error_kind(ErrorType)]
/// enum CacheError {
///     #[error_kind(ErrorType, A)]
///     Poisoned,
///
///     #[error_kind(ErrorType, B)]
///     Missing,
/// }
///
/// #[derive(ErrorKind)]
/// #[error_kind(ErrorType)]
/// enum ServiceError {
///     #[error_kind(transparent)]
///     Cache(CacheError),
///
///     #[error_kind(ErrorType, C)]
///     Db,
/// }
///
/// assert_eq!(ServiceError::Cache(CacheError::Missing).kind(), ErrorType::B);
/// assert_eq!(ServiceError::Db.kind(), ErrorType::C);
/// ```
#[proc_macro_derive(ErrorKind, attributes(error_kind))]
pub fn error_kind(input: TokenStream) -> TokenStream {
    error_kind_macro(input)
}

fn error_kind_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let kind_ty = get_kind_ty(&input);

    let name = input.ident;
    let variants = if let syn::Data::Enum(data) = input.data {
        data.variants
    } else {
        panic!("ImplKind just can be used in enums");
    };

    let mut kind_variants = Vec::new();

    for variant in variants.clone() {
        let ident = variant.ident;
        if let Some(attr) = variant
            .attrs
            .into_iter()
            .find(|attr| attr.path.is_ident("error_kind"))
        {
            if let Ok(syn::Meta::List(meta)) = attr.parse_meta() {
                if meta.nested.len() == 2 {
                    if let (
                        syn::NestedMeta::Meta(syn::Meta::Path(enum_ty)),
                        syn::NestedMeta::Meta(syn::Meta::Path(variant)),
                    ) = (&meta.nested[0], &meta.nested[1])
                    {
                        kind_variants.push((ident, enum_ty.clone(), Some(variant.clone())));
                    } else {
                        panic!("Invalid value for error_kind");
                    }
                } else if meta.nested.len() == 1 {
                    for sub_meta in meta.nested {
                        if let NestedMeta::Meta(Meta::Path(path)) = sub_meta {
                            if path.is_ident("transparent") {
                                kind_variants.push((ident.clone(), kind_ty.clone(), None));
                            }
                        } else {
                            panic!("Invalid value for #[error_kind]");
                        }
                    }
                } else {
                    panic!("error_kind must have one two arguments");
                }
            } else {
                panic!("Error parsing meta");
            }
        } else {
            panic!("Enum variants must have the attribute `error_kind`");
        }
    }

    let kind_enum = kind_variants
        .first()
        .expect("No variants in Enum")
        .1
        .clone();
    let match_arms = kind_variants.into_iter().map(|(ident, enum_ty, variant)| {
        let fields = &variants.iter().find(|v| v.ident == ident).unwrap().fields;
        match fields {
            syn::Fields::Unit => {
                quote! {
                    Self::#ident => #enum_ty::#variant,
                }
            }
            syn::Fields::Named(_) => {
                quote! {
                    Self::#ident{..} => #enum_ty::#variant,
                }
            }
            syn::Fields::Unnamed(_) => match variant {
                Some(v) => quote! {
                    Self::#ident(..) => #enum_ty::#v,
                },
                None => quote! {
                    Self::#ident(inner) => inner.kind(),
                },
            },
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn kind(&self) -> #kind_enum {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_kind_ty(input: &DeriveInput) -> Path {
    let metas = find_attribute(input, "error_kind")
        .expect("#[derive(ErrorKind)] requires error_kind attribute");
    if let Some(&NestedMeta::Meta(Meta::Path(ref path))) = metas.iter().next() {
        path.to_owned()
    } else {
        panic!("#[error_kind(KIND_IDENT)] attribute requires and identifier");
    }
}

/// Get an attribute from the input.
/// 
/// Adapted from https://crates.io/crates/enum-kinds
fn find_attribute(
    definition: &DeriveInput,
    name: &str,
) -> Option<Punctuated<NestedMeta, syn::token::Comma>> {
    for attr in definition.attrs.iter() {
        match attr.parse_meta() {
            Ok(Meta::List(MetaList {
                ref path,
                ref nested,
                ..
            })) if path.is_ident(name) => return Some(nested.clone()),
            _ => continue,
        }
    }
    None
}
