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
                    |Operator::ShiftRightAssign|Operator::Defined => {
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
                _ => tokens_to_ast_impl(tokens, PreprocessorAst::Leaf(token.clone()), false)
            }
            _ if stop_at_block => PreprocessorAst::Leaf(token.clone()),
            _ => tokens_to_ast_impl(tokens, PreprocessorAst::Leaf(token.clone()), false)
        }
    }
    else {
        acc
    }
}

pub fn tokens_to_ast(tokens: &[PreprocessorToken]) -> PreprocessorAst {
    tokens_to_ast_impl(&mut tokens.iter(), PreprocessorAst::Empty, false)
}

#[rustfmt::skip]
pub fn eval(ast: &PreprocessorAst, macros: &mut HashMap<String, MacroValue>) -> i32 {
    match ast {
        PreprocessorAst::Empty => 0,
        PreprocessorAst::BinaryOperator { operator, left, right } => {let (x, y) = (eval(left, macros), eval(right, macros)); match operator {
            Operator::Plus|Operator::Minus|Operator::Not|Operator::Increment|Operator::Decrement
            | Operator::AddAssign|Operator::SubAssign|Operator::MulAssign|Operator::DivAssign|Operator::ModAssign
            | Operator::OrAssign|Operator::AndAssign|Operator::XorAssign|Operator::Arrow
            | Operator::ShiftLeftAssign|Operator::ShiftRightAssign | Operator::Defined => panic!("Unexpected error"),
            Operator::Add => x+y,
            Operator::Sub => x-y,
            Operator::Mul => x*y,
            Operator::Div => x/y,
            Operator::Mod => x%y,
            Operator::ShiftLeft => x<<y,
            Operator::ShiftRight => x>>y,
            Operator::BitwiseAnd => x&y,
            Operator::BitwiseOr => x|y,
            Operator::BitwiseXor => x^y,
            Operator::And => i32::from((x!=0) && (y!=0)),
            Operator::Or => i32::from((x!=0) || (y!=0)),
            Operator::Xor => i32::from((x!=0) ^ (y!=0)),
            Operator::NotEqual => i32::from(x != y),
            Operator::Eequal => i32::from(x == y),
            Operator::LessThan => i32::from(x < y),
            Operator::GreaterThan => i32::from(x > y),
            Operator::LessEqual => i32::from(x <= y),
            Operator::GreaterEqual => i32::from(x >= y),
        }},
        PreprocessorAst::UnaryOperator { operator, child } => {
            let x = eval(child, macros); match operator {
                Operator::Add|Operator::Sub|Operator::Mul|Operator::Div|Operator::Mod
                | Operator::BitwiseAnd|Operator::BitwiseOr|Operator::BitwiseXor
                | Operator::And|Operator::Or|Operator::Xor|Operator::NotEqual|Operator::Eequal
                | Operator::LessThan|Operator::GreaterThan|Operator::LessEqual
                | Operator::GreaterEqual|Operator::ShiftLeft|Operator::ShiftRight => panic!("Unexpected error"),
                Operator::Defined => 
                    if let PreprocessorAst::Leaf(macro_token) = child.as_ref() {
                        if let PreprocessorToken::Macro(macro_name) = &macro_token {
                            i32::from(macros.contains_key(macro_name))
                        } else {
                            panic!("Defined child {:?} isn't a macro", &macro_token)
                        }
                    } else {
                        panic!("Expected a leaf as defined child got {:?}", child.as_ref())
                    }
                Operator::Plus => x,
                Operator::Minus => -x,
                Operator::Not => i32::from(x==0),
                Operator::Increment|Operator::Decrement|Operator::AddAssign|Operator::SubAssign
                | Operator::MulAssign|Operator::DivAssign|Operator::ModAssign|Operator::OrAssign
                | Operator::AndAssign|Operator::XorAssign|Operator::Arrow|Operator::ShiftLeftAssign
                | Operator::ShiftRightAssign => panic!("Unknown")
                }
        },
        PreprocessorAst::Leaf(leaf) => match leaf {
            PreprocessorToken::Macro(macro_name) => {
                let default = MacroValue::String(String::from("0"));
                let macro_value = macros.get(macro_name).unwrap_or(&default);
                match macro_value {
                    MacroValue::String(macro_string) => eval(&tokens_to_ast(&parse_preprocessor(macro_string)), macros),
                    MacroValue::Function { .. } => panic!("Macro with arguments are unsupported"),
                }
            },
            #[allow(clippy::cast_possible_truncation)]
            PreprocessorToken::LiteralNumber(x) => x.floor() as i32,
            PreprocessorToken::LiteralString(_) => panic!("No strings allowed"),
            _ => panic!("Not a valid leaf {leaf:?}")
        },
    }
}
