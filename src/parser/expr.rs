use ordered_float::OrderedFloat;
use std::cell::RefCell;

use crate::lexer::{
    cursor::Cursor,
    token::{KeywordKind, TokenKind},
};

#[derive(Debug, Clone)]
pub enum ExprKind {
    Literal(LiteralType),
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
        step: Option<Box<Expr>>,
    },
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
    Ternary {
        condition: Box<Expr>,
        true_branch: Box<Expr>,
        false_branch: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
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
    Get {
        obj: Box<Expr>,
        name: String,
    },
    Set {
        obj: Box<Expr>,
        name: String,
        op: AssignOp,
        val: Box<Expr>,
    },
    Index {
        obj: Box<Expr>,
        index: Box<Expr>,
    },
    IndexSet {
        obj: Box<Expr>,
        index: Box<Expr>,
        op: AssignOp,
        val: Box<Expr>,
    },
    ESelf,
}

#[derive(Debug, Clone)]
pub struct Expr {
    /// Kind of the expression
    pub kind: ExprKind,
    /// Location of the expression as a Cursor
    pub cursor: Cursor,
    /// Resolved distance
    pub resolved_dist: RefCell<Option<usize>>,
}

impl Expr {
    pub fn new(kind: ExprKind, cursor: Cursor) -> Self {
        Self {
            kind,
            cursor,
            resolved_dist: RefCell::new(None),
        }
    }

    pub fn resolve(&mut self, dist: usize) {
        *self.resolved_dist.borrow_mut() = Some(dist);
    }

    pub fn get_resolved_dist(&self) -> Option<usize> {
        *self.resolved_dist.borrow()
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
    Num(OrderedFloat<f64>),
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
            _ => Err(OpFromTokenError::NotAssign(
                "expected assign operator token",
            )),
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
