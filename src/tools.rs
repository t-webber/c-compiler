#[derive(Default, Debug)]
pub struct FilePosition {
    pub line: u32,
    pub col: u32,
    pub filename: String,
    pub filepath: String,
}

#[allow(unused)]
#[derive(Debug)]
pub enum GeneralError<'a> {
    UnclosedComment(&'a FilePosition, &'a str), // (start, level)
    UnclosedString,
    UnclosedChar,
    UnclosedParenthesis,
    UnclosedBracket,
    UnclosedBrace,
    MainNotFound,
}

#[rustfmt::skip]
impl<'a> GeneralError<'a> {
    fn get_code(&self) -> (u32, String) {
        match self {
            GeneralError::UnclosedComment(start, level) => (1, format!("unclosed comment that started {}:{}:{} with level {level}",
            start.filepath, start.line, start.col)),
            GeneralError::UnclosedString => (2, String::from("unclosed string")),
            GeneralError::UnclosedChar => (3, String::from("unclosed char")),
            GeneralError::UnclosedParenthesis => (4, String::from("unclosed parenthesis")),
            GeneralError::UnclosedBracket => (5, String::from("unclosed bracket")),
            GeneralError::UnclosedBrace => (6, String::from("unclosed brace")),
            GeneralError::MainNotFound => (7, String::from("main not found")),
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub enum PreprocessorError<'a> {
    DirectiveNameMissing,
    InvalidSharpPosition,
    //
    InvalidFileName(&'a str),
    FileNotFound(&'a str),
    FileNotReadable(&'a str),
    //
    InvalidMaxcroName(&'a str),
    MacroArgsNotClosed,
    MacroNotDefined(&'a str),
    //
    IncompleteOperator(&'a str),
    InvalidOperator(&'a str),
    DefinedChildNotLeaf,
    DefinedChildNotMacro,
    InvalidLeaf(&'a str),
    StringsNotAllowed,
    //
    ElifWithoutIf,
    ElseWithoutIf,
    EndifWithoutIf,
    //
    DirectiveError(&'a str),
    DirectiveWarning(&'a str),
    DirectiveUnknown(&'a str),
}

#[rustfmt::skip]
impl<'a> PreprocessorError<'a> {
    fn get_code(&'a self) -> (u32, String) {
        match self {
            PreprocessorError::DirectiveNameMissing => (1, String::from("directive name missing")),
            PreprocessorError::InvalidSharpPosition => (2, String::from("invalid sharp position")),
            //
            PreprocessorError::InvalidFileName(filename) => (11, format!("invalid filename {filename}")),
            PreprocessorError::FileNotFound(filename) => (12, format!("file {filename} not found")),
            PreprocessorError::FileNotReadable(filename) => (13, format!("file {filename} not readable")),
            //
            PreprocessorError::InvalidMaxcroName(name) => (21, format!("invalid macro name {name}")),
            PreprocessorError::MacroArgsNotClosed => (22, String::from("macro arguments not closed")),
            PreprocessorError::MacroNotDefined(name) => (23, format!("macro {name} not defined")),
            //
            PreprocessorError::IncompleteOperator(operator) => (31,format!("incomplete operator {operator}: missing argument")),
            PreprocessorError::InvalidOperator(operator) => (31,format!("invalid operator {operator}: not supported in preprocessor")),
            PreprocessorError::DefinedChildNotLeaf => (32, String::from("child of \"defined\" should be a leaf")),
            PreprocessorError::DefinedChildNotMacro => (33, String::from("a macro was expected after defined")),
            PreprocessorError::InvalidLeaf(leaf) => (34, format!("invalid leaf {leaf}")),
            PreprocessorError::StringsNotAllowed => (35, String::from("strings not allowed in preprocessor")),
            //
            PreprocessorError::ElifWithoutIf => (41, String::from("elif found without an if")),
            PreprocessorError::ElseWithoutIf => (42, String::from("else found without an if")),
            PreprocessorError::EndifWithoutIf => (43, String::from("endif found without an if")),
            //
            PreprocessorError::DirectiveError(message) => (51, format!("#error raised {message}")),
            PreprocessorError::DirectiveWarning(message) => (52, format!("#warning raised {message}")),
            PreprocessorError::DirectiveUnknown(message) => (53, format!("directive {message} unknown by compiler")),
        }
    }
}

#[rustfmt::skip]
impl<'a> GeneralError<'a> {
        pub fn fail(self, current_position: &FilePosition) -> String {
            let (code, message) = self.get_code();
            String::from("\n")
                + &format!(
                    "[ERROR: {:2e}]\t{}:{}:{}   {:?}",
                    code, current_position.filepath, current_position.line, current_position.col, message
                )
        }
    }

#[rustfmt::skip]
impl<'a> PreprocessorError<'a> {
        pub fn fail(self, current_position: &FilePosition) -> String {
            let (code, message) = self.get_code();
            String::from("\n")
                + &format!(
                    "[ERROR: {:2e}]\t{}:{}:{}   {:?}",
                    100+code, current_position.filepath, current_position.line, current_position.col, message
                )
        }
    }
