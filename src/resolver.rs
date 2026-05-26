use std::collections::HashMap;

use crate::{
    ast::{Expr, ExprKind, FunctionDeclaration, Stmt},
    errors::LoxError,
    interpreter::Interpreter,
};

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
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
                self.end_scope()?;
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
            Stmt::Return { keyword: _, value } => {
                if self.current_function == FunctionType::None {
                    return Err(LoxError::ResolutionError(
                        "Can't return from top level".to_string(),
                    ));
                }
                if let Some(expression) = value {
                    self.resolve_expression(expression)?;
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(body)?;
                Ok(())
            }
            Stmt::Var { name, initializer } => {
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
                Ok(())
            }
            ExprKind::Assign { name, value } => {
                self.resolve_expression(value)?;
                self.resolve_local(expression, name);
                Ok(())
            }
            ExprKind::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
                Ok(())
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
                Ok(())
            }
            ExprKind::Grouping { expression } => self.resolve_expression(expression),
            ExprKind::Literal(_value) => Ok(()),
            ExprKind::Unary { operator: _, right } => self.resolve_expression(right),
            ExprKind::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
                Ok(())
            }
        }
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
        self.end_scope()?;
        self.current_function = enclosing_function;
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) -> Result<(), LoxError> {
        match self.scopes.pop() {
            None => Err(LoxError::InternalError("No scopes left to pop".to_string())),
            Some(_scope) => Ok(()),
        }
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
}
