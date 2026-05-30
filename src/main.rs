use std::env;
use std::fs;
use std::io;
use std::process;

use crate::errors::LoxError;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::Scanner;

mod ast;
mod class;
mod environment;
mod errors;
mod function;
mod interpreter;
mod native_functions;
mod operator_type;
mod parser;
mod resolver;
mod scanner;
mod token;
mod token_type;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();

    match args.len() {
        1 => lox.run_prompt(),
        2 => lox.run_file(&args[1]),
        _ => {
            eprintln!("Usage: rlox [script]");
            process::exit(64);
        }
    }
}

pub struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    fn new() -> Self {
        Lox {
            interpreter: Interpreter::new(),
        }
    }

    fn run_file(&mut self, path: &str) {
        let source: String = fs::read_to_string(path).expect("could not read file");
        let result = self.run(source);
        match result {
            Ok(()) => {}
            Err(error) => {
                println!("{}", error);
                process::exit(65);
            }
        }
    }

    fn run_prompt(&mut self) {
        loop {
            let mut line = String::new();
            let line_count = io::stdin()
                .read_line(&mut line)
                .expect("Failed to read input");
            if line_count == 0 {
                break;
            }
            let result = self.run(line);
            match result {
                Ok(()) => {}
                Err(e) => println!("{}", e),
            }
        }
    }

    fn run(&mut self, source: String) -> Result<(), LoxError> {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let mut resolver = Resolver::new(&mut self.interpreter);
        resolver.resolve_multiple_statements(&statements)?;
        self.interpreter.interpret(statements)
    }
}
