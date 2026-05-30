use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{
    ast::{Literal, LoxCallable},
    errors::LoxError,
    function::LoxFunction,
    interpreter::Interpreter,
    token::Token,
    token_type::TokenType::{self},
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn arity(&self) -> usize {
        let initializer = self.find_method("init");
        match initializer {
            Some(function) => function.arity(),
            None => 0,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        self.methods.get(name)
    }

    pub fn call(
        class: &Rc<Self>,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Literal>,
    ) -> Result<Literal, LoxError> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(class.clone())));
        if let Some(initializer) = class.find_method("init") {
            initializer
                .bind(&instance, true)
                .call(_interpreter, _arguments)?;
        }
        Ok(Literal::ClassInstance(instance))
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    class: Rc<LoxClass>,
    fields: HashMap<String, Literal>,
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self {
            class: class.clone(),
            fields: HashMap::new(),
        }
    }

    pub fn get(instance: &Rc<RefCell<LoxInstance>>, name: &Token) -> Result<Literal, LoxError> {
        let identifier = match &name.token_type {
            TokenType::Identifier(name) => name,
            _ => panic!(
                "Tried to evaluate non-identifier instance property. This shouldn't make it past the parser."
            ),
        };
        if let Some(value) = instance.borrow().fields.get(identifier) {
            return Ok(value.clone());
        }
        if let Some(method) = instance.borrow().class.find_method(identifier) {
            return Ok(Literal::Callable(LoxCallable::UserFunction(
                method.bind(instance, name.lexeme == "init").clone(),
            )));
        }
        return Err(LoxError::RuntimeError {
            token: name.clone(),
            message: format!("Property {identifier} not found"),
        });
    }

    pub fn set(&mut self, name: Token, value: Literal) {
        self.fields.insert(name.lexeme, value);
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
