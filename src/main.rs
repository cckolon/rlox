use std::env;
use std::fs;
use std::io;
use std::process;

use crate::errors::LoxError;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::scanner::Scanner;

mod environment;
mod errors;
mod expr;
mod interpreter;
mod operator_type;
mod parser;
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
                self.report_error(error);
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
                Err(e) => self.report_error(e),
            }
        }
    }

    fn report_error(&self, error: LoxError) {
        match error {
            LoxError::SyntaxError { token, message } => {
                let line = token.line;
                let lexeme = token.lexeme;
                println!("Syntax error at line {line} at token {lexeme}: {message}")
            }
            LoxError::DivideByZeroError { token } => {
                let line = token.line;
                let lexeme = token.lexeme;
                println!("Error at line {line} at token {lexeme}: Can't divide by zero.")
            }
            LoxError::UndefinedVariable { name } => {
                println!("Undefined variable: {name}")
            }
            _ => {
                println!("Error: {error}")
            }
        }
    }

    fn run(&mut self, source: String) -> Result<(), LoxError> {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        self.interpreter.interpret(statements)
    }
}
