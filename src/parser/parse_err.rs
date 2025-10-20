use std::{error::Error, fmt::Display, num::ParseIntError};

pub type ParseResult<T> = std::result::Result<T, ParseErr>;

#[derive(Debug)]
pub struct ParseErr {
    /// Error message
    pub msg: String,
}

impl ParseErr {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }

    pub fn msg(&mut self, msg: String) {
        self.msg = msg;
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
        Self::new("".into())
    }
}

impl From<()> for ParseErr {
    fn from(_value: ()) -> Self {
        Self::new("".into())
    }
}
