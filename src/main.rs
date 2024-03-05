#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
// #![feature(stmt_expr_attributes)]
#![allow(clippy::implicit_return, clippy::single_call_fn)]
#![allow(clippy::pattern_type_mismatch)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::use_debug)]
#![allow(clippy::print_stderr)]
#![allow(clippy::question_mark_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::separated_literal_suffix)]

use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

#[allow(unused)]
use std::env;

mod errors;
mod eval;
mod parser;
mod preprocessor;
mod structs;
mod ternary;

fn main() -> Result<(), io::Error> {
    run_main("./test/file.c")
    // run_main("/usr/lib/gcc/x86_64-linux-gnu/12/include/stddef.h")
    // test_parser(
    //     env::args().collect::<Vec<String>>().get(1).map_or(
    //         // "! defined (_FILE_OFFSET_BITS) || _FILE_OFFSET_BITS != 64",
    //         "! defined (MACRO) || 1",
    //         |argv| argv.as_str(),
    //     ),
    // )
}

#[allow(unused)]
fn run_main(path: &str) -> io::Result<()> {
    let preprocessed_file = preprocessor::preprocess_unit(PathBuf::from(path));
    eprintln!("{preprocessed_file}");
    let mut data: &mut [u8] = &mut [0; 32];
    let mut file = File::create(format!(
        "{}i",
        &path
            .get(0..path.len().checked_sub(1).expect("File was empty."))
            .expect("Empty file found.")
    ))?;
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
    // dbg!(&expression);
    // dbg!(&tast);
    let res = ternary::eval_all(&tast, &mut structs::State::default());
    // dbg!(&res);
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
