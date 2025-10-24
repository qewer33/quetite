pub mod expr;
pub mod parse_err;
pub mod stmt;

use crate::{
    lexer::token::{KeywordKind, Token, TokenKind, TokenKindDiscriminants},
    parser::{
        expr::{AssignOp, BinaryOp, Expr, ExprKind, LiteralType, LogicalOp, UnaryOp},
        parse_err::{ParseErr, ParseResult},
        stmt::{Stmt, StmtKind},
    },
    reporter::Reporter,
    src::Src,
};

pub struct Parser<'a> {
    /// Source code
    src: &'a Src,
    /// Tokens to parse as a Vec
    tokens: Vec<Token>,
    /// Index of the current token
    curr: usize,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a Src) -> Self {
        Self {
            src,
            tokens: src.tokens.as_ref().expect("ecpected tokens").clone(),
            curr: 0,
        }
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
                    Reporter::error_at(&err.msg, self.src, err.cursor);
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
            return self.var_declr(true);
        }

        self.stmt()
    }

    fn var_declr(&mut self, expect_eol: bool) -> ParseResult<Stmt> {
        let ident = self.consume(TokenKindDiscriminants::Identifier, "expected variable name")?;
        let name = if let TokenKind::Identifier(str) = ident.kind {
            str
        } else {
            unreachable!()
        };

        let mut init: Option<Expr> = None;
        if self.match_tokens(vec![TokenKindDiscriminants::Assign]) {
            init = Some(self.expr()?);
        }

        if expect_eol {
            self.consume(
                TokenKindDiscriminants::EOL,
                "expected '\\n' after variable declaration",
            )?;
        }
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
        if self.match_keyword(KeywordKind::If) {
            return self.if_stmt();
        }
        if self.match_keyword(KeywordKind::While) {
            return self.while_stmt();
        }
        if self.match_keyword(KeywordKind::For) {
            return self.for_stmt();
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

    fn if_stmt(&mut self) -> ParseResult<Stmt> {
        let condition = self.expr()?;

        let then_branch = Box::new(self.stmt()?);
        let mut else_branch: Option<Box<Stmt>> = None;
        if self.match_keyword(KeywordKind::Else) {
            else_branch = Some(Box::new(self.stmt()?));
        }

        Ok(Stmt::new(
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            },
            self.previous().cursor,
        ))
    }

    fn while_stmt(&mut self) -> ParseResult<Stmt> {
        let condition = self.expr()?;
        let body = self.stmt()?;

        Ok(Stmt::new(
            StmtKind::While {
                condition,
                body: Box::new(body),
            },
            self.previous().cursor,
        ))
    }

    fn for_stmt(&mut self) -> ParseResult<Stmt> {
        // INIT (optional)
        let init: Option<Stmt> = if self.check_keyword(KeywordKind::While)
            || self.check_keyword(KeywordKind::Do)
            || self.check_keyword(KeywordKind::Step)
        {
            None
        } else if self.match_keyword(KeywordKind::Var) {
            Some(self.var_declr(false)?)
        } else {
            let e = self.assignment()?; // or expr(); but assignment is fine/highest you need
            Some(Stmt::new(StmtKind::Expr(e), self.previous().cursor))
        };

        // CONDITION (optional)
        let condition = if self.match_keyword(KeywordKind::While) {
            self.assignment()? // parse the expression after 'while'
        } else {
            Expr::new(
                ExprKind::Literal(LiteralType::Bool(true)),
                self.previous().cursor,
            )
        };

        // INCREMENT (optional)
        let incr: Option<Expr> = if self.match_keyword(KeywordKind::Step) {
            Some(self.assignment()?)
        } else {
            None
        };

        // BODY
        self.consume_keyword(KeywordKind::Do, "expected 'do' before loop body")?;
        let mut body = self.block_stmt()?;

        // Desugar: { init; while (condition) { body; incr; } }
        if let Some(e) = incr {
            body = Stmt::new(
                StmtKind::Block(vec![
                    body,
                    Stmt::new(StmtKind::Expr(e), self.previous().cursor),
                ]),
                self.previous().cursor,
            );
        }

        let while_stmt = Stmt::new(
            StmtKind::While {
                condition,
                body: Box::new(body),
            },
            self.previous().cursor,
        );

        if let Some(init_stmt) = init {
            Ok(Stmt::new(
                StmtKind::Block(vec![init_stmt, while_stmt]),
                self.previous().cursor,
            ))
        } else {
            Ok(while_stmt)
        }
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

        while !self.check_keyword(KeywordKind::End)
            && !self.check_keyword(KeywordKind::Else)
            && !self.is_at_end()
        {
            statements.push(self.declr()?);

            self.skip_eols();
        }

        if !self.check_keyword(KeywordKind::Else) {
            self.consume_keyword(KeywordKind::End, "Expected closing \"do\" after block")?;
        }
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
        let expr = self.or()?;

        if self.match_tokens(vec![
            TokenKindDiscriminants::Assign,
            TokenKindDiscriminants::AddAssign,
            TokenKindDiscriminants::SubAssign,
            TokenKindDiscriminants::Incr,
            TokenKindDiscriminants::Decr,
        ]) {
            let op = AssignOp::try_from(&self.previous().kind).unwrap();
            let mut val = Expr {
                kind: ExprKind::Literal(LiteralType::Num(1.0)),
                cursor: self.current().cursor,
            };
            if self.previous().kind != TokenKind::Incr && self.previous().kind != TokenKind::Decr {
                val = self.assignment()?;
            }

            if let ExprKind::Var(name) = expr.kind {
                return Ok(Expr::new(
                    ExprKind::Assign {
                        name,
                        op,
                        val: Box::new(val),
                    },
                    self.previous().cursor,
                ));
            }

            return Err(ParseErr::new(
                "invalid assignment target".into(),
                self.previous().cursor,
            ));
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.and()?;

        while self.match_keyword(KeywordKind::Or) {
            let op = LogicalOp::try_from(&self.previous().kind).unwrap();
            let right = self.and()?;
            expr.kind = ExprKind::Logical {
                left: Box::new(expr.clone()),
                op,
                right: Box::new(right),
            };
            expr.cursor = self.previous().cursor;
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_keyword(KeywordKind::And) {
            let op = LogicalOp::try_from(&self.previous().kind).unwrap();
            let right = self.equality()?;
            expr.kind = ExprKind::Logical {
                left: Box::new(expr.clone()),
                op,
                right: Box::new(right),
            };
            expr.cursor = self.previous().cursor;
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
            TokenKindDiscriminants::Mod,
            TokenKindDiscriminants::Pow,
            TokenKindDiscriminants::Nullish,
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
            if let TokenKind::Bool(b) = self.previous().kind {
                return Ok(Expr::new(
                    ExprKind::Literal(LiteralType::Bool(b)),
                    self.previous().cursor,
                ));
            }
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
                    ExprKind::Literal(LiteralType::Num(
                        s.parse::<f64>()
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

        Err(ParseErr::new(
            "expected expression".into(),
            self.previous().cursor,
        ))
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

        Err(ParseErr::new(msg.into(), self.current().cursor))
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

        Err(ParseErr::new(msg.into(), self.current().cursor))
    }

    fn consume_keyword(&mut self, keyword: KeywordKind, msg: &str) -> ParseResult<Token> {
        if self.check_keyword(keyword) {
            return Ok(self.next());
        }

        Err(ParseErr::new(msg.into(), self.current().cursor))
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
