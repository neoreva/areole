use std::{borrow::Cow, iter::Peekable};

use crate::{
    parser::{Parse, ParseError, ParseResult},
    span::{Span, Spanned},
    token::{Kind, LexError, Token, TokenIter},
};

macro_rules! extract_token {
    ($tokens:ident, $(Kind::$ident:ident)|*) => {{
        let token = match $tokens.next() {
            Some(s) => s,
            None => return Err(ParseError::Eof),
        };
        // It is cleaner if the above match statement says seperate from the below
        match token {
        $(Ok(
                t @ Token {
                    span: _,
                    kind: Kind::$ident,
                },
            ) => t,
            )*
            Ok(tok) => return Err(ParseError::InvalidToken(tok)),
            Err(e) => return Err(ParseError::LexError(e)),
        }
    }};
    ($tokens:ident, Option<Kind::$ident:ident>) => {
        'label: {
            if !$tokens
                .peek()
                .is_some_and(|p| p.as_ref().is_ok_and(|t| t.kind == Kind::$ident))
            {
                break 'label None;
            }

            let token = match $tokens.next() {
                Some(s) => s,
                None => return Err(ParseError::Eof),
            };
            // It is cleaner if the above match statement says seperate from the below
            Some(match token {
                Ok(
                    t @ Token {
                        span: _,
                        kind: Kind::$ident,
                    },
                ) => t,

                Ok(tok) => return Err(ParseError::InvalidToken(tok)),
                Err(e) => return Err(ParseError::LexError(e)),
            })
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function<'src> {
    pub statements: Vec<Stmt<'src>>,
}

impl<'src> Function<'src> {
    pub fn new(statements: Vec<Stmt<'src>>) -> Self {
        Self { statements }
    }
}

impl<'src> Parse<'src> for Function<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let mut statements = Vec::new();
        while tokens.peek().is_some() {
            let statement = Stmt::parse(tokens)?;

            statements.push(statement)
        }
        Ok(Function::new(statements))
    }
}

impl<'src> Spanned for Function<'src> {
    fn span(&self) -> Span {
        Span::new(
            self.statements
                .first()
                .map(|s| s.span().start)
                .unwrap_or_default(),
            self.statements
                .last()
                .map(|s| s.span().end)
                .unwrap_or_default(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'src> {
    Command(StmtCommand<'src>),
    Comment(StmtComment<'src>),
}

impl<'src> Spanned for Stmt<'src> {
    fn span(&self) -> Span {
        match self {
            Stmt::Command(c) => c.span(),
            Stmt::Comment(c) => c.span(),
        }
    }
}

impl<'src> Parse<'src> for Stmt<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        match tokens.peek() {
            Some(Ok(Token {
                kind: Kind::Comment(_),
                span: _,
            })) => Ok(Stmt::Comment(StmtComment::parse(tokens)?)),

            Some(Ok(_)) => Ok(Stmt::Command(StmtCommand::parse(tokens)?)),

            Some(Err(err)) => Err(ParseError::LexError(err.clone())),
            None => Err(ParseError::Eof),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StmtCommand<'src> {
    slash: Option<Token<'src>>,
    ident: Ident<'src>,
    arguments: Option<Vec<Expr<'src>>>,
}

impl<'src> StmtCommand<'src> {
    pub fn new(
        slash: Option<Token<'src>>,
        ident: Ident<'src>,
        arguments: Option<Vec<Expr<'src>>>,
    ) -> Self {
        Self {
            slash,
            ident,
            arguments,
        }
    }
}

impl<'src> Spanned for StmtCommand<'src> {
    fn span(&self) -> Span {
        Span::new(
            if let Some(s) = &self.slash {
                s.span.start
            } else {
                self.ident.span.start
            },
            if let Some(Some(s)) = self.arguments.as_ref().map(|s| s.last()) {
                s.span().end
            } else {
                self.ident.span.end
            },
        )
    }
}

impl<'src> Parse<'src> for StmtCommand<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let slash = extract_token!(tokens, Option<Kind::Slash>);

        let ident = Ident::parse(tokens)?;

        if tokens.peek().is_none() {
            return Ok(StmtCommand::new(slash, ident, None));
        }

        let mut arguments = vec![];

        loop {
            let expr = Expr::parse(tokens)?;

            arguments.push(expr);

            if tokens.peek().is_none() {
                break;
            }
        }

        Ok(StmtCommand::new(slash, ident, Some(arguments)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'src> {
    Lit(Lit<'src>),
    Urnary(ExprUrnary<'src>),
    Range(ExprRange<'src>),
    Map(ExprMap<'src>),
    Target(ExprTarget<'src>),
}

impl<'src> Spanned for Expr<'src> {
    fn span(&self) -> Span {
        match self {
            Expr::Lit(lit) => lit.span(),
            Expr::Urnary(u) => u.span(),
            Expr::Range(r) => r.span(),
            Expr::Map(m) => m.span(),
            Expr::Target(t) => t.span(),
        }
    }
}

impl<'src> Parse<'src> for Expr<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        match tokens.peek() {}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprTarget<'src> {
    /// `@`
    select: Token<'src>,
    target: Ident<'src>,
    params: Option<Table<'src, Ident<'src>>>,
}

impl<'src> Spanned for ExprTarget<'src> {
    fn span(&self) -> Span {
        Span::new(
            self.select.span.start,
            if let Some(s) = &self.params {
                s.brackets.1.span.end
            } else {
                self.target.span.end
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table<'src, K> {
    brackets: (Token<'src>, Token<'src>),
    fields: Vec<TableField<'src, K>>,
}

impl<'src, K> Table<'src, K> {
    pub fn new(brackets: (Token<'src>, Token<'src>), fields: Vec<TableField<'src, K>>) -> Self {
        Self { brackets, fields }
    }
}

impl<'src, K> Spanned for Table<'src, K> {
    fn span(&self) -> Span {
        Span::new(self.brackets.0.span.start, self.brackets.1.span.end)
    }
}

impl<'src, K> Parse<'src> for Table<'src, K>
where
    K: Parse<'src>,
{
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let open = extract_token!(tokens, Kind::LeftBracket);

        // A Vec does not allocate right away.
        let mut fields = vec![];

        loop {
            let field = TableField::<'src, K>::parse(tokens)?;
            fields.push(field);
            match tokens.peek() {
                Some(Ok(Token {
                    kind: Kind::RightBracket,
                    span: _,
                })) => break,
                Some(Ok(_)) => continue,
                Some(Err(e)) => return Err(ParseError::LexError(e.clone())),
                None => return Err(ParseError::Eof),
            }
        }
        let close = extract_token!(tokens, Kind::RightBracket);

        Ok(Table::new((open, close), fields))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableField<'src, K> {
    key: K,
    eq: Token<'src>,
    value: Option<Expr<'src>>,
    comma: Option<Token<'src>>,
}

impl<'src, K> TableField<'src, K> {
    pub fn new(
        key: K,
        assign: Token<'src>,
        value: Option<Expr<'src>>,
        comma: Option<Token<'src>>,
    ) -> Self {
        Self {
            key,
            eq: assign,
            value,
            comma,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Delimited<Opn, T, Cls> {
    open: Opn,
    inner: T,
    close: Cls,
}

#[derive(Debug, Clone, PartialEq)]
struct Field<K, Eq, V> {
    key: K,
    eq: Eq,
    value: V,
}

struct Separated<T, Sep, const IS_TRAILING: bool> {
    // This uses an SOA.
    values: Vec<T>,
    separators: Vec<Sep>,
}

impl<K, S, V> Spanned for Field<K, S, V>
where
    K: Spanned,
    V: Spanned,
{
    fn span(&self) -> Span {
        Span::new(self.key.span().start, self.value.span().end)
    }
}

impl<'src, K> Parse<'src> for TableField<'src, K>
where
    K: Parse<'src>,
{
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let ident = K::parse(tokens)?;

        let assign = extract_token!(tokens, Kind::Equal);
        let mut comma = None;

        let value = match tokens.peek() {
            Some(Ok(Token {
                kind: Kind::Comma,
                span: _,
            })) => {
                comma = extract_token!(tokens, Option<Kind::Comma>);
                None
            }
            Some(Ok(Token {
                kind: Kind::Not,
                span: _,
            })) => {
                // TODO: This could just take the span from the `_` and
                // simply clone, rather than extracting the token,
                // but this should be fine for now
                let not = if let Some(s) = tokens.next()
                    && let Ok(token) = s
                {
                    token
                } else {
                    unreachable!()
                };

                let expr = if let Some(Ok(Token {
                    kind: Kind::Comma,
                    span: _,
                })) = tokens.peek()
                {
                    comma = extract_token!(tokens, Option<Kind::Comma>);
                    None
                } else {
                    Some(Box::new(Expr::parse(tokens)?))
                };

                let urnary = ExprUrnary::new(UnOp::Not(not), expr);

                Some(Expr::Urnary(urnary))
            }
            Some(Ok(_)) => {
                let expr = Expr::parse(tokens)?;

                comma = extract_token!(tokens, Option<Kind::Comma>);
                Some(expr)
            }
            Some(Err(err)) => return Err(ParseError::LexError(err.clone())),
            None => return Err(ParseError::Eof),
        };

        Ok(TableField::new(ident, assign, value, comma))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ident<'src> {
    value: Cow<'src, str>,
    span: Span,
}

impl<'src> Ident<'src> {
    pub fn new(value: Cow<'src, str>, span: Span) -> Self {
        Self { value, span }
    }
}

impl<'src> Spanned for Ident<'src> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<'src> Parse<'src> for Ident<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let token = match tokens.next() {
            Some(s) => s,
            None => return Err(ParseError::Eof),
        };
        match token {
            Ok(Token {
                span,
                kind: Kind::Ident(s),
            }) => Ok(Ident::new(Cow::Borrowed(s), span)),

            Ok(tok) => Err(ParseError::InvalidToken(tok)),
            Err(e) => Err(ParseError::LexError(e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprRange<'src> {
    start: Option<LitInt>,

    /// `..`
    limit: Token<'src>,

    end: Option<LitInt>,
}

impl<'src> ExprRange<'src> {
    pub fn new(start: Option<LitInt>, limit: Token<'src>, end: Option<LitInt>) -> Self {
        Self { start, limit, end }
    }
}

impl<'src> Spanned for ExprRange<'src> {
    fn span(&self) -> Span {
        Span::new(
            if let Some(s) = &self.start {
                s.span().start
            } else {
                self.limit.span.start
            },
            if let Some(s) = &self.end {
                s.span().end
            } else {
                self.limit.span.end
            },
        )
    }
}

impl<'src> Parse<'src> for ExprRange<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        fn parse_opt_int<'src>(
            tokens: &mut Peekable<TokenIter<'src>>,
        ) -> ParseResult<'src, Option<LitInt>> {
            Ok(match tokens.peek() {
                Some(Ok(Token {
                    kind: Kind::Int(_),
                    span: _,
                })) => Some(LitInt::parse(tokens)?),
                Some(Ok(_)) => None,
                Some(Err(err)) => return Err(ParseError::LexError(err.clone())),
                None => return Err(ParseError::Eof),
            })
        }

        let start = parse_opt_int(tokens)?;
        let limit = extract_token!(tokens, Kind::Limit);
        let end = parse_opt_int(tokens)?;

        Ok(ExprRange::new(start, limit, end))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprUrnary<'src> {
    pub op: UnOp<'src>,
    pub expr: Option<Box<Expr<'src>>>,
}

impl<'src> ExprUrnary<'src> {
    pub fn new(op: UnOp<'src>, expr: Option<Box<Expr<'src>>>) -> Self {
        Self { op, expr }
    }
}

impl<'src> Spanned for ExprUrnary<'src> {
    fn span(&self) -> Span {
        let op_span = self.op.span();

        Span::new(
            op_span.start,
            if let Some(s) = &self.expr {
                s.span().end
            } else {
                op_span.end
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp<'src> {
    /// `!`
    Not(Token<'src>),

    ///// `-`
    //Neg(Token<'src>),
    //
    /// `~`
    LocalCoordinate(Token<'src>),

    /// `^`
    RelativeCoordinate(Token<'src>),

    /// `§`
    FormatSelection(Token<'src>),
}

impl<'src> Spanned for UnOp<'src> {
    fn span(&self) -> Span {
        match self {
            UnOp::Not(token) => token.span(),
            // UnOp::Neg(token) => token.span(),
            UnOp::LocalCoordinate(token) => token.span(),
            UnOp::RelativeCoordinate(token) => token.span(),
            UnOp::FormatSelection(token) => token.span(),
        }
    }
}

impl<'src> Parse<'src> for ExprUrnary<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let op = {
            let token = match tokens.next() {
                Some(s) => s,
                None => return Err(ParseError::Eof),
            };
            match token {
                Ok(
                    t @ Token {
                        span: _,
                        kind: Kind::Not,
                    },
                ) => UnOp::Not(t),
                // Ok(
                //     t @ Token {
                //         span: _,
                //         kind: Kind::Neg,
                //     },
                // ) => UnOp::Neg(t),
                Ok(
                    t @ Token {
                        span: _,
                        kind: Kind::LocalCoordinate,
                    },
                ) => UnOp::LocalCoordinate(t),
                Ok(
                    t @ Token {
                        span: _,
                        kind: Kind::RelativeCoordinate,
                    },
                ) => UnOp::RelativeCoordinate(t),
                Ok(
                    t @ Token {
                        span: _,
                        kind: Kind::FormatSelection,
                    },
                ) => UnOp::FormatSelection(t),
                Ok(tok) => return Err(ParseError::InvalidToken(tok)),
                Err(e) => return Err(ParseError::LexError(e)),
            }
        };

        // TODO: Support ~~~
        let expr = Expr::parse(tokens)?;

        Ok(Self::new(op, Some(Box::new(expr))))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit<'src> {
    Int(LitInt),
    String(LitString<'src>),
    Bool(LitBool),
    Float(LitFloat),
    Path(LitPath<'src>),
}

impl<'src> Spanned for Lit<'src> {
    fn span(&self) -> Span {
        match self {
            Lit::Int(i) => i.span.clone(),
            Lit::String(s) => s.span.clone(),
            Lit::Bool(b) => b.span.clone(),
            Lit::Float(f) => f.span.clone(),
            Lit::Path(p) => p.span.clone(),
        }
    }
}

impl<'src> Parse<'src> for Lit<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let token = match tokens.peek() {
            Some(s) => s,
            None => return Err(ParseError::Eof),
        };

        let token @ Token { kind, span: _ } = match token {
            Ok(ok) => ok,
            Err(e) => return Err(ParseError::LexError(e.clone())),
        };

        Ok(match kind {
            Kind::Float(_) => Lit::Float(LitFloat::parse(tokens)?),
            Kind::Int(_) => Lit::Int(LitInt::parse(tokens)?),
            Kind::String(_) => Lit::String(LitString::parse(tokens)?),
            Kind::Path(_) => Lit::Path(LitPath::parse(tokens)?),
            Kind::Bool(_) => Lit::Bool(LitBool::parse(tokens)?),

            _ => return Err(ParseError::InvalidToken(token.clone())),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LitInt {
    pub value: i32,
    pub span: Span,
}

impl LitInt {
    pub fn new(value: i32, span: Span) -> Self {
        Self { value, span }
    }
}

impl Spanned for LitInt {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<'src> Parse<'src> for LitInt {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let token = match tokens.next() {
            Some(s) => s,
            None => return Err(ParseError::Eof),
        };

        match token {
            Ok(Token {
                span,
                kind: Kind::Int(s),
            }) => Ok(Self::new(s, span)),
            Ok(tok) => Err(ParseError::InvalidToken(tok)),
            Err(e) => Err(ParseError::LexError(e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LitFloat {
    pub value: f32,
    pub span: Span,
}

impl LitFloat {
    pub fn new(value: f32, span: Span) -> Self {
        Self { value, span }
    }
}

impl Spanned for LitFloat {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<'src> Parse<'src> for LitFloat {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let token = match tokens.next() {
            Some(s) => s,
            None => return Err(ParseError::Eof),
        };

        match token {
            Ok(Token {
                span,
                kind: Kind::Float(s),
            }) => Ok(Self::new(s, span)),
            Ok(tok) => Err(ParseError::InvalidToken(tok)),
            Err(e) => Err(ParseError::LexError(e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LitString<'src> {
    pub value: Cow<'src, str>,
    pub span: Span,
}

impl<'src> LitString<'src> {
    pub fn new(value: Cow<'src, str>, span: Span) -> Self {
        Self { value, span }
    }
}

impl<'src> Spanned for LitString<'src> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<'src> Parse<'src> for LitString<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let token = match tokens.next() {
            Some(s) => s,
            None => return Err(ParseError::Eof),
        };

        match token {
            Ok(Token {
                span,
                kind: Kind::String(s),
            }) => Ok(Self::new(Cow::Borrowed(s), span)),
            Ok(tok) => Err(ParseError::InvalidToken(tok)),
            Err(e) => Err(ParseError::LexError(e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprMap<'src> {
    pub curlies: (Token<'src>, Token<'src>),
    pub fields: Vec<ExprMapField<'src>>,
}

impl<'src> Spanned for ExprMap<'src> {
    fn span(&self) -> Span {
        Span::new(self.curlies.0.span.start, self.curlies.1.span.start)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprMapField<'src> {
    pub key: LitString<'src>,
    pub colon: Token<'src>,
    pub value: Expr<'src>,

    pub comma: Option<Token<'src>>,
}

impl<'src> ExprMapField<'src> {
    pub fn new(
        key: LitString<'src>,
        colon: Token<'src>,
        value: Expr<'src>,
        comma: Option<Token<'src>>,
    ) -> Self {
        Self {
            key,
            colon,
            value,
            comma,
        }
    }
}

impl<'src> Spanned for ExprMapField<'src> {
    fn span(&self) -> Span {
        Span::new(
            self.key.span.start,
            if let Some(s) = self.comma.as_ref() {
                s.span.end
            } else {
                self.value.span().end
            },
        )
    }
}

impl<'src> Parse<'src> for ExprMapField<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let key = LitString::parse(tokens)?;
        let colon = extract_token!(tokens, Kind::Colon);
        let value = Expr::parse(tokens)?;

        let comma = extract_token!(tokens, Option<Kind::Comma>);

        Ok(Self::new(key, colon, value, comma))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LitBool {
    pub value: bool,
    pub span: Span,
}

impl LitBool {
    pub fn new(value: bool, span: Span) -> Self {
        Self { value, span }
    }
}

impl Spanned for LitBool {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<'src> Parse<'src> for LitBool {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let token = match tokens.next() {
            Some(s) => s,
            None => return Err(ParseError::Eof),
        };

        match token {
            Ok(Token {
                span,
                kind: Kind::Bool(s),
            }) => Ok(Self::new(s, span)),
            Ok(tok) => Err(ParseError::InvalidToken(tok)),
            Err(e) => Err(ParseError::LexError(e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprOperator {
    pub value: Operator,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    /// `<>` or `=`
    Equal,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Gt,
    Lt,
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StmtComment<'src> {
    pub value: Cow<'src, str>,
    pub span: Span,
}

impl<'src> StmtComment<'src> {
    pub fn new(value: Cow<'src, str>, span: Span) -> Self {
        Self { value, span }
    }
}

impl<'src> Spanned for StmtComment<'src> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<'src> Parse<'src> for StmtComment<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let token = match tokens.next() {
            Some(s) => s,
            None => return Err(ParseError::Eof),
        };

        match token {
            Ok(Token {
                span,
                kind: Kind::Comment(s),
            }) => Ok(Self::new(Cow::Borrowed(s), span)),
            Ok(tok) => Err(ParseError::InvalidToken(tok)),
            Err(e) => Err(ParseError::LexError(e)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LitPath<'src> {
    pub value: Cow<'src, str>,
    pub span: Span,
}

impl<'src> LitPath<'src> {
    pub fn new(value: Cow<'src, str>, span: Span) -> Self {
        Self { value, span }
    }
}

impl<'src> Spanned for LitPath<'src> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<'src> Parse<'src> for LitPath<'src> {
    fn parse(tokens: &mut Peekable<TokenIter<'src>>) -> ParseResult<'src, Self> {
        let token = match tokens.next() {
            Some(s) => s,
            None => return Err(ParseError::Eof),
        };

        match token {
            Ok(Token {
                span,
                kind: Kind::Path(s),
            }) => Ok(Self::new(Cow::Borrowed(s), span)),
            Ok(tok) => Err(ParseError::InvalidToken(tok)),
            Err(e) => Err(ParseError::LexError(e)),
        }
    }
}
