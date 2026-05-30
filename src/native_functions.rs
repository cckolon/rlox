use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{ast::Literal, errors::LoxError, interpreter::Interpreter};

#[derive(Debug, Clone)]
pub enum NativeFunction {
    Clock,
}

impl NativeFunction {
    // TODO: maybe put the trait back
    pub fn arity(&self) -> usize {
        match self {
            NativeFunction::Clock => 0,
        }
    }

    pub fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Literal>,
    ) -> Result<Literal, LoxError> {
        match self {
            NativeFunction::Clock => {
                let now = SystemTime::now();
                match now.duration_since(UNIX_EPOCH) {
                    Ok(value) => Ok(Literal::Number(value.as_secs() as f64)),
                    Err(_e) => Err(LoxError::InternalError(
                        "Unable to get system time".to_string(),
                    )),
                }
            }
        }
    }
}

impl fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NativeFunction::Clock => write!(f, "clock"),
        }
    }
}
