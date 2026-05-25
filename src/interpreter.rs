use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{Expr, Literal, Stmt},
    environment::Environment,
    errors::LoxError,
    function::LoxFunction,
    native_functions::Clock,
    operator_type::{BinaryOpType, LogicalOpType, UnaryOpType},
};

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let environment = Environment::new();
        environment
            .borrow_mut()
            .define("clock", Literal::Callable(Rc::new(Clock {})));
        Interpreter {
            globals: environment.clone(),
            environment,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), LoxError> {
        statements
            .iter()
            .map(|stmt| self.execute_statement(stmt))
            .collect()
    }

    pub fn execute_statement(&mut self, stmt: &Stmt) -> Result<(), LoxError> {
        match stmt {
            Stmt::Expression(expression) => {
                self.evaluate_expression(expression)?;
            }
            Stmt::Print(expression) => {
                println!("{}", self.evaluate_expression(expression)?);
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate_expression(expr)?,
                    None => Literal::Nil,
                };
                self.environment.borrow_mut().define(name, value);
            }
            Stmt::Block(statements) => {
                let new_environment = Environment::enclosed_by(&self.environment);
                self.execute_block(statements, new_environment)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_value = self.evaluate_expression(condition)?;
                if is_truthy(&condition_value) {
                    self.execute_statement(then_branch)?
                } else if let Some(else_statement) = else_branch {
                    self.execute_statement(else_statement)?
                };
            }
            Stmt::While { condition, body } => loop {
                let condition_evaluated = self.evaluate_expression(condition)?;
                if !is_truthy(&condition_evaluated) {
                    break;
                };
                self.execute_statement(body)?;
            },
            Stmt::Function(function) => {
                self.environment.borrow_mut().define(
                    function.name.clone(),
                    Literal::Callable(Rc::new(LoxFunction {
                        // TODO: might want to RC the function instead
                        declaration: function.clone(),
                        closure: self.environment.clone(),
                    })),
                );
            }
            Stmt::Return { keyword, value } => {
                let evaluated_value = value
                    .as_ref()
                    .map(|expression| self.evaluate_expression(expression))
                    .transpose()?
                    .unwrap_or(Literal::Nil);
                return Err(LoxError::Return(evaluated_value));
            }
        };
        Ok(())
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), LoxError> {
        let previous_environment = std::mem::replace(&mut self.environment, environment);
        let result = self.execute_block_inner(statements);
        self.environment = previous_environment;
        result
    }

    fn execute_block_inner(&mut self, statements: &Vec<Stmt>) -> Result<(), LoxError> {
        for statement in statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    pub fn evaluate_expression(&mut self, expr: &Expr) -> Result<Literal, LoxError> {
        match expr {
            Expr::Assign { name, value } => {
                let value = self.evaluate_expression(value)?;
                self.environment.borrow_mut().assign(name, value)?;
                Ok(Literal::Nil)
            }
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Variable { name } => Ok(self.environment.borrow().get(name)?.clone()),
            Expr::Grouping { expression } => self.evaluate_expression(expression),
            Expr::Unary { right, operator } => {
                let right_value = self.evaluate_expression(right)?;
                match operator.op_type {
                    UnaryOpType::Negative => match right_value {
                        Literal::Number(value) => Ok(Literal::Number(-value)),
                        _ => Err(LoxError::SyntaxError {
                            token: operator.token.clone(),
                            message: "Cannot negate this value".to_string(),
                        }),
                    },
                    UnaryOpType::Not => Ok(Literal::Bool(!is_truthy(&right_value))),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate_expression(left)?;
                let right = self.evaluate_expression(right)?;
                match operator.op_type {
                    BinaryOpType::Subtract => match (left, right) {
                        (Literal::Number(left_num), Literal::Number(right_num)) => {
                            Ok(Literal::Number(left_num - right_num))
                        }
                        _ => Err(LoxError::SyntaxError {
                            token: operator.token.clone(),
                            message: "Cannot subtract non-numeric types.".to_string(),
                        }),
                    },
                    BinaryOpType::Divide => match (left, right) {
                        (Literal::Number(left_num), Literal::Number(right_num)) => {
                            if right_num == 0. {
                                Err(LoxError::RuntimeError {
                                    token: operator.token.clone(),
                                    message: "Cannot divide by zero.".to_string(),
                                })
                            } else {
                                Ok(Literal::Number(left_num / right_num))
                            }
                        }
                        _ => Err(LoxError::SyntaxError {
                            token: operator.token.clone(),
                            message: "Cannot divide non-numeric types".to_string(),
                        }),
                    },
                    BinaryOpType::Multiply => match (left, right) {
                        (Literal::Number(left_num), Literal::Number(right_num)) => {
                            Ok(Literal::Number(left_num * right_num))
                        }
                        _ => Err(LoxError::SyntaxError {
                            token: operator.token.clone(),
                            message: "Cannot multiply non-numeric types".to_string(),
                        }),
                    },
                    BinaryOpType::Add => match (left, right) {
                        (Literal::Number(left_num), Literal::Number(right_num)) => {
                            Ok(Literal::Number(left_num + right_num))
                        }
                        (Literal::String(left_string), Literal::String(right_string)) => {
                            Ok(Literal::String(format!("{}{}", left_string, right_string)))
                        }
                        _ => Err(LoxError::SyntaxError {
                            token: operator.token.clone(),
                            message: "Cannot add non-numeric or non-string types".to_string(),
                        }),
                    },
                    BinaryOpType::GreaterThan => match (left, right) {
                        (Literal::Number(left_num), Literal::Number(right_num)) => {
                            Ok(Literal::Bool(left_num > right_num))
                        }
                        _ => Err(LoxError::SyntaxError {
                            token: operator.token.clone(),
                            message: "Cannot compare non-numeric types".to_string(),
                        }),
                    },
                    BinaryOpType::GreaterThanEqualTo => match (left, right) {
                        (Literal::Number(left_num), Literal::Number(right_num)) => {
                            Ok(Literal::Bool(left_num >= right_num))
                        }
                        _ => Err(LoxError::SyntaxError {
                            token: operator.token.clone(),
                            message: "Cannot compare non-numeric types".to_string(),
                        }),
                    },
                    BinaryOpType::LessThan => match (left, right) {
                        (Literal::Number(left_num), Literal::Number(right_num)) => {
                            Ok(Literal::Bool(left_num < right_num))
                        }
                        _ => Err(LoxError::SyntaxError {
                            token: operator.token.clone(),
                            message: "Cannot compare non-numeric types".to_string(),
                        }),
                    },
                    BinaryOpType::LessThanEqualTo => match (left, right) {
                        (Literal::Number(left_num), Literal::Number(right_num)) => {
                            Ok(Literal::Bool(left_num <= right_num))
                        }
                        _ => Err(LoxError::SyntaxError {
                            token: operator.token.clone(),
                            message: "Cannot compare non-numeric types".to_string(),
                        }),
                    },
                    BinaryOpType::Equal => Ok(Literal::Bool(left == right)),
                    BinaryOpType::NotEqual => Ok(Literal::Bool(left != right)),
                }
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let evaluated_callee = self.evaluate_expression(callee)?;
                let argument_results: Result<Vec<Literal>, LoxError> = arguments
                    .iter()
                    .map(|expr| self.evaluate_expression(expr))
                    .collect();
                let arguments = argument_results?;
                match evaluated_callee {
                    Literal::Callable(function) => {
                        if function.arity() != arguments.len() {
                            Err(LoxError::RuntimeError {
                                token: paren.clone(),
                                message: format!(
                                    "Expected {} arguments, got {}",
                                    function.arity(),
                                    arguments.len()
                                ),
                            })
                        } else {
                            function.call(self, arguments)
                        }
                    }
                    _ => Err(LoxError::RuntimeError {
                        token: paren.clone(),
                        // TODO: should report the function type
                        message: "Expression is not callable".to_string(),
                    }),
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate_expression(left)?;
                let short_circuit = match operator.op_type {
                    LogicalOpType::Or => is_truthy(&left_value),
                    LogicalOpType::And => !is_truthy(&left_value),
                };
                if short_circuit {
                    Ok(left_value)
                } else {
                    self.evaluate_expression(right)
                }
            }
        }
    }
}

fn is_truthy(literal: &Literal) -> bool {
    match literal {
        Literal::Bool(value) => value.to_owned(),
        Literal::Nil => false,
        _ => true,
    }
}
