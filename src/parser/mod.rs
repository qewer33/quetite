pub mod expr;
pub mod parse_err;
pub mod stmt;

use crate::{
    lexer::{
        cursor::Cursor,
        token::{KeywordKind, Token, TokenKind, TokenKindDiscriminants},
    },
    parser::{
        expr::{BinaryOp, Expr, ExprKind, LiteralType, UnaryOp},
        parse_err::{ParseErr, ParseResult},
        stmt::{Stmt, StmtKind},
    },
    reporter::Reporter,
};

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

    pub fn parse(&mut self) -> Option<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = vec![];

        self.skip_eols();

        while !self.is_at_end() {
            let stmt = self.declr();

            match stmt {
                Ok(stmt) => {
                    statements.push(stmt.clone());
                    self.skip_eols();
                }
                Err(err) => {
                    Reporter::error(&err.msg);
                    self.synchronize();
                    return None;
                }
            }
        }

        Some(statements)
    }

    // Grammar functions

    // Statements

    fn declr(&mut self) -> ParseResult<Stmt> {
        if self.match_keyword(KeywordKind::Var) {
            return self.var_declr();
        }

        self.stmt()
    }

    fn var_declr(&mut self) -> ParseResult<Stmt> {
        let ident = self.consume(TokenKindDiscriminants::Identifier, "Expected variable name")?;
        let name = if let TokenKind::Identifier(str) = ident.kind {
            str
        } else {
            unreachable!()
        };

        let mut init: Option<Expr> = None;
        if self.match_tokens(vec![TokenKindDiscriminants::Assign]) {
            init = Some(self.expr()?);
        }

        self.consume(
            TokenKindDiscriminants::EOL,
            "Expected EOL after variable declaration",
        );
        Ok(Stmt::new(
            StmtKind::Var { name, init },
            self.previous().cursor,
        ))
    }

    fn stmt(&mut self) -> ParseResult<Stmt> {
        if self.match_keyword(KeywordKind::Print) {
            return self.print_stmt();
        }
        if self.match_keyword(KeywordKind::Do) {
            return self.block_stmt();
        }

        self.expr_stmt()
    }

    fn expr_stmt(&mut self) -> ParseResult<Stmt> {
        let expr = self.expr()?;
        self.consume(
            TokenKindDiscriminants::EOL,
            "expected '\\n' after expression",
        )?;
        Ok(Stmt::new(StmtKind::Expr(expr), self.previous().cursor))
    }

    fn print_stmt(&mut self) -> ParseResult<Stmt> {
        let val = self.expr()?;
        self.consume(
            TokenKindDiscriminants::EOL,
            "expected '\\n' after expression",
        )?;
        Ok(Stmt::new(StmtKind::Print(val), self.previous().cursor))
    }

    fn block_stmt(&mut self) -> ParseResult<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();

        self.skip_eols();

        while !self.check_keyword(KeywordKind::End) && !self.is_at_end() {
            statements.push(self.declr()?);

            self.skip_eols();
        }

        self.consume_keyword(KeywordKind::End, "Expected closing \"do\" after block");
        Ok(Stmt::new(
            StmtKind::Block(statements),
            self.previous().cursor,
        ))
    }

    // Expressions

    fn expr(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.equality()?;

        if self.match_tokens(vec![TokenKindDiscriminants::Assign]) {
            let eq = self.previous();
            let val = self.assignment()?;

            if let ExprKind::Var(name) = expr.kind {
                return Ok(Expr::new(
                    ExprKind::Assign {
                        name,
                        val: Box::new(val),
                    },
                    self.previous().cursor,
                ));
            }

            return Err(ParseErr::new("Invalid assignment target".into()));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.comparison()?;
        while self.match_tokens(vec![
            TokenKindDiscriminants::NotEquals,
            TokenKindDiscriminants::Equals,
        ]) {
            let op = BinaryOp::try_from(&self.previous().kind).unwrap();
            let right = self.comparison()?;
            expr.kind = ExprKind::Binary {
                left: Box::new(expr.clone()),
                op,
                right: Box::new(right),
            };
            expr.cursor = self.previous().cursor;
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.term()?;

        while self.match_tokens(vec![
            TokenKindDiscriminants::Greater,
            TokenKindDiscriminants::GreaterEquals,
            TokenKindDiscriminants::Lesser,
            TokenKindDiscriminants::LesserEquals,
        ]) {
            let op = BinaryOp::try_from(&self.previous().kind).unwrap();
            let right = self.term()?;
            expr.kind = ExprKind::Binary {
                left: Box::new(expr.clone()),
                op,
                right: Box::new(right),
            };
            expr.cursor = self.previous().cursor;
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_tokens(vec![
            TokenKindDiscriminants::Sub,
            TokenKindDiscriminants::Add,
        ]) {
            let op = BinaryOp::try_from(&self.previous().kind).unwrap();
            let right = self.factor()?;
            expr.kind = ExprKind::Binary {
                left: Box::new(expr.clone()),
                op,
                right: Box::new(right),
            };
            expr.cursor = self.previous().cursor;
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_tokens(vec![
            TokenKindDiscriminants::Div,
            TokenKindDiscriminants::Mult,
            TokenKindDiscriminants::Pow,
        ]) {
            let op = BinaryOp::try_from(&self.previous().kind).unwrap();
            let right = self.unary()?;
            expr.kind = ExprKind::Binary {
                left: Box::new(expr.clone()),
                op,
                right: Box::new(right),
            };
            expr.cursor = self.previous().cursor;
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        while self.match_tokens(vec![
            TokenKindDiscriminants::Not,
            TokenKindDiscriminants::Sub,
        ]) {
            let op = UnaryOp::try_from(&self.previous().kind).unwrap();
            let right = self.unary()?;
            return Ok(Expr::new(
                ExprKind::Unary {
                    op,
                    right: Box::new(right),
                },
                self.previous().cursor,
            ));
        }

        Ok(self.primary()?)
    }

    fn primary(&mut self) -> ParseResult<Expr> {
        if self.match_tokens(vec![TokenKindDiscriminants::Bool]) {
            return Ok(Expr::new(
                ExprKind::Literal(LiteralType::Bool(true)),
                self.previous().cursor,
            ));
        }
        if self.match_tokens(vec![TokenKindDiscriminants::NULL]) {
            return Ok(Expr::new(
                ExprKind::Literal(LiteralType::Null),
                self.previous().cursor,
            ));
        }
        if self.match_tokens(vec![TokenKindDiscriminants::Num]) {
            if let TokenKind::Num(s) = self.previous().kind {
                return Ok(Expr::new(
                    ExprKind::Literal(LiteralType::Int(
                        s.parse::<i64>()
                            .map_err(|err| ParseErr::from(err).msg("invalid int literal".into()))?,
                    )),
                    self.previous().cursor,
                ));
            }
        }
        if self.match_tokens(vec![TokenKindDiscriminants::Str]) {
            if let TokenKind::Str(s) = self.previous().kind {
                return Ok(Expr::new(
                    ExprKind::Literal(LiteralType::Str(s)),
                    self.previous().cursor,
                ));
            }
        }
        if self.match_tokens(vec![TokenKindDiscriminants::LParen]) {
            let expr = self.expr()?;
            self.consume(
                TokenKindDiscriminants::RParen,
                "expected ')' after expression".into(),
            )?;
            return Ok(Expr::new(
                ExprKind::Grouping {
                    expr: Box::new(expr),
                },
                self.previous().cursor,
            ));
        }
        if self.match_tokens(vec![TokenKindDiscriminants::Identifier]) {
            if let TokenKind::Identifier(name) = self.previous().kind {
                return Ok(Expr::new(ExprKind::Var(name), self.previous().cursor));
            }
        }

        Err(ParseErr::new(format!(
            "Expected expression at {:?}",
            self.current().cursor,
        )))
    }

    // Util functions

    fn match_tokens(&mut self, tokens: Vec<TokenKindDiscriminants>) -> bool {
        let mut out = false;
        tokens.iter().for_each(|token| {
            if self.check(token.clone()) {
                self.next();
                out = true;
            }
        });

        out
    }

    fn match_keyword(&mut self, keyword: KeywordKind) -> bool {
        if self.check_keyword(keyword) {
            self.next();
            return true;
        }

        false
    }

    fn consume(&mut self, token: TokenKindDiscriminants, msg: &str) -> ParseResult<Token> {
        if self.check(token) {
            return Ok(self.next());
        }

        Err(ParseErr::new(format!(
            "Syntax error at {:?}, {}",
            self.current().cursor,
            msg
        )))
    }

    fn consume_multiple(
        &mut self,
        tokens: Vec<TokenKindDiscriminants>,
        msg: &str,
    ) -> ParseResult<Token> {
        let mut check = false;
        tokens.iter().for_each(|token| {
            if self.check(*token) {
                check = true;
            }
        });

        if check {
            return Ok(self.next());
        }

        Err(ParseErr::new(format!(
            "Syntax error at {:?}, {}",
            self.current().cursor,
            msg
        )))
    }

    fn consume_keyword(&mut self, keyword: KeywordKind, msg: &str) -> ParseResult<Token> {
        if self.check_keyword(keyword) {
            return Ok(self.next());
        }

        Err(ParseErr::new(format!(
            "Syntax error at {:?}, {}",
            self.current().cursor,
            msg
        )))
    }

    fn check(&self, token: TokenKindDiscriminants) -> bool {
        if self.is_at_end() {
            return false;
        }
        TokenKindDiscriminants::from(&self.current().kind) == token
    }

    fn check_keyword(&self, keyword: KeywordKind) -> bool {
        if self.is_at_end() {
            return false;
        }

        if let &TokenKind::Keyword(curr_keyword) = &self.current().kind {
            return curr_keyword == keyword;
        }

        false
    }

    fn current(&self) -> Token {
        self.tokens[self.curr].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.curr - 1].clone()
    }

    fn peek(&self) -> Token {
        self.tokens[self.curr + 1].clone()
    }

    fn next(&mut self) -> Token {
        self.curr += 1;

        if self.is_at_end() {
            return self.current();
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current().kind == TokenKind::EOF
    }

    fn skip_eols(&mut self) {
        while self.check(TokenKindDiscriminants::EOL) {
            self.next();
        }
    }

    // Error handling functions

    fn synchronize(&mut self) {
        self.next();

        while !self.is_at_end() {
            if self.previous().kind == TokenKind::EOL {
                return;
            }

            self.next();
        }
    }
}
