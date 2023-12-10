use derive_syn_parse::Parse;
use syn::{LitChar, LitInt, Token};
use std::fmt::Debug;

fn flip<T, E>(o: Option<Result<T, E>>) -> Result<Option<T>, E> {
    match o {
        Some(Ok(x)) => Ok(Some(x)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

///
/// Formatting syntax based off [`std::fmt`].
///
pub struct Formatting {
    align: Option<Align>,
    sign: Option<Sign>,
    pretty: Option<Pretty>,
    zero: Option<Zero>,
    width: Option<Width>,
    precision: Option<DecimalPrecision>,
    _type: FormatType,
}

impl syn::parse::Parse for Formatting {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let align = flip(Align::peek(input).then(|| input.parse()))?;

        let sign = flip(Sign::peek(input).then(|| input.parse()))?;

        let pretty = flip(Pretty::peek(input).then(|| input.parse()))?;

        let Numbers(zero, width, precision) =
            flip(Numbers::peek(input).then(|| input.parse()))?.unwrap_or(Numbers(None, None, None));

        Ok(Self {
            align,
            sign,
            pretty,
            zero,
            width,
            precision,
            _type: input.parse()?,
        })
    }
}

impl Debug for Formatting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Formatting(")?;

        if let Some(ref align) = self.align {
            write!(f, "{align:?}, ")?;
        }

        if let Some(ref sign) = self.sign {
            write!(f, "{sign:?}, ")?;
        }

        if let Some(ref pretty) = self.pretty {
            write!(f, "{pretty:?}, ")?;
        }

        if let Some(ref zero) = self.zero {
            write!(f, "{zero:?}, ")?;
        }

        if let Some(ref width) = self.width {
            write!(f, "{width:?}, ")?;
        }

        if let Some(ref precision) = self.precision {
            write!(f, "{precision:?}, ")?;
        }

        write!(f, "{:?}", self._type)?;

        write!(f, ")")
    }
}

#[derive(Parse)]
pub struct Align {
    fill: Option<LitChar>,
    direction: AlignDirection,
}

impl Debug for Align {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Align(direction={:?}{})",
            self.direction,
            self.fill
                .as_ref()
                .map(|l| format!(", padding=`{}`", l.value()))
                .unwrap_or_default()
        )
    }
}

impl Align {
    fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![<])
            || input.peek(Token![^])
            || input.peek(Token![>])
            || input.peek2(Token![<])
            || input.peek2(Token![^])
            || input.peek2(Token![>])
    }
}

#[derive(Parse)]
pub enum AlignDirection {
    #[peek(syn::token::Gt, name = ">")]
    Right(Token![>]),

    #[peek(syn::token::Caret, name = "^")]
    Center(Token![^]),

    #[peek(syn::token::Lt, name = "<")]
    Left(Token![<]),
}

impl Debug for AlignDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left(_) => write!(f, "left"),
            Self::Center(_) => write!(f, "center"),
            Self::Right(_) => write!(f, "right"),
        }
    }
}

pub enum Sign {
    Positive(Token![+]),
    Negative(Token![-]),
}

impl syn::parse::Parse for Sign {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![+]) {
            return Ok(Self::Positive(input.parse().unwrap()));
        }

        if input.peek(Token![-]) {
            return Ok(Self::Negative(input.parse().unwrap()));
        }

        Err(input.error("Expected `+` or `-` here!"))
    }
}

impl Debug for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Positive(_) => write!(f, "Positive"),
            Self::Negative(_) => write!(f, "Negative"),
        }
    }
}

impl Sign {
    fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![+]) || input.peek(Token![-])
    }
}

#[derive(Parse)]
pub struct Pretty(Token![#]);

impl Pretty {
    fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![#])
    }
}

impl Debug for Pretty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pretty")
    }
}

///
/// Literally a `0`.
///
pub struct Zero(syn::LitInt);

impl Zero {
    fn peek(input: syn::parse::ParseStream) -> bool {
        let f = input.fork();
        let r: syn::Result<bool> = try {
            let i: LitInt = input.fork().parse()?;
            i.to_string().starts_with('0')
        };

        r.unwrap_or_default()
    }
}

impl Debug for Zero {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Zero")
    }
}

pub struct Width(syn::LitInt);

impl Debug for Width {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Width")
            .field(&self.0.base10_parse::<usize>().unwrap())
            .finish()
    }
}

struct Numbers(Option<Zero>, Option<Width>, Option<DecimalPrecision>);

impl Numbers {
    fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(syn::LitInt) || input.peek(syn::LitFloat)
    }
}

impl syn::parse::Parse for Numbers {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitInt) {
            let i: LitInt = input.parse()?;
            let s = i.to_string();

            let precision = if input.peek(Token![.]) {
                let decimal: Token![.] = input.parse()?;
                let precision: Precision = input.parse()?;
                Some(DecimalPrecision { decimal, precision })
            } else {
                None
            };

            if s.len() > 1 {
                return match (&s[0..1], &s[1..2]) {
                    ("0", "0") => Err(input.error("Expected maximum one leading `0` here")),
                    ("0", _) => Ok(Self(
                        Some(Zero(LitInt::new("0", i.span()))),
                        Some(Width(LitInt::new(&s[1..], i.span()))),
                        precision,
                    )),
                    _ => Ok(Self(None, Some(Width(i)), precision)),
                };
            } else {
                return Ok(Self(None, Some(Width(i)), precision));
            }
        }

        if input.peek(syn::LitFloat) {
            let float: syn::LitFloat = input.parse()?;
            if !float.suffix().is_empty() {
                return Err(input.error("Expected [`0`] [INT] [. INT] here"));
            }
            let float_s = float.to_string();
            let Some((int_s, after)) = float_s.split_once('.') else {
                return Err(input.error("Expected [`0`] [INT] [. INT] here"));
            };

            let precision: DecimalPrecision = if after.is_empty() {
                DecimalPrecision {
                    decimal: syn::token::Dot {
                        spans: [float.span()],
                    },
                    precision: input.parse()?,
                }
            } else {
                DecimalPrecision {
                    decimal: syn::token::Dot {
                        spans: [float.span()],
                    },
                    precision: Precision::Count(Count::Integer(LitInt::new(after, float.span()))),
                }
            };

            let int = LitInt::new(int_s, float.span());

            if int_s.len() > 1 {
                return match (&int_s[0..1], &int_s[1..2]) {
                    ("0", "0") => Err(input.error("Expected maximum one leading `0` here")),
                    ("0", _) => Ok(Self(
                        Some(Zero(LitInt::new("0", int.span()))),
                        Some(Width(LitInt::new(&int_s[1..], int.span()))),
                        Some(precision),
                    )),
                    _ => Ok(Self(None, Some(Width(int)), Some(precision))),
                };
            } else {
                return Ok(Self(None, Some(Width(int)), Some(precision)));
            }
        }

        Err(input.error("Expected [`0`] [INT] [. INT] here"))
    }
}

pub struct DecimalPrecision {
    decimal: Token![.],
    precision: Precision,
}

impl syn::parse::Parse for DecimalPrecision {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            decimal: input.parse()?,
            precision: input.parse()?,
        })
    }
}

impl Debug for DecimalPrecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Precision").field(&self.precision).finish()
    }
}

impl DecimalPrecision {
    fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(Token![.])
    }
}

pub enum Precision {
    Star(Token![*]),
    Count(Count),
}

impl syn::parse::Parse for Precision {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![*]) {
            return Ok(Self::Star(input.parse().unwrap()));
        }

        Ok(Self::Count(input.parse()?))
    }
}

impl Debug for Precision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Star(_) => write!(f, "*"),
            Self::Count(count) => write!(f, "{count:?}"),
        }
    }
}

pub enum Count {
    Parameter(Parameter),
    Integer(syn::LitInt),
}

impl syn::parse::Parse for Count {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitInt) {
            return Ok(Self::Integer(input.parse().unwrap()));
        }
        if input.peek(syn::Ident) && input.peek2(Token![$]) {
            return Ok(Self::Parameter(input.parse().unwrap()));
        }

        Err(input.error("Expected either integer or parameter `var$` here"))
    }
}

impl Debug for Count {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parameter(parameter) => write!(f, "{parameter:?}"),
            Self::Integer(arg0) => write!(f, "{}", arg0.base10_parse::<usize>().unwrap()),
        }
    }
}

#[derive(Parse)]
pub struct Parameter(syn::Ident, Token![$]);

impl Debug for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

pub enum FormatType {
    Display,
    Debug(Token![?]),
    DebugLowerHex(format_chars::x, Token![?]),
    DebugUpperHex(format_chars::X, Token![?]),
    Other(syn::Ident),
}

impl syn::parse::Parse for FormatType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::Display);
        }

        if input.peek(Token![?]) {
            return Ok(Self::Debug(input.parse()?));
        }

        if input.peek2(Token![?]) {
            let i: syn::Ident = input.parse()?;
            let question: Token![?] = input.parse()?;

            return Ok(match i.to_string().as_str() {
                "x" => Self::DebugLowerHex(format_chars::x(i), question),
                "X" => Self::DebugUpperHex(format_chars::X(i), question),
                _ => {
                    return Err(syn::Error::new_spanned(
                        i,
                        "Expected either `x` or `X` here!",
                    ))
                }
            });
        }

        Ok(Self::Other(input.parse()?))
    }
}

impl Debug for FormatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Display => write!(f, "Display"),
            Self::Debug(_) => write!(f, "Debug"),
            Self::DebugLowerHex(_, _) => write!(f, "Debug-LowerHex"),
            Self::DebugUpperHex(_, _) => write!(f, "Debug-UpperHex"),
            Self::Other(arg0) => f.debug_tuple("Other").field(arg0).finish(),
        }
    }
}

#[allow(non_camel_case_types)]
mod format_chars {
    pub struct x(pub(super) syn::Ident);
    pub struct X(pub(super) syn::Ident);
}

#[cfg(test)]
mod tests {
    use super::{
        Align, AlignDirection, Count, DecimalPrecision, FormatType, Parameter, Precision, Pretty,
        Sign, Width, Zero, Formatting
    };

    #[test]
    fn formatting_types() {
        assert!(matches!(
            syn::parse_str("").expect("Valid parse"),
            Formatting {
                _type: FormatType::Display,
                ..
            }
        ));

        assert!(matches!(
            syn::parse_str("?").expect("Valid parse"),
            Formatting {
                _type: FormatType::Debug(_),
                ..
            }
        ));

        assert!(matches!(
            syn::parse_str("x?").expect("Valid parse"),
            Formatting {
                _type: FormatType::DebugLowerHex(_, _),
                ..
            }
        ));

        assert!(matches!(
            syn::parse_str("X?").expect("Valid parse"),
            Formatting {
                _type: FormatType::DebugUpperHex(_, _),
                ..
            }
        ));

        assert!(matches!(
            syn::parse_str("o").expect("Valid parse"),
            Formatting {
                _type: FormatType::Other(_),
                ..
            }
        ));

        syn::parse_str::<Formatting>("o?").expect_err("Invalid parse");
        syn::parse_str::<Formatting>("adbjadsbvja?").expect_err("Invalid parse");
    }

    #[test]
    fn formatting_precision() {
        assert!(matches!(
            syn::parse_str(".*").expect("Valid parse"),
            Formatting {
                precision: Some(DecimalPrecision {
                    precision: Precision::Star(_),
                    ..
                }),
                ..
            }
        ));

        assert!(matches!(
            syn::parse_str(".5").expect("Valid parse"),
            Formatting {
                precision: Some(DecimalPrecision { precision: Precision::Count(Count::Integer(l)), .. }),
                ..
            } if l.base10_digits() == "5"
        ));

        assert!(matches!(
            syn::parse_str(".i$").expect("Valid parse"),
            Formatting {
                precision: Some(DecimalPrecision { precision: Precision::Count(Count::Parameter(Parameter(i, _))), .. }),
                ..
            } if i == "i"
        ));

        syn::parse_str::<Formatting>(".2$").expect_err("Invalid parse");
        syn::parse_str::<Formatting>(".applea nd$").expect_err("Invalid parse");
    }

    #[test]
    fn formatting_width() {
        assert!(matches!(
            syn::parse_str("5").expect("Valid parse"),
            Formatting {
                width: Some(Width(i)),
                ..
            } if i.base10_digits() == "5"
        ));

        assert!(matches!(
            syn::parse_str("05").expect("Valid parse"),
            Formatting {
                zero: Some(Zero(_)),
                width: Some(Width(i)),
                ..
            } if i.base10_digits() == "5"
        ));
    }

    #[test]
    fn formatting_pretty() {
        assert!(matches!(
            syn::parse_str("#").expect("Valid parse"),
            Formatting {
                pretty: Some(Pretty(_)),
                ..
            }
        ));

        assert!(matches!(
            syn::parse_str("").expect("Valid parse"),
            Formatting { pretty: None, .. }
        ));
    }

    #[test]
    fn formatting_sign() {
        assert!(matches!(
            syn::parse_str("+").expect("Valid parse"),
            Formatting {
                sign: Some(Sign::Positive(_)),
                ..
            }
        ));

        assert!(matches!(
            syn::parse_str("-").expect("Valid parse"),
            Formatting {
                sign: Some(Sign::Negative(_)),
                ..
            }
        ));

        assert!(matches!(
            syn::parse_str("").expect("Valid parse"),
            Formatting { sign: None, .. }
        ));
    }

    #[test]
    fn formatting_align() {
        assert!(matches!(
            syn::parse_str("<").expect("Valid parse"),
            Formatting {
                align: Some(Align {
                    fill: None,
                    direction: AlignDirection::Left(_)
                }),
                ..
            }
        ));
        assert!(matches!(
            syn::parse_str("^").expect("Valid parse"),
            Formatting {
                align: Some(Align {
                    fill: None,
                    direction: AlignDirection::Center(_)
                }),
                ..
            }
        ));
        assert!(matches!(
            syn::parse_str(">").expect("Valid parse"),
            Formatting {
                align: Some(Align {
                    fill: None,
                    direction: AlignDirection::Right(_)
                }),
                ..
            }
        ));
        assert!(matches!(
            syn::parse_str("'0'>").expect("Valid parse"),
            Formatting {
                align: Some(Align {
                    fill: Some(c),
                    direction: AlignDirection::Right(_)
                }),
                ..
            } if c.value() == '0'
        ));
    }

    #[test]
    fn formatting() {
        assert!(matches!(
            syn::parse_str("'0'>6").expect("Valid parse"),
            Formatting {
                align: Some(Align {
                    fill: Some(c),
                    direction: AlignDirection::Right(_),
                }),
                width: Some(Width(l)),
                ..
            } if c.value() == '0' && l.base10_parse::<usize>().unwrap() == 6
        ));

        assert!(matches!(
            syn::parse_str("'0'>+#6.5").expect("Valid parse"),
            Formatting {
                align: Some(Align {
                    fill: Some(c),
                    direction: AlignDirection::Right(_),
                }),
                sign: Some(Sign::Positive(_)),
                pretty: Some(Pretty(_)),
                width: Some(Width(l)),
                precision: Some(DecimalPrecision { precision: Precision::Count(Count::Integer(p)), ..}),
                ..
            } if c.value() == '0' && l.base10_parse::<usize>().unwrap() == 6 && p.base10_parse::<usize>().unwrap() == 5
        ));
        assert!(matches!(
            syn::parse_str("'2'>?").expect("Valid parse"),
            Formatting {
                align: Some(Align {
                    fill: Some(c),
                    direction: AlignDirection::Right(_),
                }),
                ..
            } if c.value() == '2'
        ));
    }
}
