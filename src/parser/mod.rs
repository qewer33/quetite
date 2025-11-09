pub mod expr;
pub mod parse_err;
pub mod stmt;

use strum::IntoDiscriminant;

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

#[derive(Default, Clone)]
pub struct ParserOutput {
    pub ast: Option<Vec<Stmt>>,
    pub errors: Option<Vec<ParseErr>>,
    pub error_count: usize,
    pub warning_count: usize,
}

impl ParserOutput {
    fn add_stmt(&mut self, stmt: Stmt) {
        if let None = self.ast {
            self.ast = Some(vec![]);
        }
        if let Some(ast) = self.ast.as_mut() {
            ast.push(stmt);
        }
    }

    fn add_err(&mut self, error: ParseErr) {
        if let None = self.errors {
            self.errors = Some(vec![]);
            self.ast = None;
        }
        if let Some(errors) = self.errors.as_mut() {
            errors.push(error);
            self.error_count += 1;
        }
    }
}

pub struct Parser<'a> {
    /// Source code
    src: &'a Src,
    /// Tokens to parse as a Vec
    tokens: Vec<Token>,
    /// Index of the current token
    curr: usize,
    /// Parser output
    out: ParserOutput,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a Src) -> Self {
        Self {
            src,
            tokens: src.tokens.as_ref().expect("ecpected tokens").clone(),
            curr: 0,
            out: ParserOutput::default(),
        }
    }

    pub fn parse(&mut self) -> ParserOutput {
        self.skip_eols();

        while !self.is_at_end() {
            let stmt = self.declr();

            match stmt {
                Ok(stmt) => {
                    self.out.add_stmt(stmt.clone());
                    self.skip_eols();
                }
                Err(err) => {
                    self.out.add_err(err.clone());
                    Reporter::parse_err_at(&err, self.src);
                    self.synchronize();
                }
            }
        }

        self.out.clone()
    }

    // Grammar functions

    // Statements

    fn declr(&mut self) -> ParseResult<Stmt> {
        if self.match_keyword(KeywordKind::Var) {
            return self.var_declr(true);
        }
        if self.match_keyword(KeywordKind::Fn) {
            return self.fn_declr();
        }
        if self.match_keyword(KeywordKind::Obj) {
            return self.obj_declr();
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

        if self.check_keyword(KeywordKind::While) {
            self.consume_keyword(
                KeywordKind::While,
                "expected 'while' after variable declaration",
            )?;
            return self.while_stmt(Some(Box::new(Stmt::new(
                StmtKind::Var { name, init },
                ident.cursor,
            ))));
        }

        if expect_eol {
            self.consume(
                TokenKindDiscriminants::EOL,
                "expected '\\n' after variable declaration",
            )?;
        }
        Ok(Stmt::new(StmtKind::Var { name, init }, ident.cursor))
    }

    fn fn_declr(&mut self) -> ParseResult<Stmt> {
        let name_token =
            self.consume(TokenKindDiscriminants::Identifier, "expected function name")?;
        let mut name = String::new();
        if let TokenKind::Identifier(ident) = name_token.kind {
            name = ident;
        }
        self.consume(
            TokenKindDiscriminants::LParen,
            "expected '(' after function name",
        )?;

        let mut bound = false;

        let mut params: Vec<String> = vec![];
        if !self.check(TokenKindDiscriminants::RParen) {
            loop {
                if params.len() >= 255 {
                    self.out.add_err(ParseErr::new(
                        "functions cannot have more than 255 arguments".into(),
                        self.current().cursor,
                    ));
                }

                if let TokenKind::Keyword(keyword) = self.current().kind {
                    if let KeywordKind::KSelf = keyword {
                        bound = true;
                        self.next();
                    }
                } else {
                    let ident = self.consume(
                        TokenKindDiscriminants::Identifier,
                        "expected parameter name",
                    )?;

                    if let TokenKind::Identifier(name) = ident.kind {
                        params.push(name);
                    }
                }

                if !self.match_tokens(vec![TokenKindDiscriminants::Comma]) {
                    break;
                }
            }
        }

        self.consume(
            TokenKindDiscriminants::RParen,
            "expected ')' after function parameters",
        )?;

        self.consume_keyword(KeywordKind::Do, "expected 'do' before function body")?;
        let body = self.block_stmt()?;
        Ok(Stmt::new(
            StmtKind::Fn {
                name,
                params,
                body: Box::new(body),
                bound,
            },
            name_token.cursor,
        ))
    }

    fn obj_declr(&mut self) -> ParseResult<Stmt> {
        let name_token =
            self.consume(TokenKindDiscriminants::Identifier, "expected object name")?;
        let mut name = String::new();
        if let TokenKind::Identifier(ident) = name_token.kind {
            name = ident;
        }

        self.consume_keyword(KeywordKind::Do, "expected 'do' before object body")?;
        self.skip_eols();

        let mut methods: Vec<Stmt> = vec![];
        while !self.check_keyword(KeywordKind::End) && !self.is_at_end() {
            methods.push(self.fn_declr()?);
            self.skip_eols();
        }

        self.consume_keyword(KeywordKind::End, "expected 'end' after object body")?;

        Ok(Stmt::new(
            StmtKind::Obj { name, methods },
            name_token.cursor,
        ))
    }

    fn stmt(&mut self) -> ParseResult<Stmt> {
        if self.match_keyword(KeywordKind::Return) {
            return self.return_stmt();
        }
        if self.match_keyword(KeywordKind::Break) {
            return self.break_stmt();
        }
        if self.match_keyword(KeywordKind::Continue) {
            return self.continue_stmt();
        }
        if self.match_keyword(KeywordKind::Do) {
            return self.block_stmt();
        }
        if self.match_keyword(KeywordKind::If) {
            return self.if_stmt();
        }
        if self.match_keyword(KeywordKind::While) {
            return self.while_stmt(None);
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

    fn for_stmt(&mut self) -> ParseResult<Stmt> {
        let item_ident = self.consume(
            TokenKindDiscriminants::Identifier,
            "expected item identifier after 'for'",
        )?;
        let item = if let TokenKind::Identifier(name) = item_ident.kind {
            name
        } else {
            unreachable!()
        };

        let mut index: Option<String> = None;
        if self.match_tokens(vec![TokenKindDiscriminants::Comma]) {
            let index_ident = self.consume(
                TokenKindDiscriminants::Identifier,
                "expected index identifier after ','",
            )?;
            index = if let TokenKind::Identifier(name) = index_ident.kind {
                Some(name)
            } else {
                unreachable!()
            };
        }

        self.consume_keyword(KeywordKind::In, "expected 'in' after variables")?;

        let iter = self.expr()?;

        self.consume_keyword(KeywordKind::Do, "expected 'do' after for statement")?;
        let body = self.block_stmt()?;

        let cursor = iter.cursor.clone();
        Ok(
            Stmt::new(
                StmtKind::For { item, index, iter, body: Box::new(body) },
                cursor
            )
        )
        
    }

    fn while_stmt(&mut self, declr: Option<Box<Stmt>>) -> ParseResult<Stmt> {
        let condition = self.expr()?;
        let step: Option<Expr> = if self.match_keyword(KeywordKind::Step) {
            Some(self.assignment()?)
        } else {
            None
        };
        let body = self.stmt()?;

        Ok(Stmt::new(
            StmtKind::While {
                declr,
                condition,
                body: Box::new(body),
                step,
            },
            self.previous().cursor,
        ))
    }

    fn return_stmt(&mut self) -> ParseResult<Stmt> {
        let mut val: Option<Expr> = None;

        if !self.check(TokenKindDiscriminants::EOL) {
            val = Some(self.expr()?);
        }

        self.consume(
            TokenKindDiscriminants::EOL,
            "expected '\\n' after return value",
        )?;
        Ok(Stmt::new(StmtKind::Return(val), self.previous().cursor))
    }

    fn break_stmt(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenKindDiscriminants::EOL, "expected '\\n' after break")?;
        Ok(Stmt::new(StmtKind::Break, self.previous().cursor))
    }

    fn continue_stmt(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenKindDiscriminants::EOL, "expected '\\n' after break")?;
        Ok(Stmt::new(StmtKind::Continue, self.previous().cursor))
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
            self.consume_keyword(KeywordKind::End, "Expected closing \"end\" after block")?;
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
            let mut val = Expr::new(
                ExprKind::Literal(LiteralType::Num(1.0)),
                self.current().cursor,
            );
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

            if let ExprKind::Get { obj, name } = expr.kind {
                return Ok(Expr::new(
                    ExprKind::Set {
                        obj,
                        name,
                        op,
                        val: Box::new(val),
                    },
                    self.previous().cursor,
                ));
            }

            if let ExprKind::Index { obj, index } = expr.kind {
                return Ok(Expr::new(
                    ExprKind::IndexSet {
                        obj,
                        index,
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

        Ok(self.call()?)
    }

    fn call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.range()?;

        loop {
            if self.match_tokens(vec![TokenKindDiscriminants::LParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_tokens(vec![TokenKindDiscriminants::LBracket]) {
                let index_expr = self.expr()?;
                self.consume(TokenKindDiscriminants::RBracket, "expected ']' after index")?;

                expr = Expr::new(
                    ExprKind::Index {
                        obj: Box::new(expr),
                        index: Box::new(index_expr),
                    },
                    self.previous().cursor,
                );
            } else if self.match_tokens(vec![TokenKindDiscriminants::Dot]) {
                let ident = self.consume(
                    TokenKindDiscriminants::Identifier,
                    "expected property name after '.'",
                )?;
                if let TokenKind::Identifier(name) = ident.kind {
                    expr = Expr::new(
                        ExprKind::Get {
                            obj: Box::new(expr),
                            name,
                        },
                        self.current().cursor,
                    );
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        let mut args: Vec<Expr> = vec![];

        if !self.check(TokenKindDiscriminants::RParen) {
            loop {
                if args.len() >= 255 {
                    self.out.add_err(ParseErr::new(
                        "functions cannot have more than 255 arguments".into(),
                        callee.cursor,
                    ));
                }

                args.push(self.expr()?);

                if !self.match_tokens(vec![TokenKindDiscriminants::Comma]) {
                    break;
                }
            }
        }

        let rparen = self.consume(
            TokenKindDiscriminants::RParen,
            "expected ')' after function arguments",
        )?;
        Ok(Expr::new(
            ExprKind::Call {
                callee: Box::new(callee),
                args,
            },
            rparen.cursor,
        ))
    }

    fn range(&mut self) -> ParseResult<Expr> {
        let start = self.list()?;

        if self.match_tokens(vec![
            TokenKindDiscriminants::Range,
            TokenKindDiscriminants::RangeEq,
        ]) {
            let inclusive = self.previous().kind == TokenKind::RangeEq;
            let end = self.expr()?;

            let mut step: Option<Box<Expr>> = None;
            if self.match_keyword(KeywordKind::Step) {
                step = Some(Box::new(self.expr()?));
            }

            return Ok(Expr::new(
                ExprKind::Range {
                    start: Box::new(start),
                    end: Box::new(end),
                    inclusive,
                    step,
                },
                self.current().cursor,
            ));
        }

        Ok(start)
    }

    fn list(&mut self) -> ParseResult<Expr> {
        if let TokenKind::LBracket = self.current().kind {
            self.next();

            let mut elements: Vec<Expr> = vec![];

            if !self.check(TokenKindDiscriminants::RBracket) {
                loop {
                    self.skip_eols();
                    elements.push(self.expr()?);

                    if !self.match_tokens(vec![TokenKindDiscriminants::Comma]) {
                        break;
                    }
                }
            }

            self.skip_eols();
            let rbrack = self.consume(
                TokenKindDiscriminants::RBracket,
                "expected ']' to end list definition".into(),
            )?;

            return Ok(Expr::new(ExprKind::List(elements), rbrack.cursor));
        }

        self.primary()
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
        if self.match_tokens(vec![TokenKindDiscriminants::Null]) {
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
        if self.match_keyword(KeywordKind::KSelf) {
            return Ok(Expr::new(ExprKind::ESelf, self.previous().cursor));
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

        Err(ParseErr::new(msg.into(), self.previous().cursor)
            .expected(token.to_string())
            .found(self.current().kind.discriminant().to_string()))
    }

    fn _consume_multiple(
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

        Err(ParseErr::new(msg.into(), self.current().cursor).expected(keyword.to_string()))
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
            match self.peek().kind {
                TokenKind::Keyword(keyword) => match keyword {
                    KeywordKind::Fn
                    | KeywordKind::Var
                    | KeywordKind::For
                    | KeywordKind::If
                    | KeywordKind::While => {
                        break;
                    }
                    _ => {}
                },
                _ => {}
            }

            self.next();
        }
    }
}
