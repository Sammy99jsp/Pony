//!
//! ## Svelte Syntax
//!
//! ### Blocks
//! Blocks have the `{#keyword ...}{/keyword}` syntax.
//!
//! - `{#if ...}`
//! - `{#match ...}`
//! - `{#for item in iter}`
//! - `{#async fut}`
//! - `{#key expr}`
//!
//! ### Dividers
//! These are the `{:keyword}` found inside their parent blocks.
//!
//! - `{:else}`
//! - `{:else if ...}`
//! - `{:case pat}`
//! - `{:await variable}`
//!
//! ### Special Tags (STag)
//! These have the syntax `{@keyword ...}`.
//!
//! - `{@let variable = expr}`
//! - `{@macro!(...)}`
//! - `{@debug ...}`
//!

pub mod blocks;

pub use blocks::Block;
use syn::parse::{ParseBuffer, ParseStream};

pub(crate) trait Peek {
    fn peek(input: ParseStream) -> bool;
}

pub(crate) fn inside_braces(input: ParseStream) -> syn::Result<ParseBuffer> {
    (|| {
        let inner;
        let _ = syn::braced!(inner in input.fork());
        Ok(inner)
    })()
}

pub mod kw {
    use syn::custom_keyword;

    custom_keyword!(case);
}
