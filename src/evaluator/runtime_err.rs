use std::{error::Error, fmt::Display};

use crate::evaluator::value::Value;

pub type EvalResult<T> = std::result::Result<T, RuntimeErr>;

#[derive(Debug)]
pub struct RuntimeErr {
    /// Error message
    pub msg: String,
}

impl RuntimeErr {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }

    pub fn msg(&mut self, msg: String) {
        self.msg = msg;
    }
}

impl Error for RuntimeErr {}

impl Display for RuntimeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}