use crate::eval;
use crate::parser::{Bracing, NonOpSymbol, PreprocessorToken};
use crate::structs::State;
use std::vec::IntoIter;

#[derive(Debug)]
pub enum FullAst {
    Empty,
    Node {
        first: Option<Box<FullAst>>,
        second: Option<Box<FullAst>>,
        third: Option<Box<FullAst>>,
    },
    Leaf(Vec<PreprocessorToken>),
}

trait ToLeaf {
    fn to_leaf(self) -> FullAst;
}

#[rustfmt::skip]
impl ToLeaf for FullAstElt {
    fn to_leaf(self) -> FullAst {
        match (self.open.unwrap_or(false), self.close) {
            (true, true) => { if self.elts[0] == PreprocessorToken::Bracing(Bracing::LeftParenthesis) && self.elts[self.elts.len() - 1] == PreprocessorToken::Bracing(Bracing::RightParenthesis) { FullAst::Leaf(self.elts[1..self.elts.len() - 1].to_vec()) } else { eprintln!("Bool set but parenthesis not found 1") ; FullAst::Leaf(self.elts)}},
            (true, false) => { if self.elts[0] == PreprocessorToken::Bracing(Bracing::LeftParenthesis) { FullAst::Leaf(self.elts[1..self.elts.len()].to_vec()) } else { eprintln!("Bool set but parenthesis not found 2"); FullAst::Leaf(self.elts)}},
            (false, true) => { if self.elts[self.elts.len() - 1] == PreprocessorToken::Bracing(Bracing::RightParenthesis) { FullAst::Leaf(self.elts[0..self.elts.len() - 1].to_vec()) } else { eprintln!("Bool set but parenthesis not found 3"); FullAst::Leaf(self.elts)}},
            (false, false) => FullAst::Leaf(self.elts),
        }
    }
}

#[derive(Debug, Default)]
struct FullAstElt {
    elts: Vec<PreprocessorToken>,
    open: Option<bool>,
    close: bool,
}

impl FullAstElt {
    fn push(&mut self, token: PreprocessorToken) {
        self.elts.push(token);
    }
}

#[rustfmt::skip]
impl From<Vec<PreprocessorToken>> for FullAstElt {
    fn from(iter: Vec<PreprocessorToken>) -> Self {
        let open = Some(iter[0] == PreprocessorToken::Bracing(Bracing::LeftParenthesis));
        let close = iter[iter.len() - 1] == PreprocessorToken::Bracing(Bracing::LeftParenthesis);
        Self {
            elts: iter,
            open,
            close
        }
    }
}

#[rustfmt::skip]
fn get_second_third(iter: &mut IntoIter<PreprocessorToken>) -> (FullAstElt, FullAstElt) {
    let mut second = FullAstElt::default();
    let mut question_level = 0;
    let mut parenthesis_level = 0;
    loop {
        let current = iter.next();
        if second.open.is_none() {second.open = Some(Some(PreprocessorToken::Bracing(Bracing::LeftParenthesis)) == current)};
        second.close = Some(PreprocessorToken::Bracing(Bracing::RightParenthesis)) == current;
    
        match current {
            Some(token @ PreprocessorToken::Bracing(Bracing::LeftParenthesis)) => {
                parenthesis_level += 1;
                second.push(token);
            }
            Some(token @ PreprocessorToken::Bracing(Bracing::RightParenthesis)) => {
                parenthesis_level -= 1;
                second.push(token);
            }
            Some(token) if parenthesis_level > 0 => second.push(token),
            Some(token @ PreprocessorToken::NonOpSymbol(NonOpSymbol::Interrogation)) => {
                question_level += 1;
                second.push(token);
            }
            Some(PreprocessorToken::NonOpSymbol(NonOpSymbol::Colon)) if question_level == 0 => break,
            Some(token @ PreprocessorToken::NonOpSymbol(NonOpSymbol::Colon)) => {
                question_level -= 1;
                second.push(token);
            }
            Some(token) => second.push(token),
            None => break,
        }
    }
    (
        second,
        FullAstElt::from(iter.collect::<Vec<PreprocessorToken>>()),
    )
}

#[rustfmt::skip]
fn vec2ternary_ast_impl(vec: Vec<PreprocessorToken>, close: bool) -> FullAst {
    let mut iter = vec.into_iter();
    let mut first = FullAstElt {
        elts: vec![],
        open: None,
        close,
    };
    let mut parenthesis_level = 0;
    loop {
        let current = iter.next();
        if first.open.is_none() {first.open = Some(Some(PreprocessorToken::Bracing(Bracing::LeftParenthesis)) == current)};
        
        match current {
            Some(token @ PreprocessorToken::Bracing(Bracing::LeftParenthesis)) => {
                parenthesis_level += 1;
                first.push(token);
                first.close = false;
            }
            Some(token @ PreprocessorToken::Bracing(Bracing::RightParenthesis)) => {
                parenthesis_level -= 1;
                first.push(token);
                first.close = true;
            }
            
            Some(PreprocessorToken::NonOpSymbol(NonOpSymbol::Interrogation)) => {
                let (second, third) = get_second_third(&mut iter);
                dbg!(&first, &second, &third);
                return FullAst::Node {
                    first: Some(Box::new(first.to_leaf())),
                    second: Some(Box::new(vec2ternary_ast_impl(second.elts, false))),
                    third: Some(Box::new(vec2ternary_ast_impl(third.elts, parenthesis_level > 0))),
                };
            }
            Some(token) => {first.push(token); first.close = true},
            None => return first.to_leaf(),
        }
    }
}

pub fn vec2ternary_ast(vec: Vec<PreprocessorToken>) -> FullAst {
    vec2ternary_ast_impl(vec, true)
}

#[rustfmt::skip]
pub fn eval_all(ast: &FullAst, state: &mut State) -> i32 {
    let empty: Box<FullAst> = Box::new(FullAst::Empty);
    match &ast {
        FullAst::Empty => 0,
        FullAst::Leaf(tokens) => {
            let ast = eval::tokens_to_ast(tokens, &mut state.current_position);
            dbg!(&tokens, &ast);
            eval::binary_ast_to_int(&ast, state)
        }
        FullAst::Node {first, second, third} => {
          let ifirst = eval_all(first.as_ref().unwrap_or_else(|| {eprintln!("first is empty"); &empty}), state);
          let isecond = eval_all(second.as_ref().unwrap_or_else(|| {eprintln!("second is empty"); &empty}), state);
          let ithird = eval_all(third.as_ref().unwrap_or_else(|| {eprintln!("third is empty"); &empty}), state);
          if ifirst == 0 {ithird} else {isecond}
        }
    }
}
