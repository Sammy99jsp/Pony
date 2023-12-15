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

use derive_syn_parse::Parse;
use syn::{
    parse::{ParseBuffer, ParseStream},
    Token,
};

use crate::syntax::pretty_rust;

use crate::syntax::jsx::Child;

use super::Peek;


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

fn inside_braces(input: ParseStream) -> syn::Result<ParseBuffer> {
    (|| {
        let inner;
        let _ = syn::braced!(inner in input.fork());
        Ok(inner)
    })()
}
fn parse_until<P: syn::parse::Parse>(
    input: ParseStream,
    pred: impl Fn(ParseStream) -> bool,
) -> syn::Result<Vec<P>> {
    let mut parsed = vec![];

    while !pred(input) {
        parsed.push(input.parse()?);
    }

    Ok(parsed)
}

fn parse_divided_until<Chi: syn::parse::Parse, Div: syn::parse::Parse + Peek>(
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

pub struct IfBlock {
    pub opening: IfOpening,
    pub children: Vec<Child>,
    pub dividers: Vec<(IfDivider, Vec<Child>)>,
    pub closing: IfClosing,
}

impl syn::parse::Parse for IfBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let opening = input.parse()?;

        Ok(Self {
            opening,
            children: parse_until(input, |i| IfDivider::peek(i) || IfClosing::peek(i))?,
            dividers: parse_divided_until(input, IfClosing::peek)?,
            closing: input.parse()?,
        })
    }
}

impl Debug for IfBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IfBlock")
            .field_with("condition", |f| {
                write!(f, "{}", pretty_rust(&self.opening.expr))
            })
            .field("children", &self.children)
            .field_with("dividers", |f| {
                self.dividers.iter().try_for_each(|(div, children)| {
                    div.fmt(f)?;
                    write!(f, " => ")?;
                    children.fmt(f)
                })
            })
            .finish()
    }
}

impl Peek for IfBlock {
    fn peek(input: ParseStream) -> bool {
        IfOpening::peek(input)
    }
}

#[derive(Parse)]
pub struct IfOpening {
    #[brace]
    pub brace: syn::token::Brace,

    #[inside(brace)]
    pub pound: Token![#],

    #[inside(brace)]
    pub if_token: Token![if],

    #[inside(brace)]
    pub expr: syn::Expr,
}

impl Peek for IfOpening {
    fn peek(input: ParseStream) -> bool {
        let r: syn::Result<bool> = try {
            let inner = inside_braces(input)?;
            inner.peek(Token![#]) && inner.peek2(Token![if])
        };

        r.unwrap_or_default()
    }
}

#[derive(Parse)]
pub struct IfClosing {
    #[brace]
    pub brace: syn::token::Brace,

    #[inside(brace)]
    pub slash: Token![/],

    #[inside(brace)]
    pub if_token: Token![if],
}

impl Peek for IfClosing {
    fn peek(input: ParseStream) -> bool {
        let r: syn::Result<bool> = try {
            let inner = inside_braces(input)?;
            inner.peek(Token![/]) && inner.peek2(Token![if])
        };

        r.unwrap_or_default()
    }
}

pub enum IfDivider {
    Else(ElseDivider),
    ElseIf(ElseIfDivider),
}

impl syn::parse::Parse for IfDivider {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if ElseIfDivider::peek(input) {
            return Ok(Self::ElseIf(input.parse()?));
        }

        if ElseDivider::peek(input) {
            return Ok(Self::Else(input.parse()?));
        }

        Err(input.error("Expected either `{:else if ...}`, or `{:else}` here"))
    }
}

impl Debug for IfDivider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Else(else_div) => else_div.fmt(f),
            Self::ElseIf(elseif_div) => elseif_div.fmt(f),
        }
    }
}

impl Peek for IfDivider {
    fn peek(input: ParseStream) -> bool {
        ElseDivider::peek(input) || ElseIfDivider::peek(input)
    }
}

#[derive(Parse)]
pub struct ElseDivider {
    #[brace]
    pub brace: syn::token::Brace,

    #[inside(brace)]
    pub colon: Token![:],

    #[inside(brace)]
    pub else_token: Token![else],
}

impl Debug for ElseDivider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElseDivider").finish()
    }
}

impl Peek for ElseDivider {
    fn peek(input: ParseStream) -> bool {
        let r: syn::Result<bool> = try {
            let inner = inside_braces(input)?;
            inner.peek(Token![:]) && inner.peek2(Token![else]) && !inner.peek3(Token![if])
        };

        r.unwrap_or_default()
    }
}

#[derive(Parse)]
pub struct ElseIfDivider {
    #[brace]
    pub brace: syn::token::Brace,

    #[inside(brace)]
    pub colon: Token![:],

    #[inside(brace)]
    pub else_token: Token![else],

    #[inside(brace)]
    pub if_token: Token![if],

    #[inside(brace)]
    pub expr: syn::Expr,
}

impl Debug for ElseIfDivider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ElseIfDivider({})", pretty_rust(&self.expr))
    }
}

impl Peek for ElseIfDivider {
    fn peek(input: ParseStream) -> bool {
        let r: syn::Result<bool> = try {
            let inner = inside_braces(input)?;
            inner.peek(Token![:]) && inner.peek2(Token![else]) && inner.peek3(Token![if])
        };

        r.unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::IfBlock;

    #[test]
    fn if_block_parsing() {
        let _blank: IfBlock =
            syn::parse_str(r#"{#if bowl.has_fruit()}{/if}"#).expect("Valid parse");

        let _else: IfBlock = syn::parse_str(
            r#"
        {#if bowl.has_fruit()}
            <FruitBowl fruits={bowl.fruits} />
        {:else}
            No fruit!
        {/if}"#,
        )
        .expect("Valid parse");

        let _else_if: IfBlock = syn::parse_str(
            r#"
        {#if bowl.has_fruit()}
            <FruitBowl fruits={bowl.fruits} />
        {:else if bowl.has_vegetable()}
            <!-- You don't store vegetables in a bowl... -->
            <VegetableBowl vegetables={bowl.vegetables} />
        {:else}
            Nothing in this bowl!
        {/if}"#,
        )
        .expect("Valid parse");

        let _nested: IfBlock = syn::parse_str(
            r#"{#if rainbow.burns(&sky.stars)}
                {#if ocean.covers(mountains.high().every())}
                    {#if Dolphin.flies() && Parrot.lives(Sea)}
                        {#if we.dream_of(Life) && Life == A(Dream)}
                            {#if Day == Night && Night == Day}
                                {#if trees.up_and_fly_away() && seas.up_and_fly_away()}
                                    {#if 8 * 8 * 8 == 4}
                                        {#if the_day().that_is(the_day().that_are(NoMore))}
                                            {#if the_earth().turning(Direction::RightToLeft)}
                                                {#if the_earth().just_for(the_sun()).denies(the_earth())}
                                                    {#if mother_nature().says("My work is through.")}
                                                        {#if the_day().that(|_| you() == me() && I() == you())}
                                                            Not loving you anymore.
                                                        {/if}
                                                    {/if}
                                                {/if}
                                            {/if}
                                        {/if}
                                    {/if}
                                {/if}
                            {/if}
                        {/if}
                    {/if}
                {/if}
            {/if}
            "#
        ).expect("Valid parse");

        println!("{_nested:#?}")
    }
}
