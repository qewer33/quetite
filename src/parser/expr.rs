use crate::lexer::{cursor::Cursor, token::TokenKind};

#[derive(Debug, Clone)]
pub enum ExprKind {
    Literal(LiteralType),
    Assign {
        name: String,
        val: Box<Expr>
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        right: Box<Expr>,
    },
    Var(String)
}

#[derive(Debug, Clone)]
pub struct Expr {
    /// Kind of the expression
    pub kind: ExprKind,
    /// Location of the expression as a Cursor
    pub cursor: Cursor,
}

impl Expr {
    pub fn new(kind: ExprKind, cursor: Cursor) -> Self {
        Self { kind, cursor }
    }
}

/// Errors for TryFrom mappings
#[derive(Debug)]
pub enum OpFromTokenError {
    NotUnary(&'static str),
    NotBinary(&'static str),
    NotLiteral(&'static str),
    BadNumber(String),
}

#[derive(Debug, Clone)]
pub enum LiteralType {
    Null,
    // Float(f64),
    Int(i64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
}

impl TryFrom<&TokenKind> for UnaryOp {
    type Error = OpFromTokenError;

    fn try_from(t: &TokenKind) -> Result<Self, Self::Error> {
        match t {
            TokenKind::Sub => Ok(UnaryOp::Negate), // e.g., prefix minus
            TokenKind::Not => Ok(UnaryOp::Not),
            _ => Err(OpFromTokenError::NotUnary("expected unary operator token")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mult,
    Div,
    Pow,
    // Boolean
    Equals,
    NotEquals,
    Greater,
    GreaterEquals,
    Lesser,
    LesserEquals,
}

impl TryFrom<&TokenKind> for BinaryOp {
    type Error = OpFromTokenError;

    fn try_from(t: &TokenKind) -> Result<Self, Self::Error> {
        let op = match t {
            // Arithmetic
            TokenKind::Add => BinaryOp::Add,
            TokenKind::Sub => BinaryOp::Sub,
            TokenKind::Mult => BinaryOp::Mult,
            TokenKind::Div => BinaryOp::Div,
            TokenKind::Pow => BinaryOp::Pow,
            // Equality / comparison
            TokenKind::Equals => BinaryOp::Equals,
            TokenKind::NotEquals => BinaryOp::NotEquals,
            TokenKind::Greater => BinaryOp::Greater,
            TokenKind::GreaterEquals => BinaryOp::GreaterEquals,
            TokenKind::Lesser => BinaryOp::Lesser,
            TokenKind::LesserEquals => BinaryOp::LesserEquals,
            _ => {
                return Err(OpFromTokenError::NotBinary(
                    "expected binary operator token",
                ));
            }
        };
        Ok(op)
    }
}
