use crate::lexer::cursor::Cursor;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // types
    Num(String),
    Bool(bool),
    Str(String),
    // assign
    Assign,
    AddAssign,
    SubAssign,
    Incr,
    Decr,
    // arithmetic
    Add,
    Sub,
    Mult,
    Div,
    Pow,
    // bool ops
    Not,
    Equals,
    NotEquals,
    Greater,
    GreaterEquals,
    Lesser,
    LesserEquals,
    // symbols
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Dot,
    // other
    Keyword(String),
    Identifier(String),
    NULL,
    EOL,
    EOF,
}

#[rustfmt::skip]
pub const KEYWORDS: &[&str] = &[
    "do",
    "end",
    "if",
    "for",
    "while",
    "return",
    "use",
    "self",
    // reserved for later
    "fn",
    "obj",
    "new",
    "throw",
    "self",
    "yeet",
    "amogus"
];

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
