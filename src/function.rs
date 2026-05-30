use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    ast::{FunctionDeclaration, Literal},
    class::LoxInstance,
    environment::Environment,
    errors::LoxError,
    interpreter::Interpreter,
};

#[derive(Debug, Clone)]
pub struct LoxFunction {
    pub declaration: Rc<FunctionDeclaration>,
    // TODO: create a test for a circular reference and eliminate it with Weak<>
    // Like, is it a problem when the environment references a function which references the environment?
    pub closure: Rc<RefCell<Environment>>,
    pub is_initializer: bool,
}

impl LoxFunction {
    pub fn bind(&self, instance: &Rc<RefCell<LoxInstance>>, is_initializer: bool) -> Rc<Self> {
        let environment = Environment::enclosed_by(&self.closure);
        environment
            .borrow_mut()
            .define("this", Literal::ClassInstance(instance.clone()));
        Rc::new(Self {
            declaration: self.declaration.clone(),
            closure: environment,
            is_initializer,
        })
    }

    pub fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    pub fn call(
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
        let this = if self.is_initializer {
            Some(Environment::get_at(&self.closure, 0, "this")?)
        } else {
            None
        };
        match result {
            Err(LoxError::Return(value)) => Ok(this.unwrap_or(value)),
            Err(error) => Err(error),
            _ => Ok(this.unwrap_or(Literal::Nil)),
        }
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.declaration.name)
    }
}
