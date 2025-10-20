use strum::EnumDiscriminants;

use crate::lexer::cursor::Cursor;

#[derive(Debug, PartialEq, Clone, EnumDiscriminants)]
pub enum TokenKind {
    // Literals
    Num(String),
    Bool(bool),
    Str(String),
    // Assign
    Assign,
    AddAssign,
    SubAssign,
    Incr,
    Decr,
    // Arithmetic
    Add,
    Sub,
    Mult,
    Div,
    Pow,
    // Boolean
    Not,
    Equals,
    NotEquals,
    Greater,
    GreaterEquals,
    Lesser,
    LesserEquals,
    // Symbols
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Dot,
    // Other
    Keyword(KeywordKind),
    Identifier(String),
    NULL,
    EOL,
    EOF,
}

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeywordKind {
    // Core
    Do,
    End,
    If,
    For,
    While,
    Return,
    Use,
    KSelf,
    Print,
    Var,
    // Reserved
    Fn,
    Obj,
    New,
    Err,
    Amogus,
}

impl ToString for KeywordKind {
    fn to_string(&self) -> String {
        match self {
            KeywordKind::Do => "do",
            KeywordKind::End => "end",
            KeywordKind::If => "if",
            KeywordKind::For => "for",
            KeywordKind::While => "while",
            KeywordKind::Return => "return",
            KeywordKind::Use => "use",
            KeywordKind::KSelf => "self",
            KeywordKind::Print => "print",
            KeywordKind::Var => "var",

            KeywordKind::Fn => "fn",
            KeywordKind::Obj => "obj",
            KeywordKind::New => "new",
            KeywordKind::Err => "err",
            KeywordKind::Amogus => "amogus",
        }
        .into()
    }
}

impl FromStr for KeywordKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, <KeywordKind as FromStr>::Err> {
        match s {
            "do" => Ok(KeywordKind::Do),
            "end" => Ok(KeywordKind::End),
            "if" => Ok(KeywordKind::If),
            "for" => Ok(KeywordKind::For),
            "while" => Ok(KeywordKind::While),
            "return" => Ok(KeywordKind::Return),
            "use" => Ok(KeywordKind::Use),
            "self" => Ok(KeywordKind::KSelf),
            "print" => Ok(KeywordKind::Print),
            "var" => Ok(KeywordKind::Var),

            "fn" => Ok(KeywordKind::Fn),
            "obj" => Ok(KeywordKind::Obj),
            "new" => Ok(KeywordKind::New),
            "err" => Ok(KeywordKind::Err),
            "amogus" => Ok(KeywordKind::Amogus),

            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    /// Kind of the token
    pub kind: TokenKind,
    /// Source lexeme string of the token
    pub lexeme: String,
    /// Location of the token as a Cursor
    pub cursor: Cursor,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, cursor: Cursor) -> Self {
        Self {
            kind,
            lexeme,
            cursor,
        }
    }
}
