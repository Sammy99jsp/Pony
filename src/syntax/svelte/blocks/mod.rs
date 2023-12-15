//!
//! ## Svelte blocks.
//!
//! ### `{#if expr}`
//! ### `{#match expr}`
//! ### `{#for item in iter}`
//! ### `{#key expr}`
//! ### `{#async expr}`
//!
//!

use std::fmt::Debug;
use syn::parse::ParseStream;

use super::{inside_braces, Peek};

pub mod if_block;
use if_block::IfBlock;

pub enum Block {
    If(IfBlock),
}

impl syn::parse::Parse for Block {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if IfBlock::peek(input) {
            return Ok(Self::If(input.parse()?));
        }

        unimplemented!("Ahhh")
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::If(if_block) => if_block.fmt(f),
        }
    }
}

impl Peek for Block {
    fn peek(input: ParseStream) -> bool {
        IfBlock::peek(input)
        // || ;
    }
}

pub(crate) fn parse_until<P: syn::parse::Parse>(
    input: ParseStream,
    pred: impl Fn(ParseStream) -> bool,
) -> syn::Result<Vec<P>> {
    let mut parsed = vec![];

    while !pred(input) {
        parsed.push(input.parse()?);
    }

    Ok(parsed)
}

pub(crate) fn parse_divided_until<Chi: syn::parse::Parse, Div: syn::parse::Parse + Peek>(
    input: ParseStream,
    pred: impl Fn(ParseStream) -> bool,
) -> syn::Result<Vec<(Div, Vec<Chi>)>> {
    let mut parsed = vec![];

    while !pred(input) {
        parsed.push((
            input.parse()?,
            parse_until(input, |i| Div::peek(i) || pred(i))?,
        ));
    }

    Ok(parsed)
}
