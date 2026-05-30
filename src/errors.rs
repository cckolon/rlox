use std::fmt;

use crate::{ast::Literal, token::Token};

#[derive(Debug)]
pub enum LoxError {
    ScannerError {
        line: usize,
        character: char,
        message: String,
    },
    UnexpectedEndOfPhrase,
    SyntaxError {
        token: Token,
        message: String,
    },
    RuntimeError {
        token: Token,
        message: String,
    },
    // TODO: this should be a token
    ResolutionError(String),
    UndefinedVariable(String),
    InternalError(String),
    Return(Literal),
}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ScannerError {
                line,
                character,
                message,
            } => {
                write!(
                    f,
                    "Scanner error at line {line} on character {character}: {message}"
                )
            }
            Self::UnexpectedEndOfPhrase => write!(f, "Unexpected end of phrase"),
            Self::SyntaxError { token, message } => {
                write!(
                    f,
                    "Syntax error on line {} at token {}: {}",
                    token.line, token.lexeme, message
                )
            }
            Self::RuntimeError { token, message } => {
                let line = token.line;
                let lexeme = token.lexeme.clone();
                write!(
                    f,
                    "Runtime error on line {line} at token {lexeme}: {message}"
                )
            }
            Self::ResolutionError(message) => write!(f, "Resolution error: {message}"),
            Self::UndefinedVariable(name) => write!(f, "Undefined variable: {name}"),
            Self::InternalError(message) => write!(f, "Internal error: {message}"),
            Self::Return(value) => write!(f, "Meant to return: {value}"),
        }
    }
}
