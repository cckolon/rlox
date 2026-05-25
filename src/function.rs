use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{Callable, FunctionDeclaration, Literal},
    environment::Environment,
    errors::LoxError,
    interpreter::Interpreter,
};

#[derive(Debug)]
pub struct LoxFunction {
    pub declaration: FunctionDeclaration,
    pub closure: Rc<RefCell<Environment>>,
}

impl Callable for LoxFunction {
    fn name(&self) -> &str {
        &self.declaration.name
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> Result<Literal, LoxError> {
        let environment = Environment::enclosed_by(&self.closure);
        self.declaration
            .params
            .iter()
            .zip(arguments.into_iter())
            .for_each(|(param_name, argument)| {
                environment.borrow_mut().define(param_name, argument)
            });
        let result = interpreter.execute_block(&self.declaration.body, environment);
        match result {
            Err(LoxError::Return(value)) => Ok(value),
            _ => Ok(Literal::Nil),
        }
    }
}
