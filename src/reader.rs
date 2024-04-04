use crate::{
    parser::{Bracing, PreprocessorToken},
    structs::State,
    ternary::{eval_all, vec2ternary_ast},
};

pub fn eval_tokens(tokens: &Vec<PreprocessorToken>, state: &mut State) -> i32 {
    let mut inindex = 0;
    match eval_between_parenthesis(&tokens, &mut inindex, state) {
        PreprocessorToken::LiteralNumber(num) => num,
        _ => panic!("Expected a number."),
    }
}

fn eval_between_parenthesis(
    intokens: &Vec<PreprocessorToken>,
    inindex: &mut usize,
    state: &mut State,
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
        1 => return outtokens.get(0).unwrap().clone(),
        _ => {
            let tern = vec2ternary_ast(outtokens);
            PreprocessorToken::LiteralNumber(eval_all(&tern, state))
        }
    }
}
