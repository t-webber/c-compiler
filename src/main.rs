// #![allow(unused)]
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
mod preprocessor;
mod tools;

// mod expression;
mod parser;
use parser::parse_preprocessor;
mod eval;
use crate::eval::{eval, tokens_to_ast};
use preprocessor::State;

fn main() -> std::io::Result<()> {
    run_main("fichier.c")
    // run_main("/usr/lib/gcc/x86_64-linux-gnu/12/include/stddef.h")
    // test_parser("MACRO1 MACRO2")
}

#[allow(unused)]
fn run_main(path: &str) -> std::io::Result<()> {
    let preprocessed_file = preprocessor::preprocess_unit(PathBuf::from(path));
    println!("{preprocessed_file}");
    let mut data: &mut [u8] = &mut [0; 32];
    let mut file = File::create(format!("{}i", &path[0..path.len() - 1]))?;
    file.write_all(preprocessed_file.as_bytes())?;
    Ok(())
}

#[allow(unused)]
fn test_parser(expression: &str) {
    let input = String::from(expression);
    let tokens = parse_preprocessor(&input);
    let ast = tokens_to_ast(&tokens.clone());
    let result = eval(&ast, &State::default());
    println!("{tokens:?}");
    println!("{input:?}");
    println!("{ast:?}");
    println!("{result:?}");
}
