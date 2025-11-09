use crate::{lexer::cursor::Cursor, parser::expr::Expr};

#[derive(Debug, Clone)]
pub enum StmtKind {
    Expr(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
    Var {
        name: String,
        init: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        declr: Option<Box<Stmt>>,
        condition: Expr,
        step: Option<Expr>,
        body: Box<Stmt>,
    },
    For {
        item: String,
        index: Option<String>,
        iter: Expr,
        body: Box<Stmt>
    },
    Fn {
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
        bound: bool,
    },
    Obj {
        name: String,
        methods: Vec<Stmt>,
    },
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
