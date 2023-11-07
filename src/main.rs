// use std::{fs, path::PathBuf};

use parser::parse_preprocessor;

// mod expression;
mod parser;
// mod preprocessor;

fn main() {
    // let input = fs::read_to_string(PathBuf::from("fichier.c")).unwrap();
    let input = String::from("a$b");
    let parsed = parse_preprocessor(&input);
    println!("{parsed:?}");
    // let preprocessed_file = preprocessor::preprocess_unit(PathBuf::from("fichier.c"));
    // println!("{preprocessed_file}");
}
