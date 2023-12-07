use std::slice::Iter;

use crate::parser::{Bracing, Operator, PreprocessorToken};

#[derive(Debug, Default, Clone)]
pub enum PreprocessorAst {
    #[default]
    Empty,
    BinaryOperator {
        operator: Operator,
        left: Box<PreprocessorAst>,
        right: Box<PreprocessorAst>,
    },
    UnaryOperator {
        operator: Operator,
        child: Box<PreprocessorAst>,
    },
    Leaf(PreprocessorToken),
}

#[rustfmt::skip]
fn tokens_to_ast_impl(tokens: &mut Iter<'_, PreprocessorToken>, acc: PreprocessorAst) -> PreprocessorAst {
    if let Some(token) = tokens.next() {
        match token {
            PreprocessorToken::Operator(operator) => PreprocessorAst::BinaryOperator { operator: operator.clone(), left : Box::new(acc), right: Box::new(tokens_to_ast_impl(tokens, PreprocessorAst::Empty)) },
            PreprocessorToken::Bracing(bracing) => match bracing {
                Bracing::LeftParenthesis => {let bidule = tokens_to_ast_impl(tokens, PreprocessorAst::Empty); tokens_to_ast_impl(tokens, bidule)},
                Bracing::RightParenthesis => acc,
                _ => tokens_to_ast_impl(tokens, PreprocessorAst::Leaf(token.to_owned()))
            }
            _ => tokens_to_ast_impl(tokens, PreprocessorAst::Leaf(token.to_owned()))
        }
    }
    else {
        acc
    }
}

pub fn tokens_to_ast(tokens: Vec<PreprocessorToken>) -> PreprocessorAst {
    tokens_to_ast_impl(&mut tokens.iter(), PreprocessorAst::Empty)
}

#[rustfmt::skip]
pub fn eval(ast: PreprocessorAst) -> i32 {
    match ast {
        PreprocessorAst::Empty => 0,
        PreprocessorAst::BinaryOperator { operator, left, right } => {let (x, y) = (eval(*left), eval(*right)); match operator {
            Operator::Plus|Operator::Minus|Operator::Not|Operator::Increment|Operator::Decrement|Operator::ShiftLeft|
            Operator::ShiftRight|Operator::AddAssign|Operator::SubAssign|Operator::MulAssign|Operator::DivAssign|Operator::ModAssign|
            Operator::OrAssign|Operator::AndAssign|Operator::XorAssign|Operator::Arrow|Operator::ShiftLeftAssign|Operator::ShiftRightAssign => panic!("Unexpected error"),
            Operator::Add => x+y,
            Operator::Sub => x-y,
            Operator::Mul => x*y,
            Operator::Div => x/y,
            Operator::Mod => x%y,
            Operator::BitwiseAnd => x&y,
            Operator::BitwiseOr => x|y,
            Operator::BitwiseXor => x^y,
            Operator::And => if (x!=0) && (y!=0) {1} else {0},
            Operator::Or => if (x!=0) || (y!=0) {1} else {0},
            Operator::Xor => if (x!=0) ^ (y!=0) {1} else {0},
            Operator::NotEqual => if x != y {1} else {0},
            Operator::Eequal => if x == y {1} else {0},
            Operator::LessThan => if x < y {1} else {0},
            Operator::GreaterThan => if x > y {1} else {0},
            Operator::LessEqual => if x <= y {1} else {0},
            Operator::GreaterEqual => if x >= y {1} else {0},
        }},
        PreprocessorAst::UnaryOperator { operator, child } => todo!(),
        PreprocessorAst::Leaf(leaf) => match leaf {
            PreprocessorToken::Macro(_) => todo!(),
            PreprocessorToken::LiteralNumber(x) => x.floor() as i32,
            PreprocessorToken::LiteralString(_) => panic!("No strings allowed"),
            _ => panic!("Not a valid leaf")
        },
    }
}
