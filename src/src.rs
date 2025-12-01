use std::{fs, path::PathBuf};

use crate::{lexer::token::Token, parser::stmt::Stmt};

pub struct Src {
    pub file: PathBuf,
    pub text: String,
    pub lines: Vec<String>,
    pub tokens: Option<Vec<Token>>,
    pub ast: Option<Vec<Stmt>>,
}

impl Src {
    pub fn new(file: PathBuf) -> Self {
        let text = match fs::read_to_string(&file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: failed to read {}: {e}", file.display());
                std::process::exit(1);
            }
        };

        let lines: Vec<String> = text.split("\n").map(|s| s.to_string()).collect();

        Self {
            file,
            text,
            lines,
            tokens: None,
            ast: None,
        }
    }

    pub fn from_text(text: String) -> Self {
        let lines: Vec<String> = text.split("\n").map(|s| s.to_string()).collect();

        Self {
            file: PathBuf::new(),
            text,
            lines,
            tokens: None,
            ast: None,
        }
    }

    /// Create an empty source for REPL sessions with a pseudo filename.
    pub fn repl(name: &str) -> Self {
        Self {
            file: PathBuf::from(name),
            text: String::new(),
            lines: Vec::new(),
            tokens: None,
            ast: None,
        }
    }

    /// Append a chunk of text to the session source, returning the starting line index.
    pub fn append_chunk(&mut self, chunk: &str) -> usize {
        let start_line = self.lines.len();
        if !self.text.is_empty() {
            self.text.push('\n');
        }
        self.text.push_str(chunk);
        self.lines.extend(chunk.split('\n').map(|s| s.to_string()));
        start_line
    }
}
