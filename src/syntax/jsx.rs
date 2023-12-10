//!
//! Syntax according to the [JSX spec](https://facebook.github.io/jsx)
//!

use std::fmt::Debug;

use derive_syn_parse::Parse;
use quote::{ToTokens, TokenStreamExt};
use syn::{
    parse::ParseStream,
    Token,
};

pub enum Root {
    Element(Element),
    Fragment(Fragment)
}

impl Debug for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Element(element) => write!(f, "{element:?}"),
            Self::Fragment(fragment) => write!(f, "{fragment:?}"),
        }
    }
}

impl syn::parse::Parse for Root {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![<]) {
            if input.peek2(Token![>]) {
                return Ok(Self::Fragment(input.parse()?));
            }
            
            if input.peek2(syn::Ident) {
                return Ok(Self::Element(input.parse()?));
            }
        }

        Err(input.error("Expected either element or fragment here"))
    }
}

use super::mustache::Mustache;

fn parse_fragment_children(input: ParseStream) -> syn::Result<Vec<Child>> {
    let mut children = vec![];

    while !(input.peek(Token![<]) && input.peek2(Token![/]) && input.peek3(Token![>])) {
        children.push(input.parse()?);
    }

    Ok(children)
}

#[derive(Parse)]
pub struct Fragment {
    pub opening: FragmentOpening,
    #[call(parse_fragment_children)]
    pub children: Children,
    pub closing: FragmentClosing,
}

impl Debug for Fragment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fragment")
            .field("children", &self.children)
            .finish()
    }
}

#[derive(Parse)]
pub struct FragmentOpening {
    pub lt: Token![<],
    pub gt: Token![>],
}

#[derive(Parse)]
pub struct FragmentClosing {
    pub lt: Token![<],
    pub slash: Token![/],
    pub gt: Token![>],
}

pub enum Element {
    Closed(ClosedElement),
    SelfClosing(SelfClosingElement),
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed(closed) => write!(f, "{closed:?}"),
            Self::SelfClosing(self_closing) => write!(f, "{self_closing:?}"),
        }
    }
}

impl syn::parse::Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let f = input.fork();

        let _: Token![<] = f.parse()?;
        let _: syn::Ident = f.parse()?;
        let _: Vec<Attribute> = parse_attrs(&f)?;

        if f.peek(Token![/]) {
            return Ok(Self::SelfClosing(input.parse()?));
        }
        
        if f.peek(Token![>]) {
            return Ok(Self::Closed(input.parse()?));
        }

        Err(input.error("Expected either opening or self-closing tag here"))
    }
}

pub struct ClosedElement {
    pub opening: OpeningElement,
    pub children: Children,
    pub closing: ClosingElement,
}

impl Debug for ClosedElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element")
            .field("name", &self.opening.name.0.to_token_stream().to_string())
            .field("attributes", &self.opening.attributes)
            .field("children", &self.children)
            .finish()
    }
}

impl syn::parse::Parse for ClosedElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let opening: OpeningElement = input.parse()?;
        let mut children = vec![];
        loop {
            if input.is_empty() {
                return Err(input.error(format!("Did not find appropriate closing tag `<{}/>`", opening.name.0.to_token_stream())));
            }

            if input.peek(Token![<]) && input.peek2(Token![/]) && input.peek3(syn::Ident) {
                let f = input.fork();
                let _: Token![<] = f.parse().unwrap();
                let _: Token![/] = f.parse().unwrap();
                let i: syn::Path = f.parse().unwrap();

                if i.segments
                    .iter()
                    .zip(opening.name.0.segments.iter())
                    .all(|(a, b)| {
                        a.ident == b.ident
                    })
                {
                    return Ok(Self {
                        opening,
                        children,
                        closing: input.parse()?,
                    });
                }
            }

            children.push(input.parse()?);
        }
    }
}

fn parse_attrs(input: ParseStream) -> syn::Result<Vec<Attribute>> {
    let mut t = vec![];
    while !(input.peek(Token![>]) || input.peek(Token![/])) {
        t.push(input.parse()?);
    }
    Ok(t)
}

#[derive(Parse)]
pub struct SelfClosingElement {
    pub lt: Token![<],
    pub name: ElementName,
    #[call(parse_attrs)]
    pub attributes: Attributes,
    pub slash: Token![/],
    pub gt: Token![>],
}

impl Debug for SelfClosingElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element")
            .field("name", &self.name.0.to_token_stream().to_string())
            .field("attributes", &self.attributes)
            .finish()
    }
}

#[derive(Parse)]
pub struct OpeningElement {
    pub lt: Token![<],
    pub name: ElementName,
    #[call(parse_attrs)]
    pub attributes: Attributes,
    pub gt: Token![>],
}

#[derive(Parse)]
pub struct ClosingElement {
    pub lt: Token![<],
    pub slash: Token![/],
    pub name: ElementName,
    pub gt: Token![>],
}

///
/// Covers all possibilities we want, including single identifiers
/// and multiple paths:
/// * `<Element />` (path length 1)
/// * `<my::module::path::Element />` (path length 4)
///
#[derive(Parse)]
pub struct ElementName(pub syn::Path);

pub type Attributes = Vec<Attribute>;
pub type Identifier = syn::Ident;

#[derive(Parse)]
pub enum Attribute {
    #[peek(syn::token::Brace, name = "spread attribute")]
    Spread(SpreadAttribute),

    #[peek(syn::Ident, name = "named attribute")]
    Named(NamedAttribute),
}

impl Debug for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spread(spread) => write!(f, "{spread:?}"),
            Self::Named(named) => write!(f, "{named:?}"),
        }
    }
}

#[derive(Parse)]
pub struct SpreadAttribute {
    #[brace]
    pub brace: syn::token::Brace,

    #[inside(brace)]
    pub rest: Token![..],

    #[inside(brace)]
    pub expr: syn::Expr,
}

impl Debug for SpreadAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpreadAttribute({})", self.expr.to_token_stream())
    }
}

#[derive(Parse)]
pub struct NamedAttribute {
    pub key: Identifier,
    #[peek(syn::token::Eq)]
    pub initializer: Option<AttributeInitializer>,
}

impl Debug for NamedAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NamedAttribute")
            .field(self.key.to_string().as_str(), &self.initializer)
            .finish()
    }
}

#[derive(Parse)]
pub struct AttributeInitializer {
    pub equals: Token![=],
    pub value: AttributeValue,
}

impl Debug for AttributeInitializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

#[derive(Parse)]
pub enum AttributeValue {
    #[peek(syn::LitStr, name = "string literal")]
    LitStr(syn::LitStr),

    #[peek(syn::token::Brace, name = "attribute value")]
    Expr(ExprAttributeValue),
}

impl Debug for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LitStr(litstr) => write!(f, "{:?}", litstr.value()),
            Self::Expr(expr) => write!(f, "{}", expr.expr.to_token_stream()),
        }
    }
}

#[derive(Parse)]
pub struct ExprAttributeValue {
    #[brace]
    pub brace: syn::token::Brace,

    #[inside(brace)]
    pub expr: syn::Expr,
}

pub type Children = Vec<Child>;

pub enum Child {
    Text(Text),
    Element(Element),
    Fragment(Fragment),
    Mustache(Mustache),
    Comment(Comment),
}

impl syn::parse::Parse for Child {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![<]) && input.peek2(Token![>]) {
            return Ok(Self::Fragment(input.parse()?));
        }

        if input.peek(Token![<]) && input.peek2(syn::Ident) {
            return Ok(Self::Element(input.parse()?));
        }

        if input.peek(syn::token::Brace) {
            return Ok(Self::Mustache(input.parse()?));
        }

        if Comment::peek(input) {
            return Ok(Self::Comment(input.parse()?));
        }

        Ok(Self::Text(input.parse()?))
    }
}

impl Debug for Child {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(text) => write!(f, "{text:?}"),
            Self::Element(element) => write!(f, "{element:?}"),
            Self::Fragment(fragment) => write!(f, "{fragment:?}"),
            Self::Mustache(mustache) => write!(f, "{mustache:?}"),
            Self::Comment(comment) => write!(f, "{comment:?}")
        }
    }
}

///
/// Anything (including spaces), except `{`,`<`,`>`,`}`
///
pub struct Text(proc_macro2::TokenStream);

impl syn::parse::Parse for Text {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tkns = proc_macro2::TokenStream::new();

        // While next tokens aren't `{`, `<`, `>`, `}`.
        while !(input.peek(Token![<]) || input.peek(Token![>]) || input.peek(syn::token::Brace)) {
            tkns.append(input.parse::<proc_macro2::TokenTree>()?)
        }

        Ok(Self(tkns))
    }
}

impl Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Text({})", self.0)
        // f.debug_tuple("Text").field(&self.0.to_string()).finish()
    }
}


///
/// HTML/XML-style Comment: `<!-- ANYTHING! -->`
///
pub struct Comment {
    pub open: OpenComment,
    pub contents: proc_macro2::TokenStream,
    pub closing: CloseComment,
}

impl Comment {
    fn peek(input: ParseStream) -> bool {
        // We need to look at 4 characters,
        //  which is more than any `input.peek` function gives us.

        // Instead, speculatively parse:
        let f = input.fork();

        let r: syn::Result<bool> = try {
            let _: Token![<] = f.parse()?;
            f.peek(Token![!]) && f.peek2(Token![-]) && f.peek3(Token![-])
        };

        r.unwrap_or_default()
    }
}

impl syn::parse::Parse for Comment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let open = input.parse()?;
        let mut contents = proc_macro2::TokenStream::new();

        while !(input.peek(Token![-]) && input.peek2(Token![-]) && input.peek3(Token![>])) {
            contents.append::<proc_macro2::TokenTree>(input.parse()?);
        }

        Ok(Self {
            open,
            contents,
            closing: input.parse()?,
        })
    }
}

impl Debug for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Comment({})", self.contents.to_token_stream())
    }
}

#[derive(Parse)]
pub struct OpenComment {
    pub lt: Token![<],
    pub bang: Token![!],
    pub minus1: Token![-],
    pub minus2: Token![-],
}

#[derive(Parse)]
pub struct CloseComment {
    pub m1: Token![-],
    pub m2: Token![-],
    pub gt: Token![>],
}

#[cfg(test)]
mod tests {
    use crate::syntax::jsx::ClosedElement;

    use super::SelfClosingElement;

    #[test]
    fn parse_element() {
        let _: SelfClosingElement = syn::parse_str(r#"
            <Button cool willToLive={8*8*8==4} {..stuff} />
        "#).expect("Valid parse");
        
        let el: ClosedElement = syn::parse_str(r#"
            <Button onClick={|| count += 1}>
                Clicked {count:'0'>3?} times!
            </Button>
        "#).expect("Valid parse");

        println!("{el:#?}")
    }
}