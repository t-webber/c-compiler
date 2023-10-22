
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

pub enum UnaryOperator {
    Minus,
    BitwiseNot,
    LogicalNot,
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement,
}

pub enum Expression {
    BinaryOp { operator : BinaryOperator, left : Box<Expression>, right : Box<Expression>},
    UnaryOp { operator : UnaryOperator, right : Box<Expression>},
    Literal { value : String },
} 