use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{ast::Literal, errors::LoxError};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Literal>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn enclosed_by(enclosing: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::default(),
            enclosing: Some(Rc::clone(enclosing)),
        }))
    }

    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::default(),
            enclosing: None,
        }))
    }

    pub fn define(&mut self, name: impl Into<String>, value: Literal) {
        let namestring = name.into();
        self.values.insert(namestring, value);
    }

    pub fn assign(&mut self, name: impl Into<String>, value: Literal) -> Result<(), LoxError> {
        let namestring = name.into();
        if self.values.contains_key(&namestring) {
            self.values.insert(namestring, value);
            Ok(())
        } else {
            match &self.enclosing {
                Some(environment) => {
                    environment.borrow_mut().assign(namestring, value)?;
                    Ok(())
                }
                None => Err(LoxError::UndefinedVariable(namestring)),
            }
        }
    }

    pub fn get(&self, name: &str) -> Result<Literal, LoxError> {
        match self.values.get(name) {
            Some(value) => Ok(value.clone()),
            None => match &self.enclosing {
                Some(environment) => environment.borrow().get(name),
                None => Err(LoxError::UndefinedVariable(name.into())),
            },
        }
    }

    pub fn get_at(
        env: &Rc<RefCell<Self>>,
        distance: usize,
        name: &str,
    ) -> Result<Literal, LoxError> {
        Ok(Self::ancestor(env, distance)?
            .borrow()
            .values
            .get(name)
            .ok_or(LoxError::InternalError(format!(
                "value matching {name} not found in environment at depth {distance}"
            )))?
            .clone())
    }

    pub fn assign_at(
        env: &Rc<RefCell<Self>>,
        distance: usize,
        name: &str,
        value: Literal,
    ) -> Result<(), LoxError> {
        Self::ancestor(env, distance)?
            .borrow_mut()
            .values
            .insert(name.into(), value);
        Ok(())
    }

    fn ancestor(env: &Rc<RefCell<Self>>, distance: usize) -> Result<Rc<RefCell<Self>>, LoxError> {
        let mut environment = Rc::clone(env);
        for _ in 0..distance {
            let next = environment
                .borrow()
                .enclosing
                .clone()
                .ok_or(LoxError::InternalError(
                    "No enclosing scope at this distance".to_string(),
                ))?;
            environment = next;
        }
        Ok(environment)
    }
}
