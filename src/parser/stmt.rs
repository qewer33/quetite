use crate::{lexer::cursor::Cursor, parser::expr::Expr};

#[derive(Debug, Clone)]
pub enum StmtKind {
    Expr(Expr),
    Print(Expr),
    Var {
        name: String,
        init: Option<Expr>
    },
    Block(Vec<Stmt>)
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub cursor: Cursor,
}

impl Stmt {
    pub fn new(kind: StmtKind, cursor: Cursor) -> Self {
        Self { kind, cursor }
    }
}
