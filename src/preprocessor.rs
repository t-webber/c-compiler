use std::clone::Clone;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use crate::eval::{eval, tokens_to_ast};
use crate::parser::parse_preprocessor;

#[derive(Default, Debug)]
pub struct FilePosition {
    pub line: u32,
    pub col: u32,
    pub filename: String,
    pub filepath: String,
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
    values: Vec<String>,
}

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
    IfnDef {
        macro_name: String,
    },
    If {
        expression: bool,
    },
    Elif {
        expression: bool,
    },
    Include {
        filename: String,
    },
    Undef {
        macro_name: String,
    },
    Warning {
        message: String,
    },
    Pragma {
        message: String,
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
    directive_parsing: Pips,
    comment_unclosed_positon: Vec<FilePosition>,
    pub current_position: FilePosition,
    pub defines: HashMap<String, MacroValue>,
    writing_if: bool,
    if_level: u32,
}

impl State {
    fn new_file(&mut self, filename: String, filepath: String) {
        self.current_position.col = 0;
        self.current_position.line = 0;
        self.current_position.filename = filename;
        self.current_position.filepath = filepath;
    }
}

#[derive(Debug)]
pub enum MacroValue {
    String(String),
    Function { args: Vec<String>, body: String },
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
    let mut tmp_dir_state = None;
    let res = 
    match &mut state.directive_parsing {
        Pips::None => String::from(c),
        Pips::DirectiveName(ref name) if c.is_whitespace() && name.is_empty() => {String::new()},
        Pips::DirectiveName(ref name) if c.is_whitespace() => {tmp_dir_state = Some(if name.trim()=="define" { Pips::DirectiveArgs(vec![]) } else { Pips::DirectiveValue(String::new()) }); current_directive.values.push(name.clone()); String::new()},
        Pips::DirectiveName(ref mut name) =>{name.push(c); String::new()},
        Pips::DirectiveArgs(ref mut args ) if args.is_empty() && c == '(' => {args.push(String::new()); String::new()},
        Pips::DirectiveArgs(_) if c == '(' => panic!("Nested parenthesis are not supported"),
        Pips::DirectiveArgs(ref mut args ) if c == ')' => {tmp_dir_state = Some(Pips::DirectiveValue(String::new())); current_directive.values.extend(args.iter().map(Clone::clone)); String::new()},
        Pips::DirectiveArgs(ref mut args ) if args.is_empty() => {tmp_dir_state = Some(Pips::DirectiveValue(String::from(c))); String::new()},
        Pips::DirectiveArgs(ref mut args ) => {args.last_mut().expect("Fatal Error: we're fucked!").push(c); String::new()},
        Pips::DirectiveValue(ref mut value) => {value.push(c); String::new()},
    };
    if let Some(newstate) = tmp_dir_state {
        state.directive_parsing = newstate;
    }
    res
}

#[rustfmt::skip]
pub fn preprocess_character(c: char, state: &mut State, previous_char: &mut char, current_directive: &mut StoreDirective) -> String {
    let in_comment = state.comment_level>0 || state.inline_comment;
    // Match double chars tokens
    let prev = *previous_char;
    *previous_char = c;
    let character = match c {
        '/' if prev =='*' && in_comment => {state.comment_level=state.comment_level.checked_sub(1).expect("*/ unmatched");*previous_char=' ';state.inline_comment=false;String::new()},
        '/' if prev =='*' => {panic!("*/ unmatched")},
        '/' if prev =='/' => {state.inline_comment=true;*previous_char=' ';String::new()} ,
        '*' if prev =='/' => {state.comment_level+=1;state.comment_unclosed_positon.push(state.current_position.clone());*previous_char=' ';String::new()},
        _ if (prev=='/' || prev=='*') && in_comment => {  String::new() },
        _ if prev=='/' || prev=='*' => { deal_with_c(prev, state, current_directive)+deal_with_c(c, state, current_directive).as_str() },
        '/'|'*' => { String::new() }
        _ if in_comment => { String::new() },
        '#' => match state.directive_parsing {
                Pips::None => { state.directive_parsing = Pips::DirectiveName(String::new()); String::new() },
                _ => {deal_with_c(c, state, current_directive)}
            },
        _ => { deal_with_c(c, state, current_directive) }
    };
    if state.writing_if {
        character
    } else {
        String::new()
    }
}

#[rustfmt::skip]
fn preprocess_define(directive: &Directive, state: &mut State) -> String {
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

fn look_for_file(filename: &String, state: &mut State) -> File {
    let places: Vec<PathBuf> = vec![
        PathBuf::from(&state.current_position.filepath)
            .parent()
            .unwrap()
            .to_owned(),
        PathBuf::from("/usr/include/"),
        PathBuf::from("/usr/local/include/"),
        PathBuf::from("/usr/include/x86_64-linux-gnu/"),
    ];
    state.current_position.filename = filename.clone();
    for place in places {
        let filepath = place.join(Path::new(&filename));
        if filepath.exists() {
            state.current_position.filepath = String::from(filepath.as_os_str().to_str().unwrap());
            return File::open(filepath).expect("Failed to open file from local directory");
        }
    }
    panic!("Header not found")
}

fn preprocess_include(filename: &String, state: &mut State) -> String {
    let mut content: String = String::new();
    let old_position = state.current_position.clone();
    look_for_file(filename, state)
        .read_to_string(&mut content)
        .expect("Failed to convert file");
    let preprocessed_file = preprocess(&content, state);
    state.current_position = old_position;
    preprocessed_file
}

#[derive(PartialEq, Debug)]
enum DirectiveParsingState {
    Name,
    AfterName,
    Args,
    Value,
}

fn convert_define_from_store(values: &&str) -> Directive {
    let mut state = DirectiveParsingState::Name;
    let mut brace_level: usize = 0;
    let mut macro_name = String::new();
    let mut args: Vec<String> = vec![];
    let mut value = String::new();
    values.chars().for_each(|c| match c {
        _ if state == DirectiveParsingState::Value => value.push(c),
        '(' if state == DirectiveParsingState::Name
            || state == DirectiveParsingState::AfterName =>
        {
            brace_level += 1;
            state = DirectiveParsingState::Args;
            args.push(String::new());
            value.push(c);
        }
        '(' if brace_level > 0 => {
            state = DirectiveParsingState::Value;
            args.clear();
            value.push(c);
        }
        '(' => {
            brace_level += 1;
            value.push(c);
        }
        ' ' if state == DirectiveParsingState::Name => state = DirectiveParsingState::AfterName,
        ' ' => {}
        ')' if state == DirectiveParsingState::Name => {
            panic!("Unexpected ')' in macro name")
        }
        ')' if brace_level > 1 => {
            brace_level -= 1;
            value.push(c);
            args.last_mut().unwrap().push(c);
        }
        ')' if state == DirectiveParsingState::Args => {
            brace_level = brace_level.checked_sub(1).expect("Unmatched (");
            state = DirectiveParsingState::Value;
            value.clear();
        }
        ')' => value.push(c),
        ',' => args.push(String::new()),
        _ if state == DirectiveParsingState::Name => macro_name.push(c),
        _ if state == DirectiveParsingState::AfterName => {
            state = DirectiveParsingState::Value;
            value.push(c);
        }
        _ => {
            args.last_mut().unwrap().push(c);
            value.push(c);
        }
    });
    if value.is_empty() {
        value = format!(
            "({})",
            args.iter().fold(String::new(), |acc, s| acc + s.as_str())
        );
        args.clear();
    }

    Directive::Define {
        macro_name,
        macro_args: args,
        macro_value: value,
    }
}

fn convert_from_store(directive: &StoreDirective, state: &mut State) -> Directive {
    match directive
        .values
        .iter()
        .map(|s: &String| s.as_str().trim())
        .collect::<Vec<&str>>()
        .as_slice()
    {
        ["define", values] => convert_define_from_store(values),
        ["undef", macro_name] => Directive::Undef {
            macro_name: String::from(*macro_name),
        },
        ["if", expression_string] => {
            let ast = tokens_to_ast(&parse_preprocessor(expression_string));
            Directive::If {
                expression: eval(&ast, state) != 0,
            }
        }
        ["elif", expression_string] => {
            let ast = tokens_to_ast(&parse_preprocessor(expression_string));
            Directive::Elif {
                expression: eval(&ast, state) != 0,
            }
        }
        ["endif"] => Directive::EndIf {},
        ["else"] => Directive::Else,
        ["ifdef", macro_name] => Directive::IfDef {
            macro_name: String::from(*macro_name),
        },
        ["ifndef", macro_name] => Directive::IfnDef {
            macro_name: String::from(*macro_name),
        },
        ["include", filename] => {
            let trimed_filename = filename.trim();
            let clamped_filename = String::from(&trimed_filename[1..trimed_filename.len() - 1]);
            Directive::Include {
                filename: clamped_filename,
            }
        }
        ["error", message] => Directive::Error {
            message: String::from(*message),
        },
        ["warning", message] => Directive::Warning {
            message: String::from(*message),
        },
        ["pragma", message] => Directive::Pragma {
            message: String::from(*message),
        },
        x => panic!("Not a valid directive : {x:?}"),
    }
}

pub fn preprocess_directive(directive: &Directive, state: &mut State) -> String {
    // println!("Directive: {directive:?}");
    if state.if_level > 0 {
        match directive {
            Directive::EndIf => {
                state.if_level = state
                    .if_level
                    .checked_sub(1)
                    .expect("We're indeniably fucked");
            }
            Directive::Else { .. } | Directive::Elif { .. } => (),
            _ if state.writing_if => (),
            _ => return String::new(),
        }
    }
    match directive {
        Directive::Define { .. } => preprocess_define(directive, state),
        Directive::IfDef { macro_name } => {
            state.if_level += 1;
            state.writing_if = state.defines.contains_key(macro_name);
            String::new()
        }
        Directive::IfnDef { macro_name } => {
            state.if_level += 1;
            state.writing_if = !state.defines.contains_key(macro_name);
            String::new()
        }
        Directive::If { expression } => {
            state.if_level += 1;
            state.writing_if = *expression;
            String::new()
        }
        Directive::Elif { expression } => {
            if state.writing_if {
                state.writing_if = false;
                String::new()
            } else {
                state.writing_if = *expression;
                String::new()
            }
        }
        Directive::Include { filename } => preprocess_include(filename, state),
        Directive::Undef { macro_name } => {
            state.defines.remove(macro_name);
            String::new()
        }
        Directive::Else => {
            state.writing_if = !state.writing_if;
            String::new()
        }
        Directive::EndIf => {
            state.writing_if = true;
            String::new()
        }
        Directive::Error { message } => {
            panic!("Encountered preprocessor error: {message}")
        }
        Directive::Warning { message } => {
            eprintln!("Encountered preprocessor warning: {message}");
            String::new()
        }
        Directive::Pragma { message } => {
            eprintln!("Encountered a pragma directive, not supported yet ({message})");
            String::new()
        }
        Directive::None => panic!("Missing directive name after #"),
    }
}

pub fn preprocess(content: &str, state: &mut State) -> String {
    let mut lines: Vec<(u32, String)> = vec![];
    let mut previous_line_escaped: bool = false;
    let mut line_number: u32 = 1;
    for line in content.lines() {
        let escaped = line.ends_with('\\');
        let trimed_line = if escaped {
            &line[0..line.len() - 1]
        } else {
            line
        };
        if previous_line_escaped {
            lines
                .last_mut()
                .expect("Unrecoverable error")
                .1
                .push_str(trimed_line.trim_start());
        } else {
            lines.push((line_number, String::from(trimed_line)));
            line_number += 1;
        }
        previous_line_escaped = escaped;
    }

    let processed_file = lines
        .iter()
        .map(|(line_number, line)| {
            state.current_position.col = 0;
            state.current_position.line = *line_number;
            state.inline_comment = false;
            state.directive_parsing = Pips::None;
            let mut current_directive = StoreDirective::default();
            let line_string = line.to_string();
            let mut previous_char: char = ' ';
            let mut preprocessed_line = line_string
                .chars()
                .map(|c| {
                    state.current_position.col += 1;
                    preprocess_character(c, state, &mut previous_char, &mut current_directive)
                })
                .collect::<String>();
            if !preprocessed_line.trim().is_empty() {
                preprocessed_line += "\n";
            }
            // println!("Hashmap: {}", format!("{:?}", state.defines).blue());
            match &state.directive_parsing {
                Pips::DirectiveValue(value) => {
                    if !value.is_empty() {
                        current_directive.values.push(value.clone());
                    }
                    // println!("Struct: {current_directive:?}");
                    preprocess_directive(&convert_from_store(&current_directive, state), state)
                }
                Pips::DirectiveName(name) => {
                    current_directive.values.push(name.clone());
                    // println!("Struct: {current_directive:?}");
                    preprocess_directive(&convert_from_store(&current_directive, state), state)
                }
                Pips::DirectiveArgs(args) => {
                    assert!(args.is_empty(), "Directive args {args:?} not closed");
                    preprocess_directive(&convert_from_store(&current_directive, state), state)
                }
                Pips::None => preprocessed_line,
            }
        })
        .collect();
    assert!(
        state.comment_level == 0,
        "{} {}",
        "/* unmatched",
        state.comment_level.to_string().as_str()
    );
    processed_file
}

pub fn preprocess_unit(filepath: PathBuf) -> String {
    let mut content: String = String::new();
    File::open(&filepath)
        .expect("Failed to read the file")
        .read_to_string(&mut content)
        .expect("Failed to convert the file");
    let mut state = State {
        writing_if: true,
        ..Default::default()
    };
    state.new_file(
        filepath
            .file_name()
            .unwrap()
            .to_owned()
            .into_string()
            .unwrap(),
        filepath.into_os_string().into_string().unwrap(),
    );
    preprocess(&content, &mut state)
}
