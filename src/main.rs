use crate::{
    lexer::{Lexer, token::*},
    parser::Parser,
};

pub mod lexer;
pub mod parser;

fn main() {
    let src = include_str!("script.qte").to_string();

    let mut lexer = Lexer::new("5 * (-2) ".into());
    let tokens = lexer.tokenize();

    dbg!(&tokens);

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    dbg!(&ast);
}
