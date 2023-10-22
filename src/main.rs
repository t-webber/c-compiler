use std::fs::File;
use std::io::prelude::*;

use colored::Colorize;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Default)]
struct FilePosition {
    line: u32,
    col: u32,
    filename: String,
    filepath: String,
}

/// Preprocessor Directive Parsing State
///
#[derive(PartialEq)]
#[derive(Default)]
enum Pips {
    #[default]
    None,
    DirectiveValues(Vec<String>),
}

#[derive(Debug)]
#[derive(Default)]
struct PreprocessorDirective {
    values: Vec<String>,
}


#[derive(Default)]
struct PreprocessorState {
    comment_level: u32,
    inline_comment: bool,
    directive_parsing_state: Pips,
    comment_unclosed_positon: Vec<FilePosition>,
    current_position: FilePosition,
    _included: HashSet<String>,
}

impl Clone for FilePosition {
    fn clone(&self) -> Self {
        Self {
            line: self.line,
            col: self.col,
            filename: self.filename.clone(),
            filepath: self.filepath.clone(),
        }
    }
}


#[rustfmt::skip]
fn deal_with_c(c: char, state: &mut PreprocessorState) -> String {
    match &mut state.directive_parsing_state {
        Pips::None => String::from(c),
        Pips::DirectiveValues(ref mut values) if c.is_whitespace() => { values.push(String::new()); String::new()},
        Pips::DirectiveValues(ref mut values) => { values.last_mut().unwrap().push(c); String::new()},
    }
}

#[rustfmt::skip]
fn preprocess_character(c: char, state: &mut PreprocessorState, previous_char: &mut char) -> String {
    let in_comment = state.comment_level>0 || state.inline_comment;
    // Match double chars tokens
    let prev = *previous_char;
    *previous_char = c;
    match c {
        '/' if prev =='*' && in_comment => {state.comment_level=state.comment_level.checked_sub(1).expect("*/ unmatched");*previous_char=' ';state.inline_comment=false;String::new()},
        '/' if prev =='*' => {panic!("*/ unmatched")},
        '/' if prev =='/' => {state.inline_comment=true;*previous_char=' ';String::new()} ,
        '*' if prev =='/' => {state.comment_level+=1;state.comment_unclosed_positon.push(state.current_position.clone());*previous_char=' ';String::new()},
        _ if (prev=='/' || prev=='*') && in_comment => {  String::new() },
        _ if in_comment => { String::new() },
        '#' => match state.directive_parsing_state {
                Pips::None => { state.directive_parsing_state = Pips::DirectiveValues(vec![String::new()]); String::new() },
                Pips::DirectiveValues(_) => panic!("Two consecutive \"#\" found"),
            },
        _ if prev=='/' || prev=='*' => {  String::from(prev)+deal_with_c(c, state).as_str() },
        '/'|'*' => { String::new() }
        _ => { deal_with_c(c, state) }
    }
}

fn build_directive_tree(_current_directive: &PreprocessorDirective) {}

fn eval_tree() {}

fn process_directive(current_directive: &PreprocessorDirective) -> String {
    build_directive_tree(current_directive);
    eval_tree();
    match current_directive.values.get(0).unwrap().as_str() {
        "define" => String::from("define"),
        "ifdef" => todo!(),
        "if" => todo!(),
        "include" => String::from("include"),
        _ => todo!(),
    }
}

fn preprocess(content: &str, state: &mut PreprocessorState) -> String {
    let processed_file = content
        .lines()
        .map(|line| {
            state.inline_comment = false;
            state.directive_parsing_state = Pips::None;
            let mut current_directive = PreprocessorDirective::default();
            let line_string = line.to_string();
            let mut previous_char: char = ' ';
            let preprocessed_line = line_string
                .chars()
                .map(|c| preprocess_character(c, state, &mut previous_char))
                .collect::<String>()
                + "\n";
            if let Pips::DirectiveValues(values) = &state.directive_parsing_state {
                current_directive.values = values.clone();
            };
            if state.directive_parsing_state == Pips::None {
                preprocessed_line
            } else {
                // Preprocessor directive
                println!("Struct: {current_directive:?}");
                process_directive(&current_directive)+"\n"
            }
        })
        .collect();
    assert!(
        state.comment_level == 0,
        "{} {}",
        "/* unmatched".red(),
        state.comment_level.to_string().as_str().red()
    );
    processed_file
}

fn preprocess_unit(filepath: PathBuf) -> String {
    let mut content: String = String::new();
    File::open(&filepath)
        .expect("Failed to read fichier.c")
        .read_to_string(&mut content)
        .unwrap();
    let mut state = PreprocessorState::default();
    state.current_position.filename = filepath
        .file_name()
        .unwrap()
        .to_owned()
        .into_string()
        .unwrap();
    state.current_position.filepath = filepath.into_os_string().into_string().unwrap();
    preprocess(&content, &mut state)
}

fn main() {
    let preprocessed_file = preprocess_unit(PathBuf::from("fichier.c"));
    println!("{preprocessed_file}");
}
