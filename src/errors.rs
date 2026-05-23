use strum_macros::Display;

use crate::token::Token;

#[derive(Debug, Display)]
pub enum LoxError {
    BadInputToken,
    NoPreviousValue,
    UnterminatedString,
    UnexpectedEndOfPhrase,
    ParseFloatError,
    SyntaxError { token: Token, message: String },
    DivideByZeroError { token: Token },
    UndefinedVariable { name: String },
}
