//!
//! ## `{#match}`
//!
//! New Pony syntax.
//!
//! ### Dividers
//! #### `{:case <expr>}`
//!
//! ### Examples
//! ```pony
//! {#match qty}
//!     <!-- Only Comments can go before cases-->
//!     {:case 0}
//!         <T>Zero</T>
//!     {:case 1}
//!         <T>Singular</T>
//!     {:case 2}
//!         <T>Couple</T>
//!     {:case 3..}
//!         <T>Many</T>
//! {/match}
//! ```
//!
//! ```pony
//! {#match food}
//!     {:case Food::Vegetable(Vegetable::Eggplant)}
//!         🍆
//!     {:case Food::Fruit(Fruit::Apple(Apple { color: Color::Red, .. }))}
//!         🍎
//!     {:case Food::Fruit(Fruit::Apple(Apple { color: Color::Green, .. }))}
//!         🍏
//!     {:case Food::Fruit(Fruit::Apple(Apple { color, .. }))}
//!         A {color} Apple.
//!     {:case _}
//!         <!-- Display nothing in this case, you can also leave this blank. -->
//! {/match}
//! ```

use std::fmt::Debug;

use derive_syn_parse::Parse;
use syn::{parse::ParseStream, Token};

use crate::syntax::svelte::kw;
use crate::syntax::{jsx::Comment, pretty_rust};

use crate::syntax::jsx::Child;

use super::{inside_braces, parse_divided_until, parse_until, Peek};

pub struct MatchBlock {
    pub opening: MatchOpening,
    pub children: Vec<Comment>,
    pub cases: Vec<(CaseDivider, Vec<Child>)>,
    pub closing: MatchClosing,
}

impl syn::parse::Parse for MatchBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            opening: input.parse()?,
            children: parse_until(input, |input| {
                MatchClosing::peek(input) || CaseDivider::peek(input)
            })?,
            cases: parse_divided_until(input, MatchClosing::peek)?,
            closing: input.parse()?,
        })
    }
}

impl Peek for MatchBlock {
    fn peek(input: ParseStream) -> bool {
        MatchOpening::peek(input)
    }
}

impl Debug for MatchBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MatchBlock")
            .field("opening", &self.opening)
            .field("children", &self.children)
            .field("cases", &self.cases)
            .field("closing", &self.closing)
            .finish()
    }
}

#[derive(Parse)]
pub struct MatchOpening {
    #[brace]
    pub brace: syn::token::Brace,

    #[inside(brace)]
    pub pound: Token![#],

    #[inside(brace)]
    pub match_token: Token![match],

    #[inside(brace)]
    pub expr: syn::Expr,
}

impl Debug for MatchOpening {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MatchOpening")
            .field_with(|f| write!(f, "{}", pretty_rust(&self.expr)))
            .finish()
    }
}

impl Peek for MatchOpening {
    fn peek(input: ParseStream) -> bool {
        let r: syn::Result<bool> = try {
            let inner = inside_braces(input)?;
            inner.peek(Token![#]) && inner.peek2(Token![match])
        };

        r.unwrap_or_default()
    }
}

#[derive(Parse)]
pub struct MatchClosing {
    #[brace]
    pub brace: syn::token::Brace,

    #[inside(brace)]
    pub slash: Token![/],

    #[inside(brace)]
    pub match_token: Token![match],
}

impl Debug for MatchClosing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MatchClosing").finish()
    }
}

impl Peek for MatchClosing {
    fn peek(input: ParseStream) -> bool {
        let r: syn::Result<bool> = try {
            let inner = inside_braces(input)?;
            inner.peek(Token![/]) && inner.peek2(Token![match])
        };

        r.unwrap_or_default()
    }
}

#[derive(Parse)]
pub struct CaseDivider {
    #[brace]
    pub brace: syn::token::Brace,

    #[inside(brace)]
    pub colon: Token![:],

    #[inside(brace)]
    pub case_token: kw::case,

    #[inside(brace)]
    #[call(syn::Pat::parse_multi_with_leading_vert)]
    pub pattern: syn::Pat,
}

impl Peek for CaseDivider {
    fn peek(input: ParseStream) -> bool {
        let r: syn::Result<bool> = try {
            let inner = inside_braces(input)?;
            inner.peek(Token![:]) && inner.peek2(kw::case)
        };

        r.unwrap_or_default()
    }
}

impl Debug for CaseDivider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CaseDivider")
            .field_with(|f| write!(f, "{}", pretty_rust(&self.pattern)))
            .finish()
    }
}
