use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    ast::{Callable, Literal},
    errors::LoxError,
    interpreter::Interpreter,
};

#[derive(Debug)]
pub struct Clock {}

impl Callable for Clock {
    fn name(&self) -> &str {
        "clock"
    }

    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Literal>,
    ) -> Result<Literal, LoxError> {
        let now = SystemTime::now();
        match now.duration_since(UNIX_EPOCH) {
            Ok(value) => Ok(Literal::Number(value.as_secs() as f64)),
            Err(_e) => Err(LoxError::InternalError(
                "Unable to get system time".to_string(),
            )),
        }
    }
}
