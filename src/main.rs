use std::fs::File;
use std::io::prelude::*;

use std::collections::HashSet;

fn preprocess(content: String, _hashmap: &mut HashSet<String>) -> String {
    content.lines().map(|line| {
        let line_string= line.to_string();
        line_string+"\n"
    }).collect()
}

fn preprocess_unit(content: String) -> String {
    let mut includes = HashSet::<String>::new();
    preprocess(content, &mut includes)
}

fn main() {
    let mut content: String = String::new();
    File::open("fichier.c")
        .expect("Failed to read fichier.c")
        .read_to_string(&mut content)
        .unwrap();
    let preprocessed_file: String = preprocess_unit(content);

    println!("{preprocessed_file}");
}
