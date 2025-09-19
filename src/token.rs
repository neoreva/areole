use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use logos::Logos;
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    span::{Span, Spanned},
    test::TEST_CMD,
};

#[derive(PartialEq, Debug, Clone)]
pub struct Token<'src> {
    pub kind: Kind<'src>,
    pub span: Span,
}

impl<'src> Token<'src> {
    pub fn new(kind: Kind<'src>, span: Span) -> Self {
        Self { kind, span }
    }
}

impl<'src> Spanned for Token<'src> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub enum LexErrorItem {
    InvalidFloat(std::num::ParseFloatError),

    InvalidInt(std::num::ParseIntError),

    InvalidBool(std::str::ParseBoolError),

    #[default]
    Unknown,
}

#[derive(PartialEq, Debug, Clone)]
pub struct LexError {
    span: Span,
    err: LexErrorItem,
}

impl LexError {
    pub fn new(err: LexErrorItem, span: Span) -> Self {
        Self { span, err }
    }
}

impl Spanned for LexError {
    fn span(&self) -> crate::span::Span {
        self.span.clone()
    }
}

impl From<std::num::ParseFloatError> for LexErrorItem {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::InvalidFloat(value)
    }
}

impl From<std::num::ParseIntError> for LexErrorItem {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::InvalidInt(value)
    }
}

impl From<std::str::ParseBoolError> for LexErrorItem {
    fn from(value: std::str::ParseBoolError) -> Self {
        Self::InvalidBool(value)
    }
}

impl std::fmt::Display for LexErrorItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexErrorItem::InvalidFloat(e) => write!(f, "{e}"),
            LexErrorItem::InvalidInt(e) => write!(f, "{e}"),
            LexErrorItem::InvalidBool(e) => write!(f, "{e}"),

            LexErrorItem::Unknown => write!(f, "Unknown Error"),
        }
    }
}

impl std::error::Error for LexErrorItem {}

#[derive(Logos, Debug, PartialEq, Clone, Default, Copy, Serialize, Deserialize)]
#[logos(skip r"[ \t\r]+", error = LexErrorItem)] // Ignore this regex pattern between tokens
pub enum Kind<'src> {
    #[default]
    Eof,
    #[regex("§.")]
    FormatSelection,

    #[regex(r"-?[0-9]*\.[0-9]+", |lex| lex.slice().parse(), priority =3)]
    Float(f32),

    #[regex("-?[0-9]+", |lex| lex.slice().parse(),  priority=3,)]
    Int(i32),

    #[regex("\"[^\"]+\"")]
    String(&'src str),

    #[regex("[a-z_.A-Z0-9]+")]
    Ident(&'src str),

    #[regex("[a-z_:.A-Z0-9]+/[a-z_:.A-Z0-9/]+", priority = 1)]
    Path(&'src str),

    #[token("/")]
    Slash,

    #[regex("", priority = 1)]
    #[token("}")]
    RightBrace,

    #[token("{")]
    LeftBrace,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[regex("@")]
    Selector,

    #[token(",")]
    Comma,

    #[token("-")]
    Neg,

    #[token("!")]
    Not,

    #[token("..")]
    Limit,
    /// Acts as `Equal` for scoreboards
    #[token("=")]
    Assign,

    // Scoreboard operators
    #[token("<>")]
    Equal,

    #[token("+=")]
    AddAssign,

    #[token("-=")]
    SubAssign,

    #[token("*=")]
    MulAssign,

    #[token("/=")]
    DivAssign,

    #[token(">")]
    Gt,

    #[token("<")]
    Lt,

    #[token("*")]
    Wildcard,

    #[regex("(true)|(false)", |lex| lex.slice().parse())]
    Bool(bool),

    #[token("~")]
    RelativeCoordinate,
    #[token("^")]
    LocalCoordinate,

    #[regex("#[^\n]+", |lex| lex.slice().trim_start_matches("#").trim())]
    Comment(&'src str),

    #[token("\n")]
    LineBreak,

    #[token(":")]
    Colon,
}

pub struct TokenIter<'src> {
    lex: logos::SpannedIter<'src, Kind<'src>>,
}

impl<'src> TokenIter<'src> {
    pub fn new(lex: logos::Lexer<'src, Kind<'src>>) -> Self {
        Self { lex: lex.spanned() }
    }
}

pub type LexResult<T> = Result<T, LexError>;

impl<'src> Iterator for TokenIter<'src> {
    type Item = LexResult<Token<'src>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.lex.next().map(|(res, span)| {
            let span = Span::from(span);
            match res {
                Ok(k) => Ok(Token::new(k, span)),
                Err(e) => Err(LexError::new(e, span)),
            }
        })
    }
}

#[test]
fn command_lex_test() {
    let lex = Kind::lexer(TEST_CMD);

    for (token, span) in lex.spanned() {
        match token {
            Ok(ok) => {
                println!(
                    "Kind:{ok:?}, Token: {}",
                    TEST_CMD[span].replace('\n', "\\n")
                );
            }
            Err(_) => panic!("Invalid Token: {}", &TEST_CMD[span]),
        }
    }
}
