extern crate alloc;
use alloc::vec::IntoIter;
use core::mem::take;

use crate::errors::{FailError, FilePosition, PreprocessorError};
use crate::eval;
use crate::parser::{NonOpSymbol, PreprocessorToken};
use crate::structs::ParsingState;

fn push_second_and_third(iter: &mut IntoIter<PreprocessorToken>, middle_buffer: &mut Vec<PreprocessorToken>, current_position: &FilePosition) {
    for token in iter {
        match token {
            | PreprocessorToken::NonOpSymbol(NonOpSymbol::Colon) => return,
            | PreprocessorToken::NonOpSymbol(NonOpSymbol::Interrogation) => PreprocessorError::IncompleteOperator.fail_with_panic(current_position),
            | PreprocessorToken::Operator(_)
            | PreprocessorToken::Bracing(_)
            | PreprocessorToken::LiteralString(_)
            | PreprocessorToken::LiteralNumber(_)
            | PreprocessorToken::Macro(_) => middle_buffer.push(token),
        }
    }
}

fn eval_expression_impl(tokens: &mut IntoIter<PreprocessorToken>, state: &mut ParsingState) -> i32 {
    let mut no_ternary_vec = vec![];

    while let Some(token) = tokens.next() {
        match token {
            | PreprocessorToken::NonOpSymbol(NonOpSymbol::Interrogation) => {
                let first = take(&mut no_ternary_vec);
                let mut second = vec![];
                push_second_and_third(tokens, &mut second, &state.current_position);

                let res = if eval_expression(first, state) != 0_i32 {
                    eval_expression(second, state)
                } else {
                    eval_expression_impl(tokens, state)
                };

                no_ternary_vec.push(PreprocessorToken::LiteralNumber(res));
            },
            | PreprocessorToken::NonOpSymbol(NonOpSymbol::Colon) => PreprocessorError::IncompleteOperator.fail_with_panic(&state.current_position),
            | PreprocessorToken::Operator(_)
            | PreprocessorToken::Bracing(_)
            | PreprocessorToken::LiteralString(_)
            | PreprocessorToken::LiteralNumber(_)
            | PreprocessorToken::Macro(_) => no_ternary_vec.push(token),
        }
    }

    eval::binary_ast_to_int(&eval::tokens_to_ast(&no_ternary_vec, state), state)
}

pub fn eval_expression(tokens: Vec<PreprocessorToken>, state: &mut ParsingState) -> i32 {
    let mut iter: IntoIter<PreprocessorToken> = tokens.into_iter();

    eval_expression_impl(&mut iter, state)
}
