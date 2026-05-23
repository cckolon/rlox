use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{errors::LoxError, expr::Literal};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Literal>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn enclosed_by(enclosing: &Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
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
                None => Err(LoxError::UndefinedVariable { name: namestring }),
            }
        }
    }

    pub fn get(&self, name: &str) -> Result<Literal, LoxError> {
        match self.values.get(name) {
            Some(value) => Ok(value.clone()),
            None => match &self.enclosing {
                Some(environment) => environment.borrow().get(name),
                None => Err(LoxError::UndefinedVariable { name: name.into() }),
            },
        }
    }
}
