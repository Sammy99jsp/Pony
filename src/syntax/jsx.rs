//!
//! Syntax according to the [JSX spec](https://facebook.github.io/jsx)
//!

use std::fmt::{Debug, write};

use derive_syn_parse::Parse;
use quote::ToTokens;
use syn::{
    parse::ParseStream,
    Token, spanned::Spanned,
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
    opening: FragmentOpening,
    #[call(parse_fragment_children)]
    children: Children,
    closing: FragmentClosing,
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
    lt: Token![<],
    gt: Token![>],
}

#[derive(Parse)]
pub struct FragmentClosing {
    lt: Token![<],
    slash: Token![/],
    gt: Token![>],
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
    opening: OpeningElement,
    children: Children,
    closing: ClosingElement,
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

fn parse_attrs<'a>(input: ParseStream<'a>) -> syn::Result<Vec<Attribute>> {
    let mut t = vec![];
    while !(input.peek(Token![>]) || input.peek(Token![/])) {
        t.push(input.parse()?);
    }
    Ok(t)
}

#[derive(Parse)]
pub struct SelfClosingElement {
    lt: Token![<],
    name: ElementName,
    #[call(parse_attrs)]
    attributes: Attributes,
    slash: Token![/],
    gt: Token![>],
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
    lt: Token![<],
    name: ElementName,
    #[call(parse_attrs)]
    attributes: Attributes,
    gt: Token![>],
}

#[derive(Parse)]
pub struct ClosingElement {
    lt: Token![<],
    slash: Token![/],
    name: ElementName,
    gt: Token![>],
}

///
/// Covers all possibilities we want, including single identifiers
/// and multiple paths:
/// * `<Element />` (path length 1)
/// * `<my::module::path::Element />` (path length 4)
///
#[derive(Parse)]
pub struct ElementName(syn::Path);

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
    brace: syn::token::Brace,

    #[inside(brace)]
    rest: Token![..],

    #[inside(brace)]
    expr: syn::Expr,
}

impl Debug for SpreadAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpreadAttribute({})", self.expr.to_token_stream().to_string())
    }
}

#[derive(Parse)]
pub struct NamedAttribute {
    key: Identifier,
    #[peek(syn::token::Eq)]
    initializer: Option<AttributeInitializer>,
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
    equals: Token![=],
    value: AttributeValue,
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
            Self::Expr(expr) => write!(f, "{}", expr.expr.to_token_stream().to_string()),
        }
    }
}

#[derive(Parse)]
pub struct ExprAttributeValue {
    #[brace]
    brace: syn::token::Brace,

    #[inside(brace)]
    expr: syn::Expr,
}

pub type Children = Vec<Child>;

pub enum Child {
    Text(Text),
    Element(Element),
    Fragment(Fragment),
    Mustache(Mustache),
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
        }
    }
}

///
/// Anything (including spaces), except `{`,`<`,`>`,`}`
///
pub struct Text(proc_macro2::TokenStream);

impl syn::parse::Parse for Text {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use quote::TokenStreamExt;

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

#[cfg(test)]
mod tests {
    use crate::syntax::jsx::{ClosingElement, ClosedElement, Root};

    use super::{Element, SelfClosingElement};

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