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
    ResolutionError {
        token: Token,
        message: String,
    },
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
                    token.line, token, message
                )
            }
            Self::RuntimeError { token, message } => {
                write!(
                    f,
                    "Runtime error on line {} at token {}: {}",
                    token.line, token, message
                )
            }
            Self::ResolutionError { token, message } => {
                write!(
                    f,
                    "Resolution error on line {} at token {}: {}",
                    token.line, token, message
                )
            }
            Self::InternalError(message) => write!(f, "Internal error: {message}"),
            Self::Return(value) => panic!("Uncaught return error. Meant to return {value}."),
        }
    }
}
