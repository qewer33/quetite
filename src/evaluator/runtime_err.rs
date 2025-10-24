use std::{error::Error, fmt::Display};

use crate::lexer::cursor::Cursor;

pub type EvalResult<T> = std::result::Result<T, RuntimeErr>;

#[derive(Debug)]
pub struct RuntimeErr {
    /// Error message
    pub msg: String,
    /// Error location as a Cursor
    pub cursor: Cursor,
}

impl RuntimeErr {
    pub fn new(msg: String, cursor: Cursor) -> Self {
        Self { msg, cursor }
    }

    pub fn msg(&mut self, msg: String) {
        self.msg = msg;
    }

    pub fn cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }
}

impl Error for RuntimeErr {}

impl Display for RuntimeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
