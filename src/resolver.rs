use std::{collections::HashMap, mem};

use crate::{
    ast::{Expr, ExprKind, FunctionDeclaration, Stmt},
    errors::LoxError,
    interpreter::Interpreter,
};

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_multiple_statements(&mut self, statements: &Vec<Stmt>) -> Result<(), LoxError> {
        for statement in statements.iter() {
            self.resolve_statement(statement)?;
        }
        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Stmt) -> Result<(), LoxError> {
        match statement {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_multiple_statements(statements)?;
                self.end_scope();
                Ok(())
            }
            Stmt::Class { name, methods } => {
                let enclosing_class = mem::replace(&mut self.current_class, ClassType::Class);
                self.declare(name)?;
                self.define(name);
                self.begin_scope();
                self.scopes
                    .last_mut()
                    .expect("Scopes were empty after beginning a new scope in class statement")
                    .insert("this".to_string(), true);

                for method in methods {
                    let function_type = if method.name == "init" {
                        FunctionType::Initializer
                    } else {
                        FunctionType::Method
                    };
                    self.resolve_function(method, function_type)?;
                }
                self.end_scope();
                self.current_class = enclosing_class;
                Ok(())
            }
            Stmt::Expression(expression) => self.resolve_expression(expression),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch)?;
                if let Some(else_expression) = else_branch {
                    self.resolve_statement(else_expression)?;
                }
                Ok(())
            }
            Stmt::Print(expression) => self.resolve_expression(expression),
            Stmt::Return { token: _, value } => {
                if self.current_function == FunctionType::None {
                    return Err(LoxError::ResolutionError(
                        "Can't return from top level".to_string(),
                    ));
                }
                if let Some(expression) = value {
                    if self.current_function == FunctionType::Initializer {
                        return Err(LoxError::ResolutionError(
                            "Can't return a value from an initializer".to_string(),
                        ));
                    }
                    self.resolve_expression(expression)?;
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;
                Ok(())
            }
            Stmt::Var {
                token,
                name,
                initializer,
            } => {
                self.declare(name)?;
                if let Some(initializer_expression) = initializer {
                    self.resolve_expression(initializer_expression)?;
                }
                self.define(name);
                Ok(())
            }
            Stmt::Function(declaration) => {
                self.declare(&declaration.name)?;
                self.define(&declaration.name);
                self.resolve_function(declaration, FunctionType::Function)?;
                Ok(())
            }
        }
    }

    fn resolve_expression(&mut self, expression: &Expr) -> Result<(), LoxError> {
        match &expression.kind {
            ExprKind::Variable { name } => {
                if let Some(scope) = self.scopes.last()
                    && scope.get(name) == Some(&false)
                {
                    return Err(LoxError::ResolutionError(format!(
                        "Can't read local variable {name} in its own initializer."
                    )));
                };
                self.resolve_local(expression, name);
            }
            ExprKind::Assign { name, value } => {
                self.resolve_expression(value)?;
                self.resolve_local(expression, name);
            }
            ExprKind::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            ExprKind::Call {
                callee,
                paren: _,
                arguments,
            } => {
                self.resolve_expression(callee)?;
                for argument in arguments {
                    self.resolve_expression(argument)?;
                }
            }
            ExprKind::Grouping { expression } => {
                self.resolve_expression(expression)?;
            }
            ExprKind::Literal(_value) => (),
            ExprKind::Unary { operator: _, right } => {
                self.resolve_expression(right)?;
            }
            ExprKind::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
            }
            ExprKind::Get { object, token: _ } => {
                self.resolve_expression(object)?;
            }
            ExprKind::Set {
                object,
                token,
                value,
            } => {
                self.resolve_expression(object)?;
                self.resolve_expression(value)?;
            }
            ExprKind::This { token } => {
                if self.current_class == ClassType::None {
                    return Err(LoxError::ResolutionError(
                        "Can't use 'this' outside a class.".to_string(),
                    ));
                }
                self.resolve_local(expression, token.lexeme.clone());
            }
        }
        Ok(())
    }

    fn resolve_function(
        &mut self,
        function: &FunctionDeclaration,
        function_type: FunctionType,
    ) -> Result<(), LoxError> {
        let enclosing_function = std::mem::replace(&mut self.current_function, function_type);
        self.begin_scope();
        for param in function.params.iter() {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_multiple_statements(&function.body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().expect("No scopes left to pop");
    }

    fn declare(&mut self, name: impl Into<String>) -> Result<(), LoxError> {
        if let Some(scope) = self.scopes.last_mut() {
            let namestring = name.into();
            if scope.contains_key(&namestring) {
                return Err(LoxError::ResolutionError(format!(
                    "Already a variable with name {namestring} in this scope"
                )));
            }
            scope.insert(namestring, false);
        }
        Ok(())
    }

    fn define(&mut self, name: impl Into<String>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.into(), true);
        }
    }

    fn resolve_local(&mut self, expression: &Expr, name: impl Into<String>) {
        let name_string = name.into();
        for (index, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name_string) {
                self.interpreter
                    .resolve(expression, self.scopes.len() - 1 - index);
                return;
            }
        }
    }
}

#[derive(PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(PartialEq)]
enum ClassType {
    None,
    Class,
}
