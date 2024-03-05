use std::collections::HashMap;

use crate::errors::FilePosition;

/// Preprocessor Directive Parsing State
///
#[derive(Eq, PartialEq, Default, Debug)]
pub enum Pips {
    #[default]
    None,
    DirectiveName(String),
    DirectiveArgs(Vec<String>),
    DirectiveValue(String),
}

#[derive(Default, Debug)]
pub struct StoreDirective {
    pub values: Vec<String>,
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
    pub comment_level: u32,
    pub inline_comment: bool,
    pub directive_parsing: Pips,
    pub comment_unclosed_positon: Vec<FilePosition>,
    pub defines: HashMap<String, MacroValue>,
    pub if_writing: bool,
    pub if_level: u32,
    pub include_stack: Vec<FilePosition>,
    pub current_position: FilePosition,
}

impl State {
    pub fn new_file(&mut self, filename: String, filepath: String) {
        self.include_stack.push(self.current_position.clone());

        self.current_position.filename = filename;
        self.current_position.filepath = filepath;
        self.current_position.col = 0;
        self.current_position.line = 0;
        self.if_level = 0;
        self.if_writing = true;
    }

    pub fn end_file(&mut self) {
        self.include_stack.pop();
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

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}
