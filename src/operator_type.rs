use crate::token::Token;

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryOp {
    pub op_type: UnaryOpType,
    pub token: Token,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOpType {
    Negative,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryOp {
    pub op_type: BinaryOpType,
    pub token: Token,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOpType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogicalOp {
    pub op_type: LogicalOpType,
    pub token: Token,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogicalOpType {
    And,
    Or,
}
