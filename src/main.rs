use clap::Parser as ClapParser;
use std::{fs, path::PathBuf};

use crate::{
    evaluator::Evaluator,
    lexer::{token::*, Lexer},
    parser::Parser, src::Src,
};

pub mod evaluator;
pub mod lexer;
pub mod parser;
pub mod reporter;
pub mod src;

#[derive(ClapParser, Debug)]
#[command(name = "qte", about = "QTE interpreter", version, author)]
struct Args {
    /// Program file to run (e.g. script.qte)
    file: PathBuf,

    /// Dump token stream and exit
    #[arg(long, conflicts_with_all = ["dump_ast", "verbose"])]
    dump_tokens: bool,

    /// Dump AST and exit
    #[arg(long, conflicts_with_all = ["dump_tokens", "verbose"])]
    dump_ast: bool,

    /// Dump tokens and AST, then execute
    #[arg(long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // 1) Read source
    let mut src = Src::new(args.file);

    // 2) Lex
    let mut lexer = Lexer::new(src.text.clone());
    src.tokens = Some(lexer.tokenize());

    if args.dump_tokens || args.verbose {
        println!("== TOKENS ==");
        dbg!(&src.tokens);
        if args.dump_tokens {
            return; // only tokens requested
        }
    }

    // 3) Parse
    let mut parser = Parser::new(&src);
    let ast_opt = parser.parse();
    src.ast = match ast_opt {
        Some(s) => Some(s),
        None => {
            // Exit on parse error
            std::process::exit(1);
        }
    };

    if args.dump_ast || args.verbose {
        println!("== AST ==");
        dbg!(&src.ast);
        if args.dump_ast {
            return; // only tokens requested
        }
    }

    // 4) Execute
    let mut evaluator = Evaluator::new(&src);
    evaluator.eval();
}
