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
    UnsupportedOS(&'msg str),
}

impl<'msg> GetCode<'msg> for SystemError<'msg> {
    const INCREMENT: u32 = 0;
    fn get_code(&self) -> (u32, String) {
        match self {
            SystemError::UnsupportedOS(msg) => (
                9,
                String::from(&format!("operating system '{msg}' isn't supported")),
            ),
        }
    }
}

#[allow(unused)]
pub enum GeneralError<'msg> {
    UnclosedComment {
        file_position: &'msg FilePosition,
        level: u32,
    },
    UnclosedString,
    UnclosedChar,
    UnclosedParenthesis,
    UnclosedBracket,
    UnclosedBrace,
    MainNotFound,
    AccessLocalDenied,
}

#[rustfmt::skip]
impl<'msg> GetCode<'msg> for GeneralError<'msg> {
    const INCREMENT: u32 = 100;
    fn get_code(&self) -> (u32, String) {
        match self {
            GeneralError::UnclosedComment {file_position , level} => (1, format!("unclosed comment that started {}:{}:{} with level {level}",
                                                                                                file_position.filepath, file_position.line, file_position.col)),
            GeneralError::UnclosedString => (2, String::from("unclosed string")),
            GeneralError::UnclosedChar => (3, String::from("unclosed char")),
            GeneralError::UnclosedParenthesis => (4, String::from("unclosed parenthesis")),
            GeneralError::UnclosedBracket => (5, String::from("unclosed bracket")),
            GeneralError::UnclosedBrace => (6, String::from("unclosed brace")),
            GeneralError::MainNotFound => (7, String::from("main not found")),
            GeneralError::AccessLocalDenied => (8, String::from("path to the source code is denied")),
        }
    }
}

#[allow(unused)]
pub enum PreprocessorError<'msg> {
    Internal(&'msg str),
    //
    DirectiveNameMissing,
    InvalidSharpPosition,
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
            PreprocessorError::Internal(message) => (99, format!("internal error: {message}.\nPlease raise an issue") ),

            PreprocessorError::DirectiveNameMissing => (1, String::from("directive name missing")),
            PreprocessorError::InvalidSharpPosition => (2, String::from("invalid sharp position")),
            //
            PreprocessorError::InvalidFileName(filename) => (11, format!("invalid filename {filename}")),
            PreprocessorError::FileNotFound(filename) => (12, format!("file {filename} not found")),
            PreprocessorError::FileNotReadable(filename) => (13, format!("file {filename} not readable")),
            //
            PreprocessorError::InvalidMacroName(name) => (21, format!("invalid macro name {name}")),
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
