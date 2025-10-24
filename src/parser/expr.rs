use crate::lexer::{
    cursor::Cursor,
    token::{KeywordKind, TokenKind},
};

#[derive(Debug, Clone)]
pub enum ExprKind {
    Literal(LiteralType),
    Assign {
        name: String,
        op: AssignOp,
        val: Box<Expr>,
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
    Logical {
        left: Box<Expr>,
        op: LogicalOp,
        right: Box<Expr>,
    },
    Var(String),
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
    NotAssign(&'static str),
    NotUnary(&'static str),
    NotBinary(&'static str),
    NotLiteral(&'static str),
    NotLogical(&'static str),
    BadNumber(String),
}

#[derive(Debug, Clone)]
pub enum LiteralType {
    Null,
    // Float(f64),
    Num(f64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum AssignOp {
    Value,
    Add,
    Sub,
}

impl TryFrom<&TokenKind> for AssignOp {
    type Error = OpFromTokenError;

    fn try_from(t: &TokenKind) -> Result<Self, Self::Error> {
        match t {
            TokenKind::Assign => Ok(AssignOp::Value),
            TokenKind::AddAssign => Ok(AssignOp::Add),
            TokenKind::SubAssign => Ok(AssignOp::Sub),
            TokenKind::Incr => Ok(AssignOp::Add),
            TokenKind::Decr => Ok(AssignOp::Sub),
            _ => Err(OpFromTokenError::NotAssign("expected assign operator token")),
        }
    }
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
pub enum LogicalOp {
    And,
    Or,
}

impl TryFrom<&TokenKind> for LogicalOp {
    type Error = OpFromTokenError;

    fn try_from(t: &TokenKind) -> Result<Self, Self::Error> {
        let op = match t {
            TokenKind::Keyword(kind) => match kind {
                KeywordKind::And => LogicalOp::And,
                KeywordKind::Or => LogicalOp::Or,
                _ => {
                    return Err(OpFromTokenError::NotLogical(
                        "expected logical operator token",
                    ));
                }
            },
            _ => {
                return Err(OpFromTokenError::NotLogical(
                    "expected logical operator token",
                ));
            }
        };
        Ok(op)
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mult,
    Div,
    Mod,
    Pow,
    // Boolean
    Equals,
    NotEquals,
    Greater,
    GreaterEquals,
    Lesser,
    LesserEquals,
    Nullish,
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
            TokenKind::Mod => BinaryOp::Mod,
            TokenKind::Pow => BinaryOp::Pow,
            // Equality / comparison
            TokenKind::Equals => BinaryOp::Equals,
            TokenKind::NotEquals => BinaryOp::NotEquals,
            TokenKind::Greater => BinaryOp::Greater,
            TokenKind::GreaterEquals => BinaryOp::GreaterEquals,
            TokenKind::Lesser => BinaryOp::Lesser,
            TokenKind::LesserEquals => BinaryOp::LesserEquals,
            TokenKind::Nullish => BinaryOp::Nullish,
            _ => {
                return Err(OpFromTokenError::NotBinary(
                    "expected binary operator token",
                ));
            }
        };
        Ok(op)
    }
}
