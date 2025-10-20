use std::fmt::Display;

use crate::evaluator::runtime_err::RuntimeErr;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    // Array(Rc<RefCell<Vec<Value>>>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Num(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s}"),
        }
    }
}

impl Value {
    pub fn is_equal(&self, other: &Value) -> bool {
        if let Value::Null = self
            && let Value::Null = other
        {
            return true;
        } else if let Value::Null = self {
            return false;
        }

        self.cast_num() == other.cast_num()
    }

    pub fn is_truthy(&self) -> bool {
        // false, 0 and Null are falsy values, everything else is thruthy
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Num(n) => *n == 0.,
            _ => true,
        }
    }

    pub fn cast_num(&self) -> f64 {
        match self {
            Value::Bool(b) => return (*b) as i32 as f64,
            Value::Null => return 0.,
            Value::Num(n) => return *n,
            Value::Str(s) => return s.len() as f64,
        }
    }

    pub fn check_num(&self) -> Result<f64, RuntimeErr> {
        if let Value::Num(num) = self {
            return Ok(*num);
        }
        Err(RuntimeErr::new(format!("Expected Num, found {:?}", self)))
    }
}
