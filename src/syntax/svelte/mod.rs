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

pub mod block;

use syn::parse::ParseStream;
pub use block::Block;

pub trait Peek {
    fn peek(input: ParseStream) -> bool;
}



pub mod kw {
    use syn::custom_keyword;

    custom_keyword!(case);
}



