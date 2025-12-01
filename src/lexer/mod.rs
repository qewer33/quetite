pub mod cursor;
pub mod token;

use std::str::FromStr;

use crate::lexer::cursor::Cursor;
use crate::lexer::token::{KeywordKind, Token, TokenKind};

#[derive(Default, Clone)]
pub struct LexerOutput {
    pub tokens: Option<Vec<Token>>,
    pub errors: Option<Vec<LexErr>>,
    pub error_count: usize,
}

#[derive(Clone, Debug)]
pub struct LexErr {
    pub msg: String,
    pub cursor: Cursor,
}

pub struct Lexer {
    /// The source code as a Vec<char>
    src: Vec<char>,
    /// Index of the current character
    curr: usize,
    /// Start of the current token being parsed
    start: usize,
    /// Current cursor location
    cursor: Cursor,
    /// Output
    out: LexerOutput,
}

impl Lexer {
    pub fn new(src: String) -> Self {
        Self {
            src: src.chars().collect(),
            curr: 0,
            start: 0,
            cursor: Cursor::new(),
            out: LexerOutput::default(),
        }
    }

    pub fn with_cursor(src: String, cursor: Cursor) -> Self {
        Self {
            src: src.chars().collect(),
            curr: 0,
            start: 0,
            cursor,
            out: LexerOutput::default(),
        }
    }

    pub fn tokenize(&mut self) -> LexerOutput {
        let mut tokens: Vec<Token> = Vec::new();

        while !self.is_at_end() {
            // Scan current char and identify token
            self.start = self.curr;
            let kind = self.scan_char();

            // Get lexeme of the identified token
            let lexeme = self.get_lexeme();

            // Build token
            if let Some(kind) = kind {
                let token = Token::new(kind, lexeme, self.cursor.clone());
                tokens.push(token);
            }
        }

        if let Some(token) = tokens.last() {
            if token.kind != TokenKind::EOL {
                tokens.push(Token::new(TokenKind::EOL, "".into(), self.cursor.clone()));
            }
        }
        tokens.push(Token::new(TokenKind::EOF, "".into(), self.cursor.clone()));
        if self.out.error_count == 0 {
            self.out.tokens = Some(tokens);
        }
        self.out.clone()
    }

    fn scan_char(&mut self) -> Option<TokenKind> {
        let c = self.current();

        let token = match c {
            // Types
            '"' => {
                let s = self.consume_string();
                Some(TokenKind::Str(s))
            }
            // Assign
            '=' => {
                if self.consume('=') {
                    self.next();
                    return Some(TokenKind::Equals);
                }

                self.next();
                Some(TokenKind::Assign)
            }
            // Arithmetic
            '+' => {
                if self.consume('=') {
                    self.next();
                    return Some(TokenKind::AddAssign);
                } else if self.consume('+') {
                    self.next();
                    return Some(TokenKind::Incr);
                }

                self.next();
                Some(TokenKind::Add)
            }
            '-' => {
                if self.consume('=') {
                    self.next();
                    return Some(TokenKind::SubAssign);
                } else if self.consume('-') {
                    self.next();
                    return Some(TokenKind::Decr);
                }

                self.next();
                Some(TokenKind::Sub)
            }
            '*' => {
                if self.consume('*') {
                    self.next();
                    return Some(TokenKind::Pow);
                }

                self.next();
                Some(TokenKind::Mult)
            }
            '/' => {
                self.next();
                Some(TokenKind::Div)
            }
            '%' => {
                self.next();
                Some(TokenKind::Mod)
            }
            // Bool ops
            '<' => {
                if self.consume('=') {
                    self.next();
                    return Some(TokenKind::LesserEquals);
                }

                self.next();
                Some(TokenKind::Lesser)
            }
            '>' => {
                if self.consume('=') {
                    self.next();
                    return Some(TokenKind::GreaterEquals);
                }

                self.next();
                Some(TokenKind::Greater)
            }
            '!' => {
                if self.consume('=') {
                    self.next();
                    return Some(TokenKind::NotEquals);
                }

                self.next();
                Some(TokenKind::Not)
            }
            ':' => {
                self.next();
                Some(TokenKind::Colon)
            }
            '?' => {
                if self.consume('?') {
                    self.next();
                    return Some(TokenKind::Nullish);
                }

                self.next();
                Some(TokenKind::Question)
            }
            // Symbols
            '(' => {
                self.next();
                Some(TokenKind::LParen)
            }
            ')' => {
                self.next();
                Some(TokenKind::RParen)
            }
            '[' => {
                self.next();
                Some(TokenKind::LBracket)
            }
            ']' => {
                self.next();
                Some(TokenKind::RBracket)
            }
            '{' => {
                self.next();
                Some(TokenKind::LBrace)
            }
            '}' => {
                self.next();
                Some(TokenKind::RBrace)
            }
            ',' => {
                self.next();
                Some(TokenKind::Comma)
            }
            '.' => {
                if self.consume('.') {
                    if self.consume('=') {
                        self.next();
                        return Some(TokenKind::RangeEq);
                    }

                    self.next();
                    return Some(TokenKind::Range);
                }

                self.next();
                Some(TokenKind::Dot)
            }
            // Other
            '\r' => {
                // handle Windows CRLF as a single EOL
                if self.peek() == '\n' {
                    self.next();
                }
                self.next();
                Some(TokenKind::EOL)
            }
            '\n' => {
                self.next();
                Some(TokenKind::EOL)
            }

            '#' => {
                // consume comment chars, stop before newline (so it will emit EOL on next loop)
                self.next(); // skip '#'
                while !self.is_at_end() && self.current() != '\n' {
                    self.next();
                }
                None
            }
            ' ' | '\t' => {
                self.next();
                None
            }
            _ => {
                // check types
                if let Some(bool) = self.check_bool() {
                    self.next();
                    return Some(TokenKind::Bool(bool));
                }

                if let Some(num) = self.check_num() {
                    self.next();
                    return Some(TokenKind::Num(num));
                }

                // check keywords, assume identifiers if it doesn't match any
                let mut str = String::new();

                // symbols accepted inside identifiers
                let accepted_symbols = ['_'];
                loop {
                    str.push(self.current());

                    let peek = self.peek();
                    if !(peek.is_alphanumeric() || accepted_symbols.contains(&peek)) {
                        break;
                    }
                    self.next();
                }

                self.next();
                if let Ok(kind) = KeywordKind::from_str(str.as_str()) {
                    return Some(TokenKind::Keyword(kind));
                }

                if str == "Null".to_string() {
                    return Some(TokenKind::Null);
                }

                Some(TokenKind::Identifier(str))
            }
        };

        token
    }

    // Type checks

    fn check_bool(&mut self) -> Option<bool> {
        if self.consume_str("true") {
            return Some(true);
        } else if self.consume_str("false") {
            return Some(false);
        }
        None
    }

    fn check_num(&mut self) -> Option<String> {
        if !self.current().is_numeric() {
            return None;
        }

        let mut num = String::new();
        let mut seen_dot = false;

        // consume the first digit (current)
        num.push(self.current());

        loop {
            let nxt = self.peek();

            // more digits?
            if nxt.is_numeric() {
                self.next(); // move onto that digit
                num.push(self.current());
                continue;
            }

            // optional single '.' with a digit after it
            if !seen_dot && nxt == '.' {
                // ensure we have a digit after the dot
                let after_dot = if self.curr + 2 < self.src.len() {
                    self.src[self.curr + 2]
                } else {
                    ' '
                };
                if after_dot.is_numeric() {
                    seen_dot = true;
                    self.next(); // move onto '.'
                    num.push('.');

                    self.next(); // move onto first frac digit
                    num.push(self.current());

                    // consume remaining fractional digits
                    while self.peek().is_numeric() {
                        self.next();
                        num.push(self.current());
                    }
                    continue;
                }
            }

            // next char is not part of the number → stop WITHOUT advancing
            break;
        }

        Some(num)
    }

    // Iter utils

    fn current(&self) -> char {
        if self.curr >= self.src.len() {
            return ' ';
        }

        self.src[self.curr]
    }

    fn next(&mut self) -> char {
        // Advance cursor
        if self.current() == '\n' {
            self.cursor.next_line();
        } else {
            self.cursor.next_col();
        }

        // Advance index
        self.curr += 1;
        if self.is_at_end() {
            return ' ';
        }

        self.current()
    }

    fn peek(&self) -> char {
        if self.curr + 1 >= self.src.len() {
            return ' ';
        }

        self.src[self.curr + 1]
    }

    fn consume(&mut self, c: char) -> bool {
        if self.curr + 1 >= self.src.len() {
            return false;
        }

        if c == self.src[self.curr + 1] {
            self.next();
            return true;
        }
        false
    }

    // TODO: check this
    fn consume_str(&mut self, s: &str) -> bool {
        let s_chars: Vec<char> = s.chars().collect();
        let needed = s_chars.len();
        let end = self.curr + needed;

        if end > self.src.len() {
            return false;
        }

        if self.src[self.curr..end] == s_chars[..] {
            // advance to the last matched char (caller will do one `next()` after)
            for _ in 0..needed.saturating_sub(1) {
                self.next();
            }
            return true;
        }

        false
    }

    fn consume_string(&mut self) -> String {
        let mut out = String::new();
        // skip opening quote
        self.next();
        let mut terminated = false;

        while !self.is_at_end() {
            let ch = self.current();
            if ch == '"' {
                // closing quote, consume it and finish
                self.next();
                terminated = true;
                break;
            }

            if ch == '\\' {
                let esc = self.peek();
                let mapped = match esc {
                    '\\' => Some('\\'),
                    '"' => Some('"'),
                    'n' => Some('\n'),
                    't' => Some('\t'),
                    'r' => Some('\r'),
                    _ => None,
                };
                // advance over the escape char
                self.next();
                if let Some(m) = mapped {
                    self.next();
                    out.push(m);
                    continue;
                } else {
                    // unknown escape, keep the backslash literal
                    out.push('\\');
                    continue;
                }
            }

            out.push(ch);
            self.next();
        }

        if !terminated {
            self.out.error_count += 1;
            let err = LexErr {
                msg: "unterminated string literal".into(),
                cursor: self.cursor,
            };
            self.out.errors.get_or_insert(Vec::new()).push(err.clone());
        }

        out
    }

    fn get_lexeme(&self) -> String {
        if self.is_at_end() {
            return "".into();
        }

        let len = self.curr - self.start;
        self.src[self.start..self.start + len]
            .iter()
            .map(|&c| c as char)
            .collect()
    }

    fn is_at_end(&self) -> bool {
        self.curr == self.src.len()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(src: &str) -> Vec<TokenKind> {
        let mut lx = Lexer::new(src.to_string());
        lx.tokenize()
            .tokens
            .unwrap_or_default()
            .iter()
            .map(|token| token.kind.clone())
            .collect()
    }

    #[test]
    fn empty_input() {
        assert_eq!(tokens(""), vec![TokenKind::EOF]);
    }

    #[test]
    fn simple_assign() {
        assert_eq!(
            tokens("a = 10\n"),
            vec![
                TokenKind::Identifier("a".into()),
                TokenKind::Assign,
                TokenKind::Num("10".into()),
                TokenKind::EOL,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn call_with_arg() {
        assert_eq!(
            tokens("print(a)\n"),
            vec![
                TokenKind::Identifier("print".into()),
                TokenKind::LParen,
                TokenKind::Identifier("a".into()),
                TokenKind::RParen,
                TokenKind::EOL,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn function_def_line() {
        assert_eq!(
            tokens("sq(n) = n*n\n"),
            vec![
                TokenKind::Identifier("sq".into()),
                TokenKind::LParen,
                TokenKind::Identifier("n".into()),
                TokenKind::RParen,
                TokenKind::Assign,
                TokenKind::Identifier("n".into()),
                TokenKind::Mult,
                TokenKind::Identifier("n".into()),
                TokenKind::EOL,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn if_block_skeleton() {
        assert_eq!(
            tokens("if a == 100 do\nend\n"),
            vec![
                TokenKind::Keyword(KeywordKind::If),
                TokenKind::Identifier("a".into()),
                TokenKind::Equals,
                TokenKind::Num("100".into()),
                TokenKind::Keyword(KeywordKind::Do),
                TokenKind::EOL,
                TokenKind::Keyword(KeywordKind::End),
                TokenKind::EOL,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn string_literal() {
        assert_eq!(
            tokens("print(\"poggers\")\n"),
            vec![
                TokenKind::Identifier("print".into()),
                TokenKind::LParen,
                TokenKind::Str("poggers".into()),
                TokenKind::RParen,
                TokenKind::EOL,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn two_char_ops() {
        assert_eq!(
            tokens("a!=b\nc>=d\ne<=f\n"),
            vec![
                TokenKind::Identifier("a".into()),
                TokenKind::NotEquals,
                TokenKind::Identifier("b".into()),
                TokenKind::EOL,
                TokenKind::Identifier("c".into()),
                TokenKind::GreaterEquals,
                TokenKind::Identifier("d".into()),
                TokenKind::EOL,
                TokenKind::Identifier("e".into()),
                TokenKind::LesserEquals,
                TokenKind::Identifier("f".into()),
                TokenKind::EOL,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn blank_lines() {
        assert_eq!(
            tokens("\n\n"),
            vec![TokenKind::EOL, TokenKind::EOL, TokenKind::EOF]
        );
    }

    #[test]
    fn string_at_eof() {
        assert_eq!(
            tokens("print(\"x\")"),
            vec![
                TokenKind::Identifier("print".into()),
                TokenKind::LParen,
                TokenKind::Str("x".into()),
                TokenKind::RParen,
                TokenKind::EOL,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn comment_then_identifier() {
        // Assumes you EMIT a Comment token and then an EOL after it.
        assert_eq!(
            tokens("# this is a comment\nx\n"),
            vec![
                TokenKind::EOL,
                TokenKind::Identifier("x".into()),
                TokenKind::EOL,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn keywords_vs_identifiers() {
        assert_eq!(
            tokens("do end if print dox\n"),
            vec![
                TokenKind::Keyword(KeywordKind::Do),
                TokenKind::Keyword(KeywordKind::End),
                TokenKind::Keyword(KeywordKind::If),
                TokenKind::Identifier("print".into()),
                TokenKind::Identifier("dox".into()),
                TokenKind::EOL,
                TokenKind::EOF
            ]
        );
    }
}
