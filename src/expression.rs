#![allow(unused)]

#[derive(Debug)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Minus,
    BitwiseNot,
    LogicalNot,
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement,
}

#[derive(Debug)]
pub enum ExpressionTree {
    BinaryOp { operator : BinaryOperator, left : Box<ExpressionTree>, right : Box<ExpressionTree>},
    UnaryOp { operator : UnaryOperator, right : Box<ExpressionTree>},
    Literal { value : String },
}