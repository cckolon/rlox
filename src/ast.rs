use core::fmt;
use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::{
    class::{LoxClass, LoxInstance},
    errors::LoxError,
    function::LoxFunction,
    interpreter::Interpreter,
    native_functions::NativeFunction,
    operator_type::{BinaryOp, LogicalOp, UnaryOp},
    token::Token,
};

#[derive(Clone, Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    // TODO: should I split out here?
    Callable(LoxCallable),
    ClassInstance(Rc<RefCell<LoxInstance>>),
    Nil,
}

#[derive(Clone, Debug)]
pub enum LoxCallable {
    NativeFunction(Rc<NativeFunction>),
    UserFunction(Rc<LoxFunction>),
    Class(Rc<LoxClass>),
}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::NativeFunction(function) => function.arity(),
            LoxCallable::UserFunction(function) => function.arity(),
            LoxCallable::Class(class) => class.arity(),
        }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> Result<Literal, LoxError> {
        match self {
            LoxCallable::Class(class) => LoxClass::call(class, interpreter, arguments),
            LoxCallable::UserFunction(function) => function.call(interpreter, arguments),
            LoxCallable::NativeFunction(function) => function.call(interpreter, arguments),
        }
    }
}

impl fmt::Display for LoxCallable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxCallable::UserFunction(function) => write!(f, "{}", function),
            LoxCallable::Class(class) => write!(f, "{}", class),
            LoxCallable::NativeFunction(function) => write!(f, "{}", function),
        }
    }
}

impl PartialEq for LoxCallable {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LoxCallable::UserFunction(a), LoxCallable::UserFunction(b)) => Rc::ptr_eq(a, b),
            (LoxCallable::Class(a), LoxCallable::Class(b)) => Rc::ptr_eq(a, b),
            (LoxCallable::NativeFunction(a), LoxCallable::NativeFunction(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::Number(a), Literal::Number(b)) => a == b,
            (Literal::String(a), Literal::String(b)) => a == b,
            (Literal::Bool(a), Literal::Bool(b)) => a == b,
            (Literal::Nil, Literal::Nil) => true,
            (Literal::Callable(a), Literal::Callable(b)) => a == b,
            (Literal::ClassInstance(a), Literal::ClassInstance(b)) => Rc::ptr_eq(a, b),
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
                write!(f, "{}", callable)
            }
            Self::ClassInstance(instance) => {
                write!(f, "{}", instance.borrow())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub id: usize,
    pub kind: ExprKind,
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Expr {}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
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
    Get {
        object: Box<Expr>,
        token: Token,
    },
    Set {
        object: Box<Expr>,
        token: Token,
        value: Box<Expr>,
    },
    This {
        token: Token,
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
        token: Token,
        name: String,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Function(Rc<FunctionDeclaration>),
    Return {
        token: Token,
        value: Option<Expr>,
    },
    Class {
        name: String,
        methods: Vec<FunctionDeclaration>,
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
