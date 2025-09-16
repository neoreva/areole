use std::iter::Peekable;

use logos::{Lexer, Logos};

use crate::{
    ast::Function,
    span::Span,
    test::TEST_CMD,
    token::{Kind, LexError, Token, TokenIter},
};

pub struct CommandParser<'src> {
    lexer: Peekable<logos::SpannedIter<'src, Kind<'src>>>,
    src: &'src str,
    last_token_loc: usize,
}

pub trait Parse<'src, T = Self> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, T>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError<'src> {
    LexError(LexError),
    // TODO: add a method to notate what kind of token was expected
    InvalidToken(Token<'src>),
    // TODO: Make spesific errors like:
    // "x" is not a valid number
    // based off in-game errors
    Eof,
}

pub type ParseResult<'src, T> = Result<T, ParseError<'src>>;

impl<'src> CommandParser<'src> {
    fn new(lexer: Lexer<'src, Kind<'src>>, src: &'src str) -> Self {
        Self {
            lexer: lexer.spanned().peekable(),
            src,
            last_token_loc: 0,
        }
    }
}

#[test]
fn test_parser() {
    let lex = Kind::lexer(TEST_CMD);

    let mut tokens = TokenIter::new(lex).peekable();
    dbg!(Function::parse(&mut tokens));
}
