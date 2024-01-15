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
use tools::FilePosition;

fn main() -> std::io::Result<()> {
    // run_main("./test/fichier.c")
    // run_main("/usr/lib/gcc/x86_64-linux-gnu/12/include/stddef.h")
    // test_parser("A && B || C");
    test_parser("!1 || 2 && 3");
    Ok(())
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
    dbg!(input.clone());
    let tokens = parse_preprocessor(&input);
    dbg!(tokens.clone());
    let ast = tokens_to_ast(&tokens.clone(), &FilePosition::default());
    dbg!(ast.clone());
    let result = eval(&ast, &State::default());
    dbg!(result);
    // println!("{input:?}\n");
    // println!("{tokens:?}\n");
    // println!("{ast:?}\n");
    // println!("{result:?}\n");
}
