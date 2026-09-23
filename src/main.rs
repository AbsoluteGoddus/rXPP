// TODO: Replace panic!(); statements with actual error handling, as well as continuation of parsing, as to provide a longer error context.

mod lexer;
mod parser;

use colored::Colorize;
use std::fs;
use std::mem::discriminant;
use std::num::{ParseFloatError, ParseIntError};

use lexer::Lexer;
use lexer::SpannedToken;
use lexer::Token;
use lexer::LexerError;

use parser::Parser;

fn read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", path, e))
}

fn main() {
    let source = read_file("main.xpp");
    if let Err(msg) = &source {
        println!("[{}] {}", "ERROR".bold().red(), msg.bold().red())
    }
    let source = source.unwrap();

    let mut lexer: Lexer = Lexer::new();
    lexer.source = source;
    let tokens = lexer.tokenize();

    println!("[{}] tokens: {:#?}", "DEBUG".green(), tokens);

    let mut parser: Parser = Parser::new(tokens);

    let asts = parser.parse();
    println!("[{}] Vec<ASTNode>: {:#?}", "DEBUG".green(), asts);
}
