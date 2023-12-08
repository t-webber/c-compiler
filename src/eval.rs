use std::{collections::HashMap, slice::Iter};

use crate::parser::{parse_preprocessor, Bracing, Operator, PreprocessorToken};
use crate::preprocessor::MacroValue;

#[derive(Debug, Default, Clone, PartialEq)]
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
fn tokens_to_ast_impl(tokens: &mut Iter<'_, PreprocessorToken>, acc: PreprocessorAst, stop_at_block: bool) -> PreprocessorAst {
    if let Some(token) = tokens.next() {
        match token {
            PreprocessorToken::Operator(operator) => 
                match operator {
                    Operator::Plus|Operator::Minus|Operator::Not|Operator::Increment|Operator::Decrement
                    |Operator::AddAssign|Operator::SubAssign|Operator::MulAssign|Operator::DivAssign|Operator::ModAssign
                    |Operator::OrAssign|Operator::AndAssign|Operator::XorAssign|Operator::Arrow|Operator::ShiftLeftAssign
                    |Operator::ShiftRightAssign => {
                        if acc == PreprocessorAst::Empty {
                            let next = tokens_to_ast_impl(tokens, PreprocessorAst::Empty, true);
                            PreprocessorAst::UnaryOperator { operator: operator.clone(), child: Box::new(next) }
                        } else {
                          panic!("Expected only one argument for unary operator")
                      }},
                    _ => PreprocessorAst::BinaryOperator { operator: operator.clone(), left : Box::new(acc), right: Box::new(tokens_to_ast_impl(tokens, PreprocessorAst::Empty, false)) },
                }
            PreprocessorToken::Bracing(bracing) => match bracing {
                Bracing::LeftParenthesis => {let bidule = tokens_to_ast_impl(tokens, PreprocessorAst::Empty, false); tokens_to_ast_impl(tokens, bidule, false)},
                Bracing::RightParenthesis => acc,
                _ => tokens_to_ast_impl(tokens, PreprocessorAst::Leaf(token.to_owned()), false)
            }
            _ if stop_at_block => PreprocessorAst::Leaf(token.to_owned()),
            _ => tokens_to_ast_impl(tokens, PreprocessorAst::Leaf(token.to_owned()), false)
        }
    }
    else {
        acc
    }
}

pub fn tokens_to_ast(tokens: Vec<PreprocessorToken>) -> PreprocessorAst {
    tokens_to_ast_impl(&mut tokens.iter(), PreprocessorAst::Empty, false)
}

#[rustfmt::skip]
pub fn eval(ast: PreprocessorAst, macros: &mut HashMap<String, MacroValue>) -> i32 {
    match ast {
        PreprocessorAst::Empty => 0,
        PreprocessorAst::BinaryOperator { operator, left, right } => {let (x, y) = (eval(*left, macros), eval(*right, macros)); match operator {
            Operator::Plus|Operator::Minus|Operator::Not|Operator::Increment|Operator::Decrement
            | Operator::AddAssign|Operator::SubAssign|Operator::MulAssign|Operator::DivAssign|Operator::ModAssign
            | Operator::OrAssign|Operator::AndAssign|Operator::XorAssign|Operator::Arrow
            | Operator::ShiftLeftAssign|Operator::ShiftRightAssign => panic!("Unexpected error"),
            Operator::Add => x+y,
            Operator::Sub => x-y,
            Operator::Mul => x*y,
            Operator::Div => x/y,
            Operator::Mod => x%y,
            Operator::ShiftLeft => x<<y,
            Operator::ShiftRight => x<<y,
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
        PreprocessorAst::UnaryOperator { operator, child } => {
            let x = eval(*child, macros); match operator {
                Operator::Add|Operator::Sub|Operator::Mul|Operator::Div|Operator::Mod
                | Operator::BitwiseAnd|Operator::BitwiseOr|Operator::BitwiseXor
                | Operator::And|Operator::Or|Operator::Xor|Operator::NotEqual|Operator::Eequal
                | Operator::LessThan|Operator::GreaterThan|Operator::LessEqual
                | Operator::GreaterEqual|Operator::ShiftLeft|Operator::ShiftRight => panic!("Unexpected error"),
                Operator::Plus => x,
                Operator::Minus => -x,
                Operator::Not => if x==0 { 1 } else { 0 },
                Operator::Increment|Operator::Decrement|Operator::AddAssign|Operator::SubAssign
                | Operator::MulAssign|Operator::DivAssign|Operator::ModAssign|Operator::OrAssign
                | Operator::AndAssign|Operator::XorAssign|Operator::Arrow|Operator::ShiftLeftAssign
                | Operator::ShiftRightAssign => panic!("Unknown")
                }
        },
        PreprocessorAst::Leaf(leaf) => match leaf {
            PreprocessorToken::Macro(macro_name) => {
                let default = MacroValue::String(String::from("0"));
                let macro_value = macros.get(&macro_name).unwrap_or(&default);
                match macro_value {
                    MacroValue::String(macro_string) => eval(tokens_to_ast(parse_preprocessor(macro_string)), macros),
                    MacroValue::Function { args, body } => panic!("Macro with arguments are unsupported"),
                }
            },
            PreprocessorToken::LiteralNumber(x) => x.floor() as i32,
            PreprocessorToken::LiteralString(_) => panic!("No strings allowed"),
            _ => panic!("Not a valid leaf")
        },
    }
}
