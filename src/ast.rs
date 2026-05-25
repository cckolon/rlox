use core::fmt;
use std::rc::Rc;

use crate::{
    errors::LoxError,
    interpreter::Interpreter,
    operator_type::{BinaryOp, LogicalOp, UnaryOp},
    token::Token,
};

#[derive(Clone, Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    // TODO: should this be a reference counter?
    // Maybe just an enum of all the different types of callable?
    Callable(Rc<dyn Callable>),
    Nil,
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::Number(a), Literal::Number(b)) => a == b,
            (Literal::String(a), Literal::String(b)) => a == b,
            (Literal::Bool(a), Literal::Bool(b)) => a == b,
            (Literal::Nil, Literal::Nil) => true,
            (Literal::Callable(a), Literal::Callable(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => {
                write!(f, "{}", value)
            }
            Self::Number(value) => {
                write!(f, "{}", value)
            }
            Self::String(value) => {
                write!(f, "{}", value)
            }
            Self::Nil => {
                write!(f, "nil")
            }
            Self::Callable(callable) => {
                write!(f, "{}", callable.name())
            }
        }
    }
}

pub trait Callable: fmt::Debug {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> Result<Literal, LoxError>;

    fn arity(&self) -> usize;

    fn name(&self) -> &str;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal(Literal),
    Logical {
        left: Box<Expr>,
        operator: LogicalOp,
        right: Box<Expr>,
    },
    Unary {
        operator: UnaryOp,
        right: Box<Expr>,
    },
    Variable {
        name: String,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Var {
        name: String,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Function(FunctionDeclaration),
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

pub enum FunctionKind {
    Function,
    Method,
}

impl fmt::Display for FunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            FunctionKind::Function => "function",
            FunctionKind::Method => "method",
        };
        write!(f, "{}", name)
    }
}
