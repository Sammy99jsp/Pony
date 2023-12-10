use std::fmt::Debug;

use quote::ToTokens;
use syn::Token;

use super::formatting::Formatting;

pub struct Mustache {
    pub brace: syn::token::Brace,
    pub expr: syn::Expr,
    pub formatting: Option<FormattingGroup>,
}

impl syn::parse::Parse for Mustache {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inner;
        Ok(Self {
            brace: syn::braced!(inner in input),
            expr: inner.parse()?,
            formatting: if inner.peek(Token![:]) {
                Some(inner.parse()?)
            } else {
                None
            },
        })
    }
}

impl Debug for Mustache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Mustache");

        s.field("expr", &self.expr.to_token_stream().to_string());

        if let Some(ref formatting) = self.formatting {
            s.field("formatting", formatting);
        }

        s.finish()
    }
}

pub struct FormattingGroup {
    pub colon: Token![:],
    pub formatting: Formatting,
}

impl syn::parse::Parse for FormattingGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            colon: input.parse()?,
            formatting: input.parse()?,
        })
    }
}

impl Debug for FormattingGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.formatting, f)
    }
}

#[cfg(test)]
mod tests {
    use super::Mustache;

    #[test]
    fn mustache_parse() {
        let m: Mustache = syn::parse_str(r"{apple:'2'>?}").unwrap();
        println!("{m:?}")
    }
}