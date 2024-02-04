use crate::parser::{
    parse_preprocessor, Associativity, Bracing, NonOpSymbol, Operator, PreprocessorToken,
};
use crate::preprocessor::{MacroValue, State};
use crate::tools::{FilePosition, PreprocessorError};

#[allow(unused)]
#[derive(Debug, Default, PartialEq)]
pub enum PreprocessorAst {
    #[default]
    Empty,
    TernaryOperator {
        operator: Operator,
        left: Box<PreprocessorAst>,
        center: Box<PreprocessorAst>,
        right: Box<PreprocessorAst>,
    },
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

struct CurrentTree<'a> {
    tokens: &'a Vec<PreprocessorToken>,
    index: &'a mut usize,
    acc: PreprocessorAst,
    acc3: &'a mut &'a PreprocessorAst,
    previous_operator: Option<&'a Operator>,
}

#[rustfmt::skip]
fn go_back_in_tokens(current_tree: CurrentTree, current_position: &mut FilePosition, current_operator: Option<&Operator>) -> PreprocessorAst {
    if current_tree.acc == PreprocessorAst::Empty {
        PreprocessorError::IncompleteOperator(&format!("{current_operator:?}")).fail_with_panic(current_position)
    } else {
        *current_tree.index -= 1;
        current_tree.acc
    }
}

#[rustfmt::skip]
fn handle_binary(current_tree: CurrentTree, current_position: &mut FilePosition, current_operator: &Operator, previous_precedence: u32, parenthesis_level: &mut usize) -> PreprocessorAst {
    let current_precedence = current_operator.precedence();
    let close_left = current_precedence>previous_precedence || (current_precedence==previous_precedence && current_operator.associativity() == Associativity::LeftToRight);
    if close_left {
        go_back_in_tokens(current_tree, current_position, Some(current_operator))
    } else {
        let current_parenthesis_level: usize = *parenthesis_level;
        let right = tokens_to_ast_impl(
            current_tree.tokens,
            current_tree.index,
            PreprocessorAst::Empty,
            &mut &PreprocessorAst::Empty,
            Some(current_operator),
            current_position,
            parenthesis_level
        );
        let binary_tree = PreprocessorAst::BinaryOperator {
            operator: current_operator.clone(),
            left: Box::new(current_tree.acc),
            right: Box::new(right),
        };
        if *parenthesis_level < current_parenthesis_level {
            binary_tree
        } else {
            tokens_to_ast_impl(
                current_tree.tokens,
                current_tree.index,
                binary_tree,
                current_tree.acc3,
                current_tree.previous_operator,
                current_position,
                parenthesis_level
            )
        }
    }
}

#[rustfmt::skip]
fn tokens_to_ast_impl_or_acc3( tokens: &Vec<PreprocessorToken>, index: &mut usize, acc: PreprocessorAst, acc3: &mut &PreprocessorAst, previous_operator: Option<&Operator>, current_position: &mut FilePosition, parenthesis_level: &mut usize,
) -> PreprocessorAst {
    if **acc3 == PreprocessorAst::Empty {
        tokens_to_ast_impl(tokens, index, acc, acc3, previous_operator, current_position, parenthesis_level)
    } else {
        let result = tokens_to_ast_impl(tokens, index, acc, acc3, previous_operator, current_position, parenthesis_level);
        *acc3 = &result;
        // Will be thrown away anyway...
        PreprocessorAst::Empty
    }
}

#[rustfmt::skip]
fn tokens_to_ast_impl(tokens: &Vec<PreprocessorToken>, index: &mut usize, acc: PreprocessorAst, acc3: &mut &PreprocessorAst, previous_operator: Option<&Operator>, current_position: &mut FilePosition, parenthesis_level: &mut usize) -> PreprocessorAst {
    eprintln!("A = {:?}", &acc);
    let previous_precedence = if let Some(previous_op) = previous_operator {
        previous_op.precedence()
    } else { Operator::max_precedence() };
    // let previous_associativity = if let Some(previous_op) = previous_operator {
    //     previous_op.associativity()
    // } else { Associativity::LeftToRight };
    if let Some(token) = tokens.get(*index) {
        *index += 1;
        match token {
            PreprocessorToken::Operator(operator) => match operator {
                // Invalid 
                Operator::Increment|Operator::Decrement|Operator::AddAssign|Operator::SubAssign|Operator::MulAssign
                | Operator::DivAssign|Operator::ModAssign|Operator::OrAssign
                | Operator::AndAssign|Operator::XorAssign|Operator::ShiftLeftAssign
                | Operator::ShiftRightAssign => PreprocessorError::InvalidOperator(&format!("{operator:?}")).fail_with_panic(current_position),
                // Unary
                Operator::Plus|Operator::Minus => {
                    // False unary
                    if acc == PreprocessorAst::Empty {
                    // In this case, we have for example
                    // acc + -b
                        let child = tokens_to_ast_impl(tokens, index, acc, acc3, Some(operator), current_position, parenthesis_level);
                        PreprocessorAst::UnaryOperator { operator: operator.clone(), child: Box::new(child) }
                    } else {
                        let current_operator = match operator {
                            Operator::Plus => Operator::Add, 
                            Operator::Minus => Operator::Sub,
                            _ => panic!("Catastrophic"),
                        };
                        handle_binary(CurrentTree { tokens, index, acc, acc3, previous_operator }, current_position, &current_operator, previous_precedence, parenthesis_level)
                    }
                },
                Operator::Not|Operator::BitwiseNot|Operator::Defined => {
                    if operator.precedence() > previous_precedence {
                        // In this case, the only possibilities are:
                        //  defined !MACRO
                        //  defined ~MACRO
                        PreprocessorError::DefinedChildNotMacro.fail_with_panic(current_position)
                    } else if acc == PreprocessorAst::Empty {
                        // Here acc is empty
                        // We need to stop as soon as we reach an operator
                        // which precedence is higher than ours
                        
                        // child contains the content of the unary operator
                        let child = tokens_to_ast_impl(tokens, index, PreprocessorAst::Empty, &mut &PreprocessorAst::Empty, Some(operator), current_position, parenthesis_level);
                        let unary_operator = PreprocessorAst::UnaryOperator { operator: operator.clone(), child: Box::new(child) };
                        // We use the precedence we received because of associativity
                        tokens_to_ast_impl(tokens, index, unary_operator, acc3, previous_operator, current_position, parenthesis_level)
                    } else {
                        // We were in a situation like
                        // a!b + ... : we read !
                        PreprocessorError::IncompleteOperator(&format!("{operator:?}")).fail_with_panic(current_position)
                    }
                }
                
                // Binary
                Operator::BitwiseAnd|Operator::BitwiseOr|Operator::BitwiseXor|Operator::And
                | Operator::Or|Operator::ShiftLeft|Operator::ShiftRight|Operator::NotEqual|Operator::Eequal|Operator::LessThan
                | Operator::GreaterThan|Operator::LessEqual|Operator::GreaterEqual
                | Operator::Add|Operator::Sub|Operator::Mul|Operator::Div|Operator::Mod => {
                    handle_binary(CurrentTree { tokens, index, acc, acc3, previous_operator}, current_position, operator, previous_precedence, parenthesis_level)
                },

                // Ternary
                Operator::Conditional => panic!("Not intented to be found in this scope"),
            },
            PreprocessorToken::Bracing(bracing) => match bracing {
                Bracing::LeftParenthesis => {
                    *parenthesis_level += 1;
                    let next = tokens_to_ast_impl(tokens, index, PreprocessorAst::Empty, &mut &PreprocessorAst::Empty, None, current_position, parenthesis_level);
                    tokens_to_ast_impl(tokens, index, next, acc3, None, current_position, parenthesis_level)
                    // tokens_to_ast_impl(tokens, index, acc, acc3, None, current_position, parenthesis_level)
                },
                Bracing::RightParenthesis => {
                    // tokens_to_ast_impl(tokens, index, acc, acc3, None, current_position) // tout ce qu'il ne faut pas faire
                    *parenthesis_level -= 1;
                    println!(">>>>>>>>>>>>>>>>>>>>>>>>> {acc:?}");
                    acc
                    // tokens_to_ast_impl(tokens, index, acc, acc3, previous_operator, current_position, parenthesis_level)
                },
                Bracing::LeftBracket | Bracing::RightBracket | Bracing::LeftBrace | Bracing::RightBrace => PreprocessorError::InvalidOperator(&format!("{bracing:?}")).fail_with_panic(current_position),
            },
            PreprocessorToken::NonOpSymbol(symbol) => match symbol {
                NonOpSymbol::Interrogation => {acc3 = &mut &acc; tokens_to_ast_impl(tokens, index, PreprocessorAst::Empty, &mut &PreprocessorAst::Empty, None, current_position, parenthesis_level)},
                NonOpSymbol::Colon => {
                    let current_parenthesis_level: usize = *parenthesis_level;
                    let right = tokens_to_ast_impl(tokens, index, PreprocessorAst::Empty, &mut &PreprocessorAst::Empty, None, current_position, parenthesis_level);
                    let ternary_tree = PreprocessorAst::TernaryOperator { operator: Operator::Conditional, left: Box::new(**acc3), center: Box::new(acc), right: Box::new(right) };
                    if *parenthesis_level < current_parenthesis_level {
                        ternary_tree
                   } else {
                        tokens_to_ast_impl(tokens, index, ternary_tree, &mut &PreprocessorAst::Empty, None, current_position, parenthesis_level)
                    }
                },
            },
            _ => { let macro_leaf = PreprocessorAst::Leaf(token.clone()); tokens_to_ast_impl_or_acc3(tokens, index, macro_leaf, acc3, previous_operator, current_position, parenthesis_level) },
        }
    } else {
        acc
    }
}

// #[rustfmt::skip]
// fn tokens_to_ast_impl(tokens: &mut Iter<'_, PreprocessorToken>, acc: PreprocessorAst, acc3: PreprocessorAst, precedence: u32, current_position: &FilePosition) -> PreprocessorAst {
//     if let Some(token) = tokens.next() {
//         match token {
//             PreprocessorToken::NonOpSymbol(NonOpSymbol::Interrogation) => {
//                 tokens_to_ast_impl(tokens, PreprocessorAst::Empty, acc, Operator::Conditional.precedence(), current_position)

//                 // let current_precedence = Operator::Conditional.precedence();
//                 // let left = acc;
//                 // let center = tokens_to_ast_impl(tokens, PreprocessorAst::Empty, current_precedence, current_position);
//                 // let right = tokens_to_ast_impl(tokens, PreprocessorAst::Empty, current_precedence, current_position);
//                 // let ternary = PreprocessorAst::TernaryOperator { operator: Operator::Conditional, left: Box::new(left), center: Box::new(center), right: Box::new(right) };
//                 // if current_precedence > precedence {
//                 //     ternary
//                 // } else {
//                 //     tokens_to_ast_impl(tokens, ternary, Operator::max_precedence(), current_position)
//                 // }
//             },
//             PreprocessorToken::NonOpSymbol(NonOpSymbol::Colon) => {
//                 PreprocessorAst::TernaryOperator { operator: Operator::Conditional, left: Box::new(acc3), center: Box::new(acc),
//                     right: Box::new(tokens_to_ast_impl(tokens, PreprocessorAst::Empty, PreprocessorAst::Empty, Operator::Conditional.precedence(), current_position)) }
//             },
//             PreprocessorToken::Operator(operator) =>
//                 match operator {
//                     Operator::Add|Operator::Sub|Operator::Not|Operator::Increment|Operator::Decrement
//                     |Operator::AddAssign|Operator::SubAssign|Operator::MulAssign|Operator::DivAssign|Operator::ModAssign
//                     |Operator::OrAssign|Operator::AndAssign|Operator::XorAssign|Operator::ShiftLeftAssign
//                     |Operator::ShiftRightAssign|Operator::Defined|Operator::Plus|Operator::Minus => {
//                         if acc == PreprocessorAst::Empty {
//                             let current_precedence = operator.precedence();
//                             let under_unary = tokens_to_ast_impl(tokens, PreprocessorAst::Empty, PreprocessorAst::Empty, current_precedence, current_position);
//                             let unary = PreprocessorAst::UnaryOperator { operator: operator.clone(), child: Box::new(under_unary) };
//                             if current_precedence > precedence {
//                                 unary
//                             } else {
//                                 tokens_to_ast_impl(tokens, unary, acc3, Operator::max_precedence(), current_position)
//                             }
//                             // tokens_to_ast_impl(tokens, unary, false)
//                         } else if (operator == &Operator::Add) || (operator == &Operator::Sub) {
//                             PreprocessorAst::BinaryOperator { operator: operator.clone(), left: Box::new(acc), right: Box::new(tokens_to_ast_impl(tokens, PreprocessorAst::Empty, acc3, operator.precedence(), current_position)) }
//                         } else {
//                             panic!("{}", compilation_error(current_position, "Expected only one argument for unary operator"))
//                         }
//                       },
//                     _ => PreprocessorAst::BinaryOperator { operator: operator.clone(), left : Box::new(acc), right: Box::new(tokens_to_ast_impl(tokens, PreprocessorAst::Empty, acc3, operator.precedence(), current_position)) },
//                 }
//             PreprocessorToken::Bracing(bracing) => match bracing {
//                 Bracing::LeftParenthesis => {let next = tokens_to_ast_impl(tokens, PreprocessorAst::Empty, PreprocessorAst::Empty, precedence, current_position); tokens_to_ast_impl(tokens, next, acc3, Operator::max_precedence(), current_position)},
//                 Bracing::RightParenthesis => acc,
//                 _ => tokens_to_ast_impl(tokens, PreprocessorAst::Leaf(token.clone()), acc3, precedence, current_position)
//             }
//             _ if precedence < Operator::max_precedence() => PreprocessorAst::Leaf(token.clone()),
//             _ => tokens_to_ast_impl(tokens, PreprocessorAst::Leaf(token.clone()), acc3, Operator::max_precedence(), current_position)
//         }
//     }
//     else {
//         acc
//     }
// }

#[rustfmt::skip]
pub fn tokens_to_ast(tokens: &mut Vec<PreprocessorToken>, current_position: &mut FilePosition) -> PreprocessorAst {
    tokens_to_ast_impl(tokens, &mut 0, PreprocessorAst::Empty, &mut &PreprocessorAst::Empty, None, current_position, &mut 0)
}

#[rustfmt::skip]
pub fn eval(ast: &PreprocessorAst, state: &mut State) -> i32 {
    match ast {
        PreprocessorAst::Empty => 0,
        PreprocessorAst::TernaryOperator { operator, left, center, right } => match operator {
            Operator::Conditional => if eval(left, state)!=0 { eval(center, state) } else { eval(right, state) },
            _ => panic!("Read a unary or binary operator as a ternary operato")
        },
        PreprocessorAst::BinaryOperator { operator, left, right } => {let (x, y) = (eval(left, state), eval(right, state)); match operator {
            Operator::Plus|Operator::Minus|Operator::Not|Operator::AddAssign|Operator::SubAssign
            | Operator::MulAssign|Operator::DivAssign|Operator::ModAssign|Operator::OrAssign
            | Operator::AndAssign|Operator::XorAssign|Operator::ShiftLeftAssign|Operator::ShiftRightAssign
            | Operator::Defined|Operator::Conditional|Operator::BitwiseNot|Operator::Increment|Operator::Decrement => panic!("Read a unary or ternary operator as a binary operator node"),
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
            Operator::NotEqual => i32::from(x != y),
            Operator::Eequal => i32::from(x == y),
            Operator::LessThan => i32::from(x < y),
            Operator::GreaterThan => i32::from(x > y),
            Operator::LessEqual => i32::from(x <= y),
            Operator::GreaterEqual => i32::from(x >= y),
        }},
        PreprocessorAst::UnaryOperator { operator, child } => {
            let x = eval(child, state); match operator {
                Operator::Add|Operator::Sub|Operator::Mul|Operator::Div|Operator::Mod
                | Operator::BitwiseAnd|Operator::BitwiseOr|Operator::BitwiseXor
                | Operator::And|Operator::Or|Operator::NotEqual|Operator::Eequal
                | Operator::LessThan|Operator::GreaterThan|Operator::LessEqual
                | Operator::GreaterEqual|Operator::ShiftLeft|Operator::ShiftRight|Operator::Conditional => panic!("Read a binary or ternary operator as a unary operator node"),
                Operator::Defined => 
                    if let PreprocessorAst::Leaf(macro_token) = child.as_ref() {
                        if let PreprocessorToken::Macro(macro_name) = &macro_token {
                            i32::from(state.defines.contains_key(macro_name))
                        } else {
                            panic!("{}", PreprocessorError::DefinedChildNotMacro.fail(&state.current_position))
                        }
                    } else {
                        panic!("{}", PreprocessorError::DefinedChildNotLeaf.fail(&state.current_position))
                    }
                Operator::Plus => x,
                Operator::Minus => -x,
                Operator::Not => i32::from(x==0),
                Operator::BitwiseNot => !x,
                Operator::Increment|Operator::Decrement|Operator::AddAssign|Operator::SubAssign|Operator::MulAssign
                | Operator::DivAssign|Operator::ModAssign|Operator::OrAssign
                | Operator::AndAssign|Operator::XorAssign|Operator::ShiftLeftAssign
                | Operator::ShiftRightAssign => panic!("{}", PreprocessorError::InvalidOperator(&format!("{:?}", &operator)).fail(&state.current_position)),
            }
        },
        PreprocessorAst::Leaf(leaf) => match leaf {
            PreprocessorToken::Macro(macro_name) => {
                let default = MacroValue::String(String::from("0"));
                let macro_value = state.defines.get(macro_name).unwrap_or(&default);
                // let macro_value = state.defines.get(macro_name).unwrap_or_else(|| panic!("{}", &compilation_error(&state.current_position, PreprocessorError::MacroNameNotFound("Manifestement on doit implémenter la ligne du dessus car les gens sont cons :)"))));
                match macro_value {
                    MacroValue::String(macro_string) => eval(&tokens_to_ast(&mut parse_preprocessor(macro_string), &mut state.current_position), state),
                    MacroValue::Function { .. } => todo!(),
                }
            },
            #[allow(clippy::cast_possible_truncation)]
            PreprocessorToken::LiteralNumber(x) => x.floor() as i32,
            PreprocessorToken::LiteralString(_) => panic!("{}", PreprocessorError::StringsNotAllowed.fail(&state.current_position)),
            _ => panic!("{}", PreprocessorError::InvalidLeaf(&format!("{leaf:?}")).fail(&state.current_position)),
        },
    }
}
