use std::{
    error::Error,
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
};

use crate::lexer::cursor::Cursor;

pub type ParseResult<T> = std::result::Result<T, ParseErr>;

#[derive(Debug)]
pub struct ParseErr {
    /// Error message
    pub msg: String,
    /// Error location as a Cursor
    pub cursor: Cursor,
}

impl ParseErr {
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

impl Error for ParseErr {}

impl Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<ParseIntError> for ParseErr {
    fn from(_value: ParseIntError) -> Self {
        Self::new("".into(), Cursor::new())
    }
}

impl From<ParseFloatError> for ParseErr {
    fn from(_value: ParseFloatError) -> Self {
        Self::new("".into(), Cursor::new())
    }
}

impl From<()> for ParseErr {
    fn from(_value: ()) -> Self {
        Self::new("".into(), Cursor::new())
    }
}
