use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{
    evaluator::{Evaluator, runtime_err::RuntimeEvent},
    lexer::cursor::Cursor,
};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Callable(Rc<dyn Callable>),
    // Array(Rc<RefCell<Vec<Value>>>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Num(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Callable(c) => write!(f, "{:?}", c),
        }
    }
}

impl Value {
    pub fn is_equal(&self, other: &Value) -> bool {
        match self {
            Value::Null => {
                if let Value::Null = other {
                    return true;
                }
                return false;
            }
            Value::Bool(b) => {
                if let Value::Bool(ob) = other {
                    return b == ob;
                }
                return false;
            }
            Value::Num(n) => {
                if let Value::Num(on) = other {
                    return n == on;
                }
                return false;
            }
            Value::Str(s) => {
                if let Value::Str(os) = other {
                    return s == os;
                }
                return false;
            }
            Value::Callable(_) => {
                return false;
            }
        }
    }

    pub fn is_truthy(&self) -> bool {
        // false, 0 and Null are falsey values, everything else is thruthy
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Num(n) => *n == 0.,
            _ => true,
        }
    }

    pub fn check_num(&self, cursor: Cursor) -> Result<f64, RuntimeEvent> {
        if let Value::Num(num) = self {
            return Ok(*num);
        }
        Err(RuntimeEvent::error(
            format!("expected Num, found {:?}", self),
            cursor,
        ))
    }
}

pub trait Callable: Debug {
    fn name(&self) -> &str;
    fn arity(&self) -> usize;
    fn call(&self, evaluator: &mut Evaluator, args: Vec<Value>) -> Value;
}
