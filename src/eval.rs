use crate::arithmetic::CheckedOperations;
use crate::errors::{FailError, FilePosition, PreprocessorError, SystemError};
use crate::parser::{
    parse_preprocessor, Associativity, BinaryOperator, Operator, OperatorTrait, PreprocessorToken,
    UnaryOperator,
};
use crate::structs::{MacroValue, ParsingState};

#[allow(unused)]
#[derive(Debug, Default, PartialEq)]
pub enum PreprocessorAst {
    #[default]
    Empty,
    TernaryTree {
        left: Box<PreprocessorAst>,
        center: Box<PreprocessorAst>,
        right: Box<PreprocessorAst>,
    },
    BinaryTree {
        binary_operator: BinaryOperator,
        left: Box<PreprocessorAst>,
        right: Box<PreprocessorAst>,
    },
    UnaryTree {
        unary_operator: UnaryOperator,
        child: Box<PreprocessorAst>,
    },
    Leaf(PreprocessorToken),
}

struct CurrentTree<'tree> {
    tokens: &'tree Vec<PreprocessorToken>,
    index: &'tree mut usize,
    acc: PreprocessorAst,
    acc3: &'tree mut Box<PreprocessorAst>,
    previous_operator: Option<&'tree Operator>,
}

#[rustfmt::skip]
fn go_back_in_tokens(current_tree: CurrentTree, current_position: &FilePosition, current_operator: Option<&BinaryOperator>) -> PreprocessorAst {
    if current_tree.acc == PreprocessorAst::Empty {
        PreprocessorError::IncompleteOperator(&format!("{current_operator:?}")).fail_with_panic(current_position)
    } else {
        current_tree.index.checked_sub_assign_unwrap(1, current_position);
        current_tree.acc
    }
}

#[rustfmt::skip]
fn handle_binary(current_tree: CurrentTree, current_position: &mut FilePosition, current_operator: &BinaryOperator, previous_precedence: u32, parenthesis_level: &mut usize) -> PreprocessorAst {
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
            &mut Box::new(PreprocessorAst::Empty),
            Some(&Operator::Binary(current_operator.clone())),
            current_position,
            parenthesis_level
        );
        let binary_tree = PreprocessorAst::BinaryTree { 
            binary_operator: current_operator.clone(),
            left: Box::new(current_tree.acc),
            right: Box::new(right),
        };
        if *parenthesis_level < current_parenthesis_level {
            binary_tree
        } else {
            tokens_to_ast_impl_or_acc3(
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

// #[rustfmt::skip]
fn tokens_to_ast_impl_or_acc3(
    tokens: &Vec<PreprocessorToken>,
    index: &mut usize,
    acc: PreprocessorAst,
    acc3: &mut Box<PreprocessorAst>,
    previous_operator: Option<&Operator>,
    current_position: &mut FilePosition,
    parenthesis_level: &mut usize,
) -> PreprocessorAst {
    if **acc3 == PreprocessorAst::Empty {
        tokens_to_ast_impl(
            tokens,
            index,
            acc,
            acc3,
            previous_operator,
            current_position,
            parenthesis_level,
        )
    } else {
        let result = tokens_to_ast_impl(
            tokens,
            index,
            acc,
            acc3,
            previous_operator,
            current_position,
            parenthesis_level,
        );
        *acc3 = Box::new(result);
        // Will be thrown away anyway...
        PreprocessorAst::Empty
    }
}

fn handle_unary(
    current_tree: CurrentTree,
    current_position: &mut FilePosition,
    parenthesis_level: &mut usize,
    previous_precedence: u32,
    current_operator: &Operator,
    current_unary_operator: &UnaryOperator,
) -> PreprocessorAst {
    match current_unary_operator {
        UnaryOperator::Plus | UnaryOperator::Minus => {
            // False unary
            if current_tree.acc == PreprocessorAst::Empty {
                // In this case, we have for example
                // acc + -b
                let child = tokens_to_ast_impl(
                    current_tree.tokens,
                    current_tree.index,
                    current_tree.acc,
                    current_tree.acc3,
                    Some(current_operator),
                    current_position,
                    parenthesis_level,
                );
                PreprocessorAst::UnaryTree {
                    unary_operator: current_unary_operator.clone(),
                    child: Box::new(child),
                }
            } else {
                let binary_current_operator = match current_unary_operator {
                    UnaryOperator::Plus => BinaryOperator::Add,
                    UnaryOperator::Minus => BinaryOperator::Sub,
                    UnaryOperator::Not
                    | UnaryOperator::BitwiseNot
                    | UnaryOperator::Increment
                    | UnaryOperator::Decrement
                    | UnaryOperator::Defined => PreprocessorError::BinarySynthaxOnUnary(&format!(
                        "{current_unary_operator:?}"
                    ))
                    .fail_with_panic(current_position),
                };
                handle_binary(
                    current_tree,
                    current_position,
                    &binary_current_operator,
                    previous_precedence,
                    parenthesis_level,
                )
            }
        }
        UnaryOperator::Not | UnaryOperator::BitwiseNot | UnaryOperator::Defined => {
            if current_unary_operator.precedence() > previous_precedence {
                // In this case, the only possibilities are:
                //  defined !MACRO
                //  defined ~MACRO
                PreprocessorError::DefinedChildNotMacro.fail_with_panic(current_position)
            } else if current_tree.acc == PreprocessorAst::Empty {
                // Here acc is empty
                // We need to stop as soon as we reach an operator
                // which precedence is higher than ours

                // child contains the content of the unary operator
                let child = tokens_to_ast_impl(
                    current_tree.tokens,
                    current_tree.index,
                    PreprocessorAst::Empty,
                    &mut Box::new(PreprocessorAst::Empty),
                    Some(current_operator),
                    current_position,
                    parenthesis_level,
                );
                let unary_tree = PreprocessorAst::UnaryTree {
                    unary_operator: current_unary_operator.clone(),
                    child: Box::new(child),
                };
                // We use the precedence we received because of associativity
                tokens_to_ast_impl_or_acc3(
                    current_tree.tokens,
                    current_tree.index,
                    unary_tree,
                    current_tree.acc3,
                    current_tree.previous_operator,
                    current_position,
                    parenthesis_level,
                )
                // unary_operator
            } else {
                // We were in a situation like
                // a!b + ... : we read !
                PreprocessorError::IncompleteOperator(&format!("{current_unary_operator:?}"))
                    .fail_with_panic(current_position)
            }
        }
        UnaryOperator::Increment | UnaryOperator::Decrement => {
            PreprocessorError::InvalidOperator(&format!("{current_unary_operator:?}"))
                .fail_with_panic(current_position)
        }
    }
}

#[rustfmt::skip]
fn tokens_to_ast_impl(
    tokens: &Vec<PreprocessorToken>,
    index: &mut usize,
    acc: PreprocessorAst,
    acc3: &mut Box<PreprocessorAst>,
    previous_operator: Option<&Operator>,
    current_position: &mut FilePosition,
    parenthesis_level: &mut usize,
) -> PreprocessorAst {
    let previous_precedence =
        previous_operator.map_or_else(Operator::max_precedence, Operator::precedence);
        // eprintln!(">>>>>>>>>>>> IN = {acc:?}\t at index {index:?}");
        let result = {
        if let Some(token) = tokens.get(*index) {
            index.checked_add_assign_unwrap(1, current_position);
            match token {
                PreprocessorToken::Operator(operator) => match operator {
                    Operator::Unary(unary_operator) => handle_unary(
                        CurrentTree  { tokens, index, acc, acc3, previous_operator}, current_position, parenthesis_level, 
                        previous_precedence, operator, unary_operator,
                    ),
                    Operator::Binary(binary_operator @ (
                        BinaryOperator::BitwiseAnd  | BinaryOperator::BitwiseOr | BinaryOperator::BitwiseXor | BinaryOperator::And | BinaryOperator::Or
                        | BinaryOperator::ShiftLeft | BinaryOperator::ShiftRight
                        | BinaryOperator::NotEqual  | BinaryOperator::Eequal
                        | BinaryOperator::LessThan  | BinaryOperator::GreaterThan
                        | BinaryOperator::LessEqual | BinaryOperator::GreaterEqual 
                        | BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul | BinaryOperator::Div | BinaryOperator::Mod)) => 
                            handle_binary(
                                CurrentTree { tokens, index, acc, acc3, previous_operator },
                                current_position,
                                binary_operator,
                                previous_precedence,
                                parenthesis_level,
                            ),
                    Operator::Binary( 
                        BinaryOperator::AddAssign | BinaryOperator::SubAssign | BinaryOperator::MulAssign 
                        | BinaryOperator::DivAssign | BinaryOperator::ModAssign | BinaryOperator::OrAssign
                        | BinaryOperator::AndAssign | BinaryOperator::XorAssign | BinaryOperator::ShiftLeftAssign 
                        | BinaryOperator::ShiftRightAssign) => {
                            PreprocessorError::InvalidOperator(&format!("{operator:?}"))
                                .fail_with_panic(current_position);
                        }
                    Operator::Conditional => SystemError::CompilationError("Found ternary operator while building binary tree").fail_with_panic(current_position),
                },
                PreprocessorToken::Bracing(_) => 
                SystemError::CompilationError("Found ternary operator while building binary tree").fail_with_panic(current_position),
                    // match bracing {
                    //     Bracing::LeftParenthesis => {
                    //         parenthesis_level.checked_add_assign_unwrap(1, current_position);
                    //         tokens_to_ast_impl(
                    //             tokens,
                    //             index,
                    //             PreprocessorAst::Empty,
                    //             &mut Box::new(PreprocessorAst::Empty),
                    //             None,
                    //             current_position,
                    //             parenthesis_level,
                    //         )
                    //         // tokens_to_ast_impl_or_acc3(tokens, index, next, acc3, None, current_position, parenthesis_level)
                    //         // tokens_to_ast_impl(tokens, index, acc, acc3, None, current_position, parenthesis_level)
                    //     }
                    //     Bracing::RightParenthesis => {
                    //         // tokens_to_ast_impl(tokens, index, acc, acc3, None, current_position) // tout ce qu'il ne faut pas faire
                    //         *parenthesis_level = parenthesis_level.checked_sub(1).unwrap_or_else(|| {eprintln!("Overflow on subtract (this is totally normal, don't worry)"); 0});
                    //         acc
                    //         // tokens_to_ast_impl(tokens, index, acc, acc3, previous_operator, current_position, parenthesis_level)
                    //     }
                    //     Bracing::LeftBracket
                    //     | Bracing::RightBracket
                    //     | Bracing::LeftBrace
                    //     | Bracing::RightBrace => {
                    //         PreprocessorError::InvalidOperator(&format!("{bracing:?}"))
                    //             .fail_with_panic(current_position)
                    //     }
                    // }
                // }
                PreprocessorToken::NonOpSymbol(symbol) => {
                    PreprocessorError::InvalidOperator(&format!("Ternary ? {symbol:?}"))
                        .fail_with_panic(current_position)
                }
                PreprocessorToken::LiteralString(_)
                | PreprocessorToken::LiteralNumber(_)
                | PreprocessorToken::Macro(_) => {
                    let macro_leaf = PreprocessorAst::Leaf(token.clone());
                    tokens_to_ast_impl(
                        tokens,
                        index,
                        macro_leaf,
                        acc3,
                        previous_operator,
                        current_position,
                        parenthesis_level,
                    )
                }
            }
        } else {
            acc
        }
    };
    // println!(">>>>>>>>>>>> OUT = {result:?}");
    result
}

pub fn tokens_to_ast(
    tokens: &Vec<PreprocessorToken>,
    current_position: &mut FilePosition,
) -> PreprocessorAst {
    tokens_to_ast_impl(
        tokens,
        &mut 0,
        PreprocessorAst::Empty,
        &mut Box::new(PreprocessorAst::Empty),
        None,
        current_position,
        &mut 0,
    )
}

#[rustfmt::skip]
pub fn binary_ast_to_int(ast: &PreprocessorAst, state: &mut ParsingState) -> i32 {
    match ast {
        PreprocessorAst::Empty => 0,
        PreprocessorAst::TernaryTree { left, center, right } => {if binary_ast_to_int(left, state)!=0 { binary_ast_to_int(center, state) } else { binary_ast_to_int(right, state) }},
        
        PreprocessorAst::BinaryTree { binary_operator, left, right } => {let (x, y) = (binary_ast_to_int(left, state), binary_ast_to_int(right, state)); match binary_operator {
            BinaryOperator::Add => x.checked_add_unwrap(y, &state.current_position),
            BinaryOperator::Sub => x.checked_sub_unwrap(y, &state.current_position),
            BinaryOperator::Mul => x.checked_mul_unwrap(y, &state.current_position),
            BinaryOperator::Div => x.div_euclid(y),
            BinaryOperator::Mod => x.rem_euclid(y),
            BinaryOperator::ShiftLeft => x<<y,
            BinaryOperator::ShiftRight => x>>y,
            BinaryOperator::BitwiseAnd => x&y,
            BinaryOperator::BitwiseOr => x|y,
            BinaryOperator::BitwiseXor => x^y,
            BinaryOperator::And => i32::from((x!=0) && (y!=0)),
            BinaryOperator::Or => i32::from((x!=0) || (y!=0)),
            BinaryOperator::NotEqual => i32::from(x != y),
            BinaryOperator::Eequal => i32::from(x == y),
            BinaryOperator::LessThan => i32::from(x < y),
            BinaryOperator::GreaterThan => i32::from(x > y),
            BinaryOperator::LessEqual => i32::from(x <= y),
            BinaryOperator::GreaterEqual => i32::from(x >= y),
            BinaryOperator::AddAssign | 
            BinaryOperator::SubAssign | 
            BinaryOperator::MulAssign | 
            BinaryOperator::DivAssign | 
            BinaryOperator::ModAssign | 
            BinaryOperator::OrAssign | 
            BinaryOperator::AndAssign | 
            BinaryOperator::XorAssign | 
            BinaryOperator::ShiftLeftAssign | 
            BinaryOperator::ShiftRightAssign => PreprocessorError::InvalidOperator(&format!("{binary_operator:?}")).fail_with_panic(&state.current_position),
        }},
        PreprocessorAst::UnaryTree { unary_operator, child } => {
            let x = binary_ast_to_int(child, state); match unary_operator {
                UnaryOperator::Defined => 
                    if let PreprocessorAst::Leaf(macro_token) = child.as_ref() {
                        if let PreprocessorToken::Macro(macro_name) = &macro_token {
                            i32::from(state.defines.contains_key(macro_name))
                        } else {
                            PreprocessorError::DefinedChildNotMacro.fail_with_panic(&state.current_position)
                        }
                    } else {
                        PreprocessorError::DefinedChildNotLeaf.fail_with_panic(&state.current_position)
                    },
                UnaryOperator::Plus => x,
                UnaryOperator::Minus => x.checked_neg_unwrap(&state.current_position),
                UnaryOperator::Not => i32::from(x==0),
                UnaryOperator::BitwiseNot => !x,
                UnaryOperator::Increment | UnaryOperator::Decrement => PreprocessorError::InvalidOperator(&format!("{:?}", &unary_operator)).fail_with_panic(&state.current_position),
            }
        },
        PreprocessorAst::Leaf(leaf) => match leaf {
            PreprocessorToken::Macro(macro_name) => {
                let default = MacroValue::String(String::from("0"));
                let macro_value = state.defines.get(macro_name).unwrap_or(&default);
                // let macro_value = state.defines.get(macro_name).unwrap_or_else(|| panic!("{}", &compilation_error(&state.current_position, PreprocessorError::MacroNameNotFound("Manifestement on doit implémenter la ligne du dessus car les gens sont cons :)"))));
                match macro_value {
                    MacroValue::String(macro_string) => binary_ast_to_int(&tokens_to_ast(&parse_preprocessor(macro_string), &mut state.current_position), state),
                    MacroValue::Function { .. } => {PreprocessorError::InvalidLeaf(&format!("{leaf:?}")).fail_with_warning(&state.current_position); 0},
                }
            },
            #[allow(clippy::cast_possible_truncation)]
            PreprocessorToken::LiteralNumber(x) => *x,
            PreprocessorToken::LiteralString(_) => PreprocessorError::StringsNotAllowed.fail_with_panic(&state.current_position),
            PreprocessorToken::Operator(_) | PreprocessorToken::Bracing(_) | PreprocessorToken::NonOpSymbol(_) => PreprocessorError::InvalidLeaf(&format!("{leaf:?}")).fail_with_panic(&state.current_position),
        },
    }
}
