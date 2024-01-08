#![allow(unused)]
use std::{path::PathBuf, collections::HashMap};

use parser::parse_preprocessor;

use crate::eval::{tokens_to_ast, eval};

// mod expression;
mod parser;
mod eval;
mod preprocessor;

fn main() {
    let preprocessed_file = preprocessor::preprocess_unit(PathBuf::from("fichier.c"));
    println!("{preprocessed_file}");
    // test_parser();
}

#[allow(unused)]
fn test_parser() {
    let input = String::from("MACRO1 MACRO2");
    let tokens = parse_preprocessor(&input);
    let ast = tokens_to_ast(tokens.clone());
    let result = eval(ast.clone(), &mut HashMap::default());
    println!("{tokens:?}");
    println!("{input:?}");
    println!("{ast:?}");
    println!("{result:?}");
}
