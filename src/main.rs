// use std::{fs, path::PathBuf};

use parser::parse_preprocessor;

use crate::eval::{tokens_to_ast, eval};

// mod expression;
mod parser;
mod eval;
// mod preprocessor;

fn main() {
    // let input = fs::read_to_string(PathBuf::from("fichier.c")).unwrap();
    let input = String::from("(2+(3-7))*5");
    let tokens = parse_preprocessor(&input);
    let ast = tokens_to_ast(tokens.clone());
    let result = eval(ast.clone());
    println!("{tokens:?}");
    println!("{input:?}");
    println!("{ast:?}");
    println!("{result:?}")
    // let preprocessed_file = preprocessor::preprocess_unit(PathBuf::from("fichier.c"));
    // println!("{preprocessed_file}");
}
