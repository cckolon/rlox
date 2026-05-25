use std::fmt;

use crate::{ast::Literal, token::Token};

#[derive(Debug)]
pub enum LoxError {
    BadInputToken { line: usize, character: char },
    NoPreviousValue,
    UnterminatedString,
    UnexpectedEndOfPhrase,
    ParseFloatError,
    SyntaxError { token: Token, message: String },
    RuntimeError { token: Token, message: String },
    UndefinedVariable(String),
    InternalError(String),
    Return(Literal),
}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BadInputToken { line, character } => {
                write!(f, "Bad input token on line {line}: {character}")
            }
            Self::NoPreviousValue => write!(f, "No previous value"),
            Self::UnterminatedString => write!(f, "Unterminated string"),
            Self::UnexpectedEndOfPhrase => write!(f, "Unexpected end of phrase"),
            Self::ParseFloatError => write!(f, "Parse float error"),
            Self::SyntaxError { token, message } => {
                let line = token.line;
                let lexeme = token.lexeme.clone();
                write!(
                    f,
                    "Syntax error on line {line} at token {lexeme}: {message}"
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
            Self::UndefinedVariable(name) => write!(f, "Undefined variable: {name}"),
            Self::InternalError(message) => write!(f, "Internal error: {message}"),
            Self::Return(value) => write!(f, "Meant to return: {value}"),
        }
    }
}
