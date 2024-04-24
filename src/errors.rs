#[derive(Default, Debug)]
pub struct FilePosition {
    pub line: u32,
    pub col: u32,
    pub filename: String,
    pub filepath: String,
}

trait GetCode<'msg> {
    const INCREMENT: u32;
    fn get_code(&'msg self) -> (u32, String);
}

#[allow(unused)]
pub enum SystemError<'msg> {
    UnsupportedOS,
    CompilationError(&'msg str),
    AccessLocalDenied,
    AccessLibraryDenied(&'msg str),
}

#[rustfmt::skip]
impl<'msg> GetCode<'msg> for SystemError<'msg> {
    const INCREMENT: u32 = 0;
    fn get_code(&self) -> (u32, String) {
        match self {
            Self::UnsupportedOS =>  (1, String::from("operating system '{msg}' isn't supported")),
            Self::CompilationError(message) => (2, String::from(&format!("compilation error: {message}"))),
            Self::AccessLocalDenied => (8, String::from("path to the source code is denied")),
            Self::AccessLibraryDenied(msg) => (9, format!("path to the library {msg} is denied")),
        }
    }
}

#[allow(unused)]
pub enum GeneralError<'msg> {
    UnclosedComment { file_position: &'msg FilePosition, level: u32 },
    UnclosedString,
    UnclosedChar,
    UnclosedParenthesis,
    UnclosedBracket,
    UnclosedBrace,
    MainNotFound,
    Overflow,
}

#[rustfmt::skip]
impl<'msg> GetCode<'msg> for GeneralError<'msg> {
    const INCREMENT: u32 = 100;
    fn get_code(&self) -> (u32, String) {
        match self {
            Self::UnclosedComment {file_position , level} => (1, format!("unclosed comment that started {}:{}:{} with level {level}",
                                                                                                file_position.filepath, file_position.line, file_position.col)),
            Self::UnclosedString => (2, String::from("unclosed string")),
            Self::UnclosedChar => (3, String::from("unclosed char")),
            Self::UnclosedParenthesis => (4, String::from("unclosed parenthesis")),
            Self::UnclosedBracket => (5, String::from("unclosed bracket")),
            Self::UnclosedBrace => (6, String::from("unclosed brace")),
            Self::MainNotFound => (7, String::from("main not found")),
            Self::Overflow => (9, String::from("overflow on airthmetic operation")),
        }
    }
}

#[allow(unused)]
pub enum PreprocessorError<'msg> {
    Internal(&'msg str),
    //
    DirectiveNameMissing,
    InvalidSharpPosition,
    DefinedSynthax,
    //
    InvalidFileName(&'msg str),
    FileNotFound(&'msg str),
    FileNotReadable(&'msg str),
    //
    InvalidMacroName(&'msg str),
    MacroArgsNotClosed,
    MacroNotDefined(&'msg str),
    //
    IncompleteOperator(&'msg str),
    InvalidOperator(&'msg str),
    DefinedChildNotLeaf,
    DefinedChildNotMacro,
    InvalidLeaf(&'msg str),
    StringsNotAllowed,
    BinarySynthaxOnUnary(&'msg str),
    EmptyParenthesis,
    //
    ElifWithoutIf,
    ElseWithoutIf,
    EndifWithoutIf,
    //
    DirectiveError(&'msg str),
    DirectiveWarning(&'msg str),
    DirectiveUnknown(&'msg str),
}

#[rustfmt::skip]
impl<'msg> GetCode<'msg> for PreprocessorError<'msg> {
    const INCREMENT: u32 = 200;
    fn get_code(&'msg self) -> (u32, String) {
        match self {
            Self::Internal(message) => (99, format!("internal error: {message}.\nPlease raise an issue") ),

            Self::DirectiveNameMissing => (1, String::from("directive name missing")),
            Self::InvalidSharpPosition => (2, String::from("invalid sharp position")),
            Self::DefinedSynthax => (3, String::from("expected to find a macro between \" or <> in defined directive")),
            //
            Self::InvalidFileName(filename) => (11, format!("invalid filename {filename}")),
            Self::FileNotFound(filename) => (12, format!("file {filename} not found")),
            Self::FileNotReadable(filename) => (13, format!("file {filename} not readable")),
            //
            Self::InvalidMacroName(name) => (21, format!("invalid macro name {name}")),
            Self::MacroArgsNotClosed => (22, String::from("macro arguments not closed")),
            Self::MacroNotDefined(name) => (23, format!("macro {name} not defined")),
            //
            Self::IncompleteOperator(operator) => (31,format!("incomplete operator {operator}: missing argument")),
            Self::InvalidOperator(operator) => (31,format!("invalid operator {operator}: not supported in preprocessor")),
            Self::DefinedChildNotLeaf => (32, String::from("child of \"defined\" should be a leaf")),
            Self::DefinedChildNotMacro => (33, String::from("a macro was expected after defined")),
            Self::InvalidLeaf(leaf) => (34, format!("invalid leaf {leaf}")),
            Self::StringsNotAllowed => (35, String::from("strings not allowed in preprocessor")),
            Self::BinarySynthaxOnUnary(operator) => (36, format!("found the unary operator {operator}, following a binary operator synthax")),
            Self::EmptyParenthesis => (37, String::from("empty parenthesis")),
            //
            Self::ElifWithoutIf => (41, String::from("elif found without an if")),
            Self::ElseWithoutIf => (42, String::from("else found without an if")),
            Self::EndifWithoutIf => (43, String::from("endif found without an if")),
            //
            Self::DirectiveError(message) => (51, format!("#error raised {message}")),
            Self::DirectiveWarning(message) => (52, format!("#warning raised {message}")),
            Self::DirectiveUnknown(message) => (53, format!("directive {message} unknown by compiler")),
        }
    }
}

pub trait FailError {
    fn fail(self, current_position: &FilePosition) -> String;
    fn fail_with_panic(self, current_position: &FilePosition) -> !;
    fn fail_with_warning(self, current_position: &FilePosition);
}

impl<'msg> FailError for SystemError<'msg> {
    fn fail(self, current_position: &FilePosition) -> String {
        let (code, message) = self.get_code();
        format!(
            "\n[ERROR {:0>3}]\t{}:{}:{}   {:?}",
            code.checked_add(Self::INCREMENT).unwrap_or(Self::INCREMENT),
            current_position.filepath,
            current_position.line,
            current_position.col,
            message
        )
    }

    #[allow(clippy::panic)]
    #[allow(unused)]
    fn fail_with_panic(self, current_position: &FilePosition) -> ! {
        panic!("{}", self.fail(current_position));
    }

    #[allow(clippy::print_stderr)]
    #[allow(unused)]
    fn fail_with_warning(self, current_position: &FilePosition) {
        eprintln!("{}", self.fail(current_position));
    }
}

impl<'msg> FailError for GeneralError<'msg> {
    fn fail(self, current_position: &FilePosition) -> String {
        let (code, message) = self.get_code();
        format!(
            "\n[ERROR: {:0>3}]\t{}:{}:{}   {:?}",
            code.checked_add(Self::INCREMENT).unwrap_or(Self::INCREMENT),
            current_position.filepath,
            current_position.line,
            current_position.col,
            message
        )
    }

    #[allow(clippy::panic)]
    #[allow(unused)]
    fn fail_with_panic(self, current_position: &FilePosition) -> ! {
        panic!("{}", self.fail(current_position));
    }

    #[allow(clippy::print_stderr)]
    #[allow(unused)]
    fn fail_with_warning(self, current_position: &FilePosition) {
        eprintln!("{}", self.fail(current_position));
    }
}

impl<'msg> FailError for PreprocessorError<'msg> {
    fn fail(self, current_position: &FilePosition) -> String {
        let (code, message) = self.get_code();
        format!(
            "\n[ERROR {:0>3}]\t{}:{}:{}   {:?}",
            code.checked_add(Self::INCREMENT).unwrap_or(Self::INCREMENT),
            current_position.filepath,
            current_position.line,
            current_position.col,
            message
        )
    }

    #[allow(clippy::panic)]
    #[allow(unused)]
    fn fail_with_panic(self, current_position: &FilePosition) -> ! {
        panic!("{}", self.fail(current_position));
    }

    #[allow(clippy::print_stderr)]
    #[allow(unused)]
    fn fail_with_warning(self, current_position: &FilePosition) {
        eprintln!("{}", self.fail(current_position));
    }
}
