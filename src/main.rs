use std::path::PathBuf;

mod preprocessor;


fn main() {
    let preprocessed_file = preprocessor::preprocess_unit(PathBuf::from("fichier.c"));
    println!("{preprocessed_file}");
}
