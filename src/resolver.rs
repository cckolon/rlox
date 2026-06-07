use std::{collections::HashMap, mem};

use crate::{
    ast::{Expr, ExprKind, FunctionDeclaration, Stmt},
    errors::LoxError,
    interpreter::Interpreter,
    token::Token,
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
            Stmt::Class {
                name,
                token,
                methods,
                superclass,
            } => {
                let enclosing_class = mem::replace(&mut self.current_class, ClassType::Class);
                self.declare(name, token)?;
                self.define(name);
                if let Some(expr) = superclass {
                    match &expr.kind {
                        ExprKind::Variable {
                            token: _,
                            name: superclass_name,
                        } => {
                            if name == superclass_name {
                                return Err(LoxError::ResolutionError {
                                    token: token.clone(),
                                    message: format!("{name} cannot be a subclass of itself"),
                                });
                            }
                        }
                        _ => {
                            panic!(
                                "Caught during resolution: {name} superclass is not an identifier, this should have been caught in the parser"
                            )
                        }
                    }
                    self.current_class = ClassType::Subclass;
                    self.resolve_expression(expr)?;
                    self.begin_scope();
                    self.scopes
                        .last_mut()
                        .expect("began scope but scope not found in self.scopes")
                        .insert("super".to_string(), true);
                }
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
                if superclass.is_some() {
                    self.end_scope();
                }
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
            Stmt::Return { token, value } => {
                if self.current_function == FunctionType::None {
                    return Err(LoxError::ResolutionError {
                        token: token.clone(),
                        message: "Can't return from top level".to_string(),
                    });
                }
                if let Some(expression) = value {
                    if self.current_function == FunctionType::Initializer {
                        return Err(LoxError::ResolutionError {
                            token: token.clone(),
                            message: "Can't return a value from an initializer".to_string(),
                        });
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
                self.declare(name, token)?;
                if let Some(initializer_expression) = initializer {
                    self.resolve_expression(initializer_expression)?;
                }
                self.define(name);
                Ok(())
            }
            Stmt::Function { declaration } => {
                self.declare(&declaration.name, &declaration.token)?;
                self.define(&declaration.name);
                self.resolve_function(declaration, FunctionType::Function)?;
                Ok(())
            }
        }
    }

    fn resolve_expression(&mut self, expression: &Expr) -> Result<(), LoxError> {
        match &expression.kind {
            ExprKind::Variable { token, name } => {
                if let Some(scope) = self.scopes.last()
                    && scope.get(name) == Some(&false)
                {
                    return Err(LoxError::ResolutionError {
                        token: token.clone(),
                        message: format!(
                            "Can't read local variable {name} in its own initializer."
                        ),
                    });
                };
                self.resolve_local(expression, name);
            }
            ExprKind::Assign {
                name,
                token: _,
                value,
            } => {
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
                token: _,
                value,
            } => {
                self.resolve_expression(object)?;
                self.resolve_expression(value)?;
            }
            ExprKind::This { token } => {
                if self.current_class == ClassType::None {
                    return Err(LoxError::ResolutionError {
                        token: token.clone(),
                        message: "Can't use 'this' outside a class.".to_string(),
                    });
                }
                self.resolve_local(expression, token.lexeme.clone());
            }
            ExprKind::Super {
                keyword,
                method: _,
                method_name: _,
            } => match self.current_class {
                ClassType::None => {
                    return Err(LoxError::ResolutionError {
                        token: keyword.clone(),
                        message: "Cannot use 'super' outside a class".to_string(),
                    });
                }
                ClassType::Class => {
                    return Err(LoxError::ResolutionError {
                        token: keyword.clone(),
                        message: "Cannot use 'super' in a class that is not a subclass".to_string(),
                    });
                }
                ClassType::Subclass => self.resolve_local(expression, &keyword.lexeme),
            },
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
            self.declare(param, &function.token)?;
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

    fn declare(&mut self, name: impl Into<String>, token: &Token) -> Result<(), LoxError> {
        if let Some(scope) = self.scopes.last_mut() {
            let namestring = name.into();
            if scope.contains_key(&namestring) {
                return Err(LoxError::ResolutionError {
                    token: token.clone(),
                    message: format!("Already a variable with name {namestring} in this scope"),
                });
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
    Subclass,
}
