use std::{error::Error, fmt::Display, mem::discriminant, num::ParseIntError};

use crate::lexer::token::{Token, TokenKind};

#[derive(Debug)]
pub enum Expr {
    NullLiteral,
    LiteralInt(i64),
    LiteralStr(String),
    LiteralBool(bool),
    Binary {
        left: Box<Expr>,
        op: TokenKind,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Unary {
        op: TokenKind,
        right: Box<Expr>,
    },
}

type ParseResult<T> = std::result::Result<T, ParseErr>;

#[derive(Debug)]
pub struct ParseErr {
    /// Error message
    msg: String,
}

impl ParseErr {
    fn new(msg: String) -> Self {
        Self { msg }
    }

    fn msg(&mut self, msg: String) {
        self.msg = msg;
    }
}

impl Error for ParseErr {}

impl Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<ParseIntError> for ParseErr {
    fn from(_value: ParseIntError) -> Self {
        Self::new("".into())
    }
}

impl From<()> for ParseErr {
    fn from(_value: ()) -> Self {
        Self::new("".into())
    }
}

pub struct Parser {
    /// Tokens to parse as a Vec
    tokens: Vec<Token>,
    /// Index of the current token
    curr: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, curr: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Expr> {
        self.expr()
    }

    // grammar functions

    fn expr(&mut self) -> ParseResult<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;

        while self.match_tokens(vec![TokenKind::NotEquals, TokenKind::Equals]) {
            let op = self.previous().kind;
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;

        while self.match_tokens(vec![
            TokenKind::Greater,
            TokenKind::GreaterEquals,
            TokenKind::Lesser,
            TokenKind::LesserEquals,
        ]) {
            let op = self.previous().kind;
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_tokens(vec![TokenKind::Sub, TokenKind::Add]) {
            let op = self.previous().kind;
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_tokens(vec![TokenKind::Div, TokenKind::Mult, TokenKind::Pow]) {
            let op = self.previous().kind;
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        while self.match_tokens(vec![TokenKind::Not, TokenKind::Sub]) {
            let op = self.previous().kind;
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op,
                right: Box::new(right),
            });
        }

        Ok(self.primary()?)
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        if self.match_tokens(vec![TokenKind::Bool(true)]) {
            return Ok(Expr::LiteralBool(true));
        }
        if self.match_tokens(vec![TokenKind::Bool(false)]) {
            return Ok(Expr::LiteralBool(false));
        }
        if self.match_tokens(vec![TokenKind::NULL]) {
            return Ok(Expr::NullLiteral);
        }
        if self.match_tokens(vec![TokenKind::Num("".into())]) {
            if let TokenKind::Num(s) = self.previous().kind {
                return Ok(Expr::LiteralInt(s.parse::<i64>().map_err(|err| {
                    ParseErr::from(err).msg("invalid int literal".into())
                })?));
            }
        }
        if self.match_tokens(vec![TokenKind::Str("".into())]) {
            if let TokenKind::Str(s) = self.previous().kind {
                return Ok(Expr::LiteralStr(s));
            }
        }
        if self.match_tokens(vec![TokenKind::Bool(true)]) {
            if let TokenKind::Bool(b) = self.previous().kind {
                return Ok(Expr::LiteralBool(b));
            }
        }
        if self.match_tokens(vec![TokenKind::LParen]) {
            let expr = self.expr()?;
            self.consume(TokenKind::RParen, "expected ')' after expression".into())?;
            return Ok(Expr::Grouping {
                expr: Box::new(expr),
            });
        }

        Err(ParseErr::new("expected expression".into()))
    }

    // util functions

    fn match_tokens(&mut self, tokens: Vec<TokenKind>) -> bool {
        let mut out = false;
        tokens.iter().for_each(|token| {
            if self.check(token.clone()) {
                self.next();
                out = true;
            }
        });

        out
    }

    fn consume(&mut self, token: TokenKind, msg: &str) -> ParseResult<Token> {
        if self.check(token) {
            return Ok(self.next());
        }

        Err(ParseErr::new(
            format!("Syntax error at {:?}, {}", self.current().cursor, msg)
        ))
    }

    fn check(&self, token: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        discriminant(&self.current().kind) == discriminant(&token)
    }

    fn current(&self) -> Token {
        self.tokens[self.curr].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.curr - 1].clone()
    }

    fn next(&mut self) -> Token {
        self.curr += 1;

        if self.is_at_end() {
            return self.current();
        }

        self.current()
    }

    fn is_at_end(&self) -> bool {
        self.current().kind == TokenKind::EOF
    }
}
