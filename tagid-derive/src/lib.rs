//! Provides the `Label` derive macro for implementing the `tagid::Label` trait.
//!
//! # Overview
//!
//! The `Label` derive macro automatically implements the `Label` trait for a given type,
//! defining an associated `Labeler` type and a default labeling function.
//!
//! # Example
//!
//! ```rust, ignore
//! use tagid::Label;
//!
//! #[derive(Label)]
//! struct MyType;
//! ```
//!
//! The derived implementation generates:
//!
//! ```rust, ignore
//! impl tagid::Label for MyType {
//!     type Labeler = tagid::MakeLabeling<Self>;
//!     fn labeler() -> Self::Labeler { tagid::MakeLabeling::default() }
//! }
//! ```
//!
//! # Dependencies
//!
//! This macro relies on the `syn`, `quote`, and `proc_macro` crates to parse the input,
//! generate Rust code, and interact with the procedural macro system.

use proc_macro::{self, TokenStream};
use quote::quote;
use syn::DeriveInput;

/// Derives the `Label` trait for a struct or enum.
///
/// This macro implements the `Label` trait for the annotated type,
/// setting its `Labeler` type to `MakeLabeling<Self>` and providing
/// a default implementation for `labeler()`.
///
/// # Example
///
/// ```rust, ignore
/// use tagid::Label;
///
/// #[derive(Label)]
/// struct Example;
///
/// let labeler = Example::labeler();
/// ```
///
/// The generated implementation:
///
/// ```ignore
/// impl tagid::Label for Example {
///     type Labeler = tagid::MakeLabeling<Self>;
///     fn labeler() -> Self::Labeler { tagid::MakeLabeling::default() }
/// }
/// ```
#[proc_macro_derive(Label)]
pub fn label_derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = syn::parse_macro_input!(input);
    let output = quote! {
        impl ::tagid::Label for #ident {
            type Labeler = ::tagid::MakeLabeling<Self>;
            fn labeler() -> Self::Labeler { ::tagid::MakeLabeling::default() }
        }
    };
    output.into()
}
