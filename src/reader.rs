use crate::{
    errors::{FailError, PreprocessorError},
    parser::{Bracing, PreprocessorToken},
    structs::{MacroValue, ParsingState},
    ternary::{eval_all, vec2ternary_ast},
};

pub fn eval_tokens(tokens: &Vec<PreprocessorToken>, state: &mut ParsingState) -> i32 {
    let mut inindex = 0;
    match eval_between_parenthesis(tokens, &mut inindex, state) {
        PreprocessorToken::LiteralNumber(num) => num,
        PreprocessorToken::LiteralString(val) => i32::from(!val.is_empty()),
        PreprocessorToken::Macro(macro_name) => {
            state
                .defines
                .get(&macro_name)
                .map_or(0, |macro_value| match macro_value {
                    MacroValue::String(value) => value
                        .parse::<i32>()
                        .unwrap_or_else(|_| i32::try_from(value.len()).unwrap_or_default()),
                    MacroValue::Function { .. } => {
                        PreprocessorError::InvalidLeaf("functions are not yet implemented")
                            .fail_with_panic(&state.current_position)
                    }
                })
        }
        tok @ (PreprocessorToken::NonOpSymbol(_)
        | PreprocessorToken::Bracing(_)
        | PreprocessorToken::Operator(_)) => {
            PreprocessorError::InvalidLeaf(&format!("expected number, but got {tok:?}"))
                .fail_with_panic(&state.current_position)
        }
    }
}

fn eval_between_parenthesis(
    intokens: &Vec<PreprocessorToken>,
    inindex: &mut usize,
    state: &mut ParsingState,
) -> PreprocessorToken {
    let mut outtokens = Vec::<PreprocessorToken>::new();
    while let Some(token) = intokens.get(*inindex).as_ref() {
        *inindex += 1;
        match token {
            PreprocessorToken::Bracing(Bracing::LeftParenthesis) => {
                outtokens.push(eval_between_parenthesis(intokens, inindex, state));
            }
            PreprocessorToken::Bracing(Bracing::RightParenthesis) => break,
            _ => outtokens.push((*token).clone()),
        }
    }
    match outtokens.len() {
        0 => panic!("Empty parenthesis."),
        1 => return outtokens.first().unwrap().clone(),
        _ => {
            let tern = vec2ternary_ast(outtokens);
            // dbg!(&tern);
            PreprocessorToken::LiteralNumber(eval_all(&tern, state))
        }
    }
}
