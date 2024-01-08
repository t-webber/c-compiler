use std::clone::Clone;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use colored::Colorize;

use crate::parser::parse_preprocessor;
use crate::eval::{tokens_to_ast, eval};


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
pub struct StoreDirective {
    values:  Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub enum Directive {
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
        expression: bool,
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
pub struct State {
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
pub enum MacroValue {
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
pub fn deal_with_c(c: char, state: &mut State, current_directive: &mut StoreDirective) -> String {
    let mut tmp_dir_state = None ;
    let res = 
    match &mut state.directive_parsing_state {
        Pips::None => String::from(c),
        Pips::DirectiveName(ref name) if c.is_whitespace() && name.is_empty() => {String::new()},
        Pips::DirectiveName(ref name) if c.is_whitespace() => {tmp_dir_state = Some(Pips::DirectiveArgs(vec![])); current_directive.values.push(name.clone()) ; String::new()},
        Pips::DirectiveName(ref mut name) =>{name.push(c); String::new()},
        Pips::DirectiveArgs(ref mut args ) if args.is_empty() && c == '(' => {args.push(String::new()); String::new()},
        Pips::DirectiveArgs(_) if c == '(' => panic!("Nested parenthesis are not supported"),
        Pips::DirectiveArgs(ref mut args ) if c == ')' => {tmp_dir_state = Some(Pips::DirectiveValue(String::new())); current_directive.values.extend(args.iter().map(Clone::clone)); String::new()},
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
pub fn preprocess_character(c: char, state: &mut State, previous_char: &mut char, current_directive: &mut StoreDirective) -> String {
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
pub fn preprocess_define(directive: &Directive, state: &mut State) -> String {
    if let Directive::Define { macro_name, macro_args, macro_value } = directive {
        if macro_args.is_empty() {
            state.defines.insert(macro_name.clone(), MacroValue::String(macro_value.clone()));
            String::new()
        } else {
            state.defines.insert(macro_name.clone(), MacroValue::Function { args: macro_args.clone(), body: macro_value.clone() });
            String::new()
        }
    } else {
        panic!("Not a define directive");
    }
}

#[derive(PartialEq, Debug)]
enum DirectiveParsingState {
    Name,
    AfterName,
    Args,
    Value,
}

fn convert_from_store(directive: &StoreDirective, state: &mut State) -> Directive {
    match directive.values.iter().map(|s| s.as_str().trim()).collect::<Vec<&str>>().as_slice() {
        ["define", values] => {
            let mut state = DirectiveParsingState::Name ;
            let mut brace_level: usize = 0 ; 
            let mut macro_name = String::new() ;
            let mut args: Vec<String> = vec![] ; 
            let mut value  = String::new() ; 
            values.chars().for_each(|c| match c {
                _ if state == DirectiveParsingState::Value => value.push(c),
                '(' if state == DirectiveParsingState::Name || state == DirectiveParsingState::AfterName => {brace_level+=1 ; state = DirectiveParsingState::Args ; args.push(String::new()) ; value.push(c)},
                '(' if brace_level > 0 => {state = DirectiveParsingState::Value ; args.clear() ; value.push(c);}, 
                '(' => {brace_level+=1 ; value.push(c) }, 
                ' ' if state == DirectiveParsingState::Name => state = DirectiveParsingState::AfterName,
                ' ' => {},
                ')' if state == DirectiveParsingState::Name => panic!("Unexpected ')' in macro name"),
                ')' if brace_level > 1  => {brace_level-=1 ; value.push(c) ;args.last_mut().unwrap().push(c)  },
                ')' if state == DirectiveParsingState::Args => {brace_level = brace_level.checked_sub(1).expect("Unmatched ("); state = DirectiveParsingState::Value; value.clear()},
                ')' => value.push(c),
                ',' => args.push(String::new()),
                _ if state == DirectiveParsingState::Name => macro_name.push(c),
                _ if state == DirectiveParsingState::AfterName => {state =  DirectiveParsingState::Value; value.push(c)},
                _ => {args.last_mut().unwrap().push(c) ; value.push(c)}, 
            });
            if value.is_empty() {
                value = format!("({})", args.iter().fold(String::new(), |acc, s| acc + s.as_str()));
                args.clear();
            }
            
            Directive::Define { macro_name, macro_args : args, macro_value : value}
        },
        ["undef", macro_name] => Directive::Undef { macro_name: (*macro_name).to_string() },
        ["if", expression_string] => { let ast = tokens_to_ast(parse_preprocessor(expression_string)); println!("ast: {ast:?}"); Directive::If { expression: eval(ast, &mut state.defines)!=0 }},
        x => panic!("Not a valid directive : {x:?}"),
    }
}

pub fn preprocess_directive(directive: &Directive, state: &mut State) -> String {
    println!("Directive: {directive:?}");
    match directive {
        Directive::Define { .. } => {
            preprocess_define(directive, state)
        }
        Directive::IfDef { .. } => todo!(),
        Directive::If { expression } => {println!("Expression: {expression}"); String::from(if *expression {"expression vraie"} else {"expression fausse"})},
        Directive::Include { .. } => todo!(),
        Directive::Undef { macro_name } => {state.defines.remove(macro_name); String::new()},
        Directive::Elif { .. } => todo!(),
        Directive::Else => todo!(),
        Directive::EndIf => todo!(),
        Directive::Error { .. } => todo!(),
        Directive::None => todo!(),
    }
}

pub fn preprocess(content: &str, state: &mut State) -> String {
    let processed_file = content
        .lines()
        .map(|line| {
            state.inline_comment = false;
            state.directive_parsing_state = Pips::None;
            let mut current_directive = StoreDirective::default();
            let line_string = line.to_string();
            let mut previous_char: char = ' ';
            let preprocessed_line = line_string
                .chars()
                .map(|c| preprocess_character(c, state, &mut previous_char, &mut current_directive))
                .collect::<String>()
                + "\n";
            println!("Hashmap: {}", format!("{:?}", state.defines).blue());
            match &state.directive_parsing_state {
                Pips::DirectiveValue(value) => {
                    current_directive.values.push(value.clone());
                    println!("Struct: {current_directive:?}");
                    preprocess_directive(&convert_from_store(&current_directive, state), state) + "\n"},
                Pips::DirectiveName(_) => {
                    println!("Struct: {current_directive:?}");
                    preprocess_directive(&convert_from_store(&current_directive, state), state) + "\n"
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
    let mut state = State::default();
    state.current_position.filename = filepath
        .file_name()
        .unwrap()
        .to_owned()
        .into_string()
        .unwrap();
    state.current_position.filepath = filepath.into_os_string().into_string().unwrap();
    preprocess(&content, &mut state)
}