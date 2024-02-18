use crate::errors;
use crate::errors::FilePosition;
use crate::eval;
use crate::parser::{NonOpSymbol, PreprocessorToken};
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

impl FullAst {
    fn push(
        &mut self,
        before: Vec<PreprocessorToken>,
        middle: Vec<PreprocessorToken>,
        after: Self,
    ) {
        match self {
            Self::Empty => {
                *self = Self::Node {
                    first: Some(Box::new(Self::Leaf(before))),
                    second: Some(Box::new(Self::Leaf(middle))),
                    third: Some(Box::new(after)),
                }
            }
            Self::Leaf(_) => panic!("Leaf should not be here"),
            Self::Node {
                first,
                second,
                third,
            } => match (first.is_none(), second.is_none(), third.is_none()) {
                (true, true, true) => {
                    *self = Self::Node {
                        first: Some(Box::new(Self::Leaf(before))),
                        second: Some(Box::new(Self::Leaf(middle))),
                        third: Some(Box::new(after)),
                    }
                }
                (true, _, _) => panic!("First is empty, but others aren't."),
                (false, true, true) => println!("false, true, true"),
                (false, true, false) => panic!("Second is empty, but third isn't."),
                (false, false, true) => println!("false, false, true"),
                (false, false, false) => println!("false, false, false"),
            },
        }
    }

    fn push_end(&mut self, end: Vec<PreprocessorToken>) {
        match self {
            Self::Empty => *self = Self::Leaf(end),
            Self::Leaf(_) => panic!("Leaf should not be here"),
            Self::Node {
                first,
                second,
                third,
            } => match (first.is_none(), second.is_none(), third.is_none()) {
                (true, true, true) => *self = Self::Leaf(end),
                (true, _, _) => panic!("First is empty, but others aren't."),
                (false, true, true) => println!("false, true, true"),
                (false, true, false) => panic!("Second is empty, but third isn't."),
                (false, false, true) => println!("false, false, true"),
                (false, false, false) => println!("false, false, false"),
            },
        }
    }
}

fn get_second_third(
    iter: &mut IntoIter<PreprocessorToken>,
) -> (Vec<PreprocessorToken>, Vec<PreprocessorToken>) {
    let mut second = vec![];
    let mut qlvl = 0;
    loop {
        let current = iter.next();
        match current {
            Some(question @ PreprocessorToken::NonOpSymbol(NonOpSymbol::Interrogation)) => {
                qlvl += 1;
                second.push(question);
            }
            Some(PreprocessorToken::NonOpSymbol(NonOpSymbol::Colon)) if qlvl == 0 => break,
            Some(colon @ PreprocessorToken::NonOpSymbol(NonOpSymbol::Colon)) => {
                qlvl -= 1;
                second.push(colon);
            }
            Some(c) => second.push(c),
            None => break,
        }
    }
    (second, iter.as_slice().to_vec())
}

pub fn vec2ternary_ast(vec: Vec<PreprocessorToken>) -> FullAst {
    let mut iter = vec.into_iter();
    let mut first = vec![];
    loop {
        match iter.next() {
            Some(PreprocessorToken::NonOpSymbol(NonOpSymbol::Interrogation)) => {
                let (second, third) = get_second_third(&mut iter);
                // dbg!(&first, &second, &third);
                return FullAst::Node {
                    first: Some(Box::new(FullAst::Leaf(first))),
                    second: Some(Box::new(vec2ternary_ast(second))),
                    third: Some(Box::new(vec2ternary_ast(third))),
                };
            }
            Some(c) => first.push(c),
            None => return FullAst::Leaf(first),
        }
    }
}

#[rustfmt::skip]
pub fn eval_all(ast: &FullAst, state: &mut State) -> i32 {
  let empty = Box::new(FullAst::Empty);
    match &ast {
        FullAst::Empty => 0,
        FullAst::Leaf(tokens) => eval::eval_one(
            &eval::tokens_to_ast(&tokens, &mut state.current_position),
            state,
        ),
        FullAst::Node {first, second, third} => {
          let ifirst = eval_all(first.as_ref().unwrap_or_else(|| {eprintln!("first is empty"); &empty}), state);
          let isecond = eval_all(second.as_ref().unwrap_or_else(|| {eprintln!("second is empty"); &empty}), state);
          let ithird = eval_all(third.as_ref().unwrap_or_else(|| {eprintln!("third is empty"); &empty}), state);
          if ifirst == 0 {ithird} else {isecond}
        }
    }
}
