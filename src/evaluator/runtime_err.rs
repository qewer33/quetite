use std::{error::Error, fmt::Display, io};

use crate::{evaluator::value::Value, lexer::cursor::Cursor};

pub type EvalResult<T> = std::result::Result<T, RuntimeEvent>;

#[derive(Debug)]
pub enum RuntimeEvent {
    Err(RuntimeErr),
    Return(Value),
    Break,
    Continue,
}

impl RuntimeEvent {
    pub fn error(msg: String, cursor: Cursor) -> Self {
        RuntimeEvent::Err(RuntimeErr {
            msg,
            cursor,
            note: None,
        })
    }

    pub fn error_with_note(msg: String, note: String, cursor: Cursor) -> Self {
        RuntimeEvent::Err(RuntimeErr {
            msg,
            cursor,
            note: Some(note),
        })
    }

    pub fn is_break(&self) -> bool {
        matches!(self, RuntimeEvent::Break)
    }
    pub fn is_continue(&self) -> bool {
        matches!(self, RuntimeEvent::Continue)
    }
    pub fn is_return(&self) -> bool {
        matches!(self, RuntimeEvent::Return(_))
    }
}

impl From<io::Error> for RuntimeEvent {
    fn from(err: io::Error) -> Self {
        RuntimeEvent::error(
            format!("IO error: {}", err),
            Cursor::new(),
        )
    }
}

#[derive(Debug)]
pub struct RuntimeErr {
    /// Error message
    pub msg: String,
    /// Error location as a Cursor
    pub cursor: Cursor,
    /// Friendly note for the user
    pub note: Option<String>,
}

impl RuntimeErr {
    pub fn new(msg: String, cursor: Cursor) -> Self {
        Self {
            msg,
            cursor,
            note: None,
        }
    }

    pub fn msg(mut self, msg: String) -> Self {
        self.msg = msg;
        self
    }

    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = cursor;
        self
    }

    pub fn note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }
}

impl Error for RuntimeErr {}

impl Display for RuntimeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
