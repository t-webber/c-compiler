#![allow(unused)]
use std::io::prelude::*;
use std::path::PathBuf;
use std::{env, fs::File};

mod errors;
mod eval;
mod parser;
mod preprocessor;
mod structs;
mod ternary;

fn main() {
    println!("Hello, world!");
    //-> std::io::Result<()> {
    run_main("./test/fichier.c");
    // run_main("/usr/lib/gcc/x86_64-linux-gnu/12/include/stddef.h")
    // test_parser(
    //     env::args()
    //         .collect::<Vec<String>>()
    //         .get(1)
    //         .map_or("1==2", |f| f.as_str()),
    // );
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
    // dbg!(&input);
    let tokens = parser::parse_preprocessor(&input);
    // dbg!(&tokens);
    let tast = ternary::vec2ternary_ast(tokens);
    dbg!(&tast);
    let res = ternary::eval_all(&tast, &mut structs::State::default());
    dbg!(&res);
    //     let ast = tokens_to_ast(&mut tokens.clone(), &mut FilePosition::default());
    //     dbg!(&ast);
    //     let result = eval(&ast, &mut State::default());
    //     dbg!(&result);
    //     dbg!(&expression);
    //     // println!("{input:?}\n");
    //     // println!("{tokens:?}\n");
    //     // println!("{ast:?}\n");
    //     // println!("{result:?}\n");
}
