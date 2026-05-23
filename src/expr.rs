use crate::operator_type::{BinaryOp, LogicalOp, UnaryOp};
use strum_macros::Display;

#[derive(Clone, Debug, Display, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
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
}

#[derive(Debug)]
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
}
