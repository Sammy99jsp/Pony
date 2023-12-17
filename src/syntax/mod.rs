use proc_macro2::{Delimiter, TokenTree};
use quote::ToTokens;

pub mod formatting;
pub mod jsx;
pub mod mustache;
pub mod svelte;

pub fn pretty_rust(tokens: &impl ToTokens) -> String {
    use std::fmt::Write;

    let tokens = tokens.to_token_stream();
    let mut out = String::new();

    for token in tokens {
        let _ = write!(
            out,
            "{}",
            match token {
                TokenTree::Group(g) => {
                    match g.delimiter() {
                        Delimiter::Parenthesis => format!("({})", pretty_rust(&g.stream())),
                        Delimiter::Brace => format!("{{{}}}", pretty_rust(&g.stream())),
                        Delimiter::Bracket => format!("[{}]", pretty_rust(&g.stream())),
                        Delimiter::None => pretty_rust(&g.stream()),
                    }
                }
                TokenTree::Ident(i) => format!(" {i}"),
                TokenTree::Punct(p) => match p.as_char() {
                    '.' => ".".to_string(),
                    '|' => "|".to_string(),
                    ':' => ": ".to_string(),
                    ';' => ";\n".to_string(),
                    ',' => ", ".to_string(),
                    p => format!(" {p} "),
                },
                TokenTree::Literal(l) => l.to_string(),
            }
        );
    }

    let out = out.replace("= >", "=>");
    let out = out.replace(". ", ".");
    let out = out.replace("( ", "(");
    let out = out.replace("= =", "=");

    let out = out.replace(": : ", "::");
    let out = out.replace(":: ", "::");
    let out = out.replace(" |", "|");
    out.replace("| |", "||")
}
