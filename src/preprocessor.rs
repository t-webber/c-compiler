use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use colored::Colorize;


#[derive(Default, Debug)]
pub struct FilePosition {
    line: u32,
    col: u32,
    filename: String,
    filepath: String,
}

/// Preprocessor Directive Parsing State
///
#[derive(PartialEq, Default, Debug)]
pub enum Pips {
    #[default]
    None,
    DirectiveName(String),
    DirectiveArgs(Vec<String>),
    DirectiveValue(String),
}


#[derive(Default, Debug)]
pub struct StorePreprocessorDirective {
    values:  Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub enum PreprocessorDirective {
    #[default]
    None,
    Define {
        macro_name: String,
        macro_args: Vec<String>,
        macro_value: String,
    },
    IfDef {
        macro_name: String,
    },
    If {
        expression: String,
    },
    Include {
        filename: String,
    },
    Undef {
        macro_name: String,
    },
    Elif {
        expression: String,
    },
    Else,
    EndIf,
    Error {
        message: String,
    },
}

#[derive(Default, Debug)]
pub struct PreprocessorState {
    comment_level: u32,
    inline_comment: bool,
    directive_parsing_state: Pips,
    comment_unclosed_positon: Vec<FilePosition>,
    current_position: FilePosition,
    defines: HashMap<String, MacroValue>,
    _included: HashSet<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
enum MacroValue {
    String(String),
    Function {
        args: Vec<String>,
        body: String,
    }
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
pub fn deal_with_c(c: char, state: &mut PreprocessorState, current_directive: &mut StorePreprocessorDirective) -> String {
    let mut tmp_dir_state = None ;
    let res = 
    match &mut state.directive_parsing_state {
        Pips::None => String::from(c),
        Pips::DirectiveName(ref name) if c.is_whitespace() && name.is_empty() => {String::new()},
        Pips::DirectiveName(ref name) if c.is_whitespace() => {tmp_dir_state = Some(Pips::DirectiveArgs(vec![])); current_directive.values.push(name.clone()) ; String::new()},
        Pips::DirectiveName(ref mut name) =>{name.push(c); String::new()},
        Pips::DirectiveArgs(ref mut args ) if args.is_empty() && c == '(' => {args.push(String::new()); String::new()},
        Pips::DirectiveArgs(_) if c == '(' => panic!("Nested parenthesis are not supported"),
        Pips::DirectiveArgs(ref mut args ) if c == ')' => {tmp_dir_state = Some(Pips::DirectiveValue(String::new())); current_directive.values.extend(args.iter().map(|s| s.to_owned())); String::new()},
        Pips::DirectiveArgs(ref mut args ) if args.is_empty() => {tmp_dir_state = Some(Pips::DirectiveValue(String::from(c))); String::new()},
        Pips::DirectiveArgs(ref mut args ) => {args.last_mut().expect("Fatal Error: we're fucked!").push(c); String::new()},
        Pips::DirectiveValue(ref mut value) => {value.push(c); String::new()},
    } ;
    if let Some(newstate) = tmp_dir_state {
        state.directive_parsing_state = newstate;
    }
    res
}

#[rustfmt::skip]
pub fn preprocess_character(c: char, state: &mut PreprocessorState, previous_char: &mut char, current_directive: &mut StorePreprocessorDirective) -> String {
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
        _ if prev=='/' || prev=='*' => { deal_with_c(prev, state, current_directive)+deal_with_c(c, state, current_directive).as_str() },
        '/'|'*' => { String::new() }
        _ if in_comment => { String::new() },
        '#' => match state.directive_parsing_state {
                Pips::None => { state.directive_parsing_state = Pips::DirectiveName(String::new()); String::new() },
                _ => {deal_with_c(c, state, current_directive)}
            },
        _ => { deal_with_c(c, state, current_directive) }
    }
}

#[rustfmt::skip]
pub fn preprocess_define(directive: &PreprocessorDirective, state: &mut PreprocessorState) -> String {
    if let PreprocessorDirective::Define { macro_name, macro_args, macro_value } = directive {
        match macro_args.len() {
            0 => {
                state.defines.insert(macro_name.clone(), MacroValue::String(macro_value.clone()));
                String::new()
            }
            _ => {
                state.defines.insert(macro_name.clone(), MacroValue::Function { args: macro_args.clone(), body: macro_value.clone() });
                String::new()
            }
        }
    } else {
        panic!("Not a define directive");
    }
}

#[derive(PartialEq, Debug)]
enum PreprocessorDirectiveParsingState {
    Name,
    AfterName,
    Args,
    Value,
}

fn convert_from_store(directive: &StorePreprocessorDirective) -> PreprocessorDirective {
    match directive.values.iter().map(|s| s.as_str().trim()).collect::<Vec<&str>>().as_slice() {
        ["define", values] => {
            let mut state = PreprocessorDirectiveParsingState::Name ;
            let mut brace_level: usize = 0 ; 
            let mut macro_name = String::new() ;
            let mut args: Vec<String> = vec![] ; 
            let mut value  = String::new() ; 
            values.chars().for_each(|c| match c {
                _ if state == PreprocessorDirectiveParsingState::Value => value.push(c),
                '(' if state == PreprocessorDirectiveParsingState::Name || state == PreprocessorDirectiveParsingState::AfterName => {brace_level+=1 ; state = PreprocessorDirectiveParsingState::Args ; args.push(String::new()) ; value.push(c)},
                '(' if brace_level > 0 => {state = PreprocessorDirectiveParsingState::Value ; args.clear() ; value.push(c);}, 
                '(' => {brace_level+=1 ; value.push(c) }, 
                ' ' if state == PreprocessorDirectiveParsingState::Name => state = PreprocessorDirectiveParsingState::AfterName,
                ' ' => {},
                ')' if state == PreprocessorDirectiveParsingState::Name => panic!("Unexpected ')' in macro name"),
                ')' if brace_level > 1  => {brace_level-=1 ; value.push(c) ;args.last_mut().unwrap().push(c)  },
                ')' if state == PreprocessorDirectiveParsingState::Args => {brace_level = brace_level.checked_sub(1).expect("Unmatched ("); state = PreprocessorDirectiveParsingState::Value; value.clear()},
                ')' => value.push(c),
                ',' => args.push(String::new()),
                _ if state == PreprocessorDirectiveParsingState::Name => macro_name.push(c),
                _ if state == PreprocessorDirectiveParsingState::AfterName => {state =  PreprocessorDirectiveParsingState::Value; value.push(c)},
                _ => {args.last_mut().unwrap().push(c) ; value.push(c)}, 
            });
            if value.is_empty() {
                value = format!("({})", args.iter().fold(String::new(), |acc, s| acc + s.as_str()));
                args.clear();
            }
            
            PreprocessorDirective::Define { macro_name, macro_args : args, macro_value : value}
        }
        x => panic!("Not a valid directive : {x:?}"),
    }
}

pub fn preprocess_directive(directive: &PreprocessorDirective) -> String {
    println!("Directive: {directive:?}");
    match directive {
        PreprocessorDirective::Define { .. } => {
            preprocess_define(directive, &mut PreprocessorState::default())
        }
        PreprocessorDirective::IfDef { .. } => todo!(),
        PreprocessorDirective::If { .. } => todo!(),
        PreprocessorDirective::Include { .. } => todo!(),
        PreprocessorDirective::Undef { .. } => todo!(),
        PreprocessorDirective::Elif { .. } => todo!(),
        PreprocessorDirective::Else => todo!(),
        PreprocessorDirective::EndIf => todo!(),
        PreprocessorDirective::Error { .. } => todo!(),
        PreprocessorDirective::None => todo!(),
    }
}

pub fn preprocess(content: &str, state: &mut PreprocessorState) -> String {
    let processed_file = content
        .lines()
        .map(|line| {
            state.inline_comment = false;
            state.directive_parsing_state = Pips::None;
            let mut current_directive = StorePreprocessorDirective::default();
            let line_string = line.to_string();
            let mut previous_char: char = ' ';
            let preprocessed_line = line_string
                .chars()
                .map(|c| preprocess_character(c, state, &mut previous_char, &mut current_directive))
                .collect::<String>()
                + "\n";
            match &state.directive_parsing_state {
                Pips::DirectiveValue(value) => {
                    current_directive.values.push(value.clone());
                    println!("Struct: {current_directive:?}");
                    preprocess_directive(&convert_from_store(&current_directive)) + "\n"},
                Pips::DirectiveName(_) => {
                    println!("Struct: {current_directive:?}");
                    preprocess_directive(&convert_from_store(&current_directive)) + "\n"
                },
                Pips::DirectiveArgs(_) => {
                    panic!("Directive args not closed")
                },
                Pips::None => preprocessed_line,
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

pub fn preprocess_unit(filepath: PathBuf) -> String {
    let mut content: String = String::new();
    File::open(&filepath)
        .expect("Failed to read the file")
        .read_to_string(&mut content)
        .expect("Failed to convert the file");
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