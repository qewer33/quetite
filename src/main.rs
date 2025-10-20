use crate::{
    evaluator::Evaluator,
    lexer::{Lexer, token::*},
    parser::Parser,
};

pub mod evaluator;
pub mod lexer;
pub mod parser;
pub mod reporter;

fn main() {
    let script = include_str!("script.qte").to_string();
    let test = include_str!("test.qte").to_string();

    // let mut lexer = Lexer::new("print \"hmm\" ".into());
    let mut lexer = Lexer::new(test);
    let tokens = lexer.tokenize();

    dbg!(&tokens);

    let mut parser = Parser::new(tokens);
    let src = parser.parse();

    dbg!(&src);

    let mut evaluator = Evaluator::new(src.unwrap());
    evaluator.eval();
}
