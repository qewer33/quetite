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
}
