#[derive(Default, Debug)]
pub struct FilePosition {
    pub line: u32,
    pub col: u32,
    pub filename: String,
    pub filepath: String,
}

trait GetCode<'msg> {
    const INCREMENT: u32;
    fn get_code(&'msg self) -> (u32, Box<str>);
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
    fn get_code(&self) -> (u32, Box<str>) {
        match self {
            Self::UnsupportedOS =>  (1, Box::from("your operating system isn't supported")),
            Self::CompilationError(message) => (2, Box::from(format!("compilation error: {message}").as_str())),
            Self::AccessLocalDenied => (8, Box::from("path to the source code is denied")),
            Self::AccessLibraryDenied(msg) => (9, Box::from(format!("path to the library {msg} is denied").as_str())),
        }
    }
}

#[allow(unused)]
pub enum GeneralError<'msg> {
    UnclosedComment { file_position: &'msg FilePosition, level: u32 },
    UnOpenedComment { level: u32 },
    UnclosedString,
    UnclosedChar,
    UnclosedParenthesis,
    UnOpenedParenthesis,
    UnclosedBracket,
    UnclosedBrace,
    MainNotFound,
    Overflow,
    NotImplemented(&'msg str),
}

impl<'msg> GetCode<'msg> for GeneralError<'msg> {
    const INCREMENT: u32 = 100;
    fn get_code(&self) -> (u32, Box<str>) {
        match self {
            | Self::UnclosedComment { file_position, level } => (
                1,
                Box::from(
                    format!(
                        "unclosed comment that started {}:{}:{} with level {level}",
                        file_position.filepath, file_position.line, file_position.col
                    )
                    .as_str(),
                ),
            ),
            | Self::UnOpenedComment { level } => (1, Box::from(format!("unopened comment with level {level}").as_str())),
            | Self::UnclosedString => (2, Box::from("unclosed string")),
            | Self::UnclosedChar => (3, Box::from("unclosed char")),
            | Self::UnclosedParenthesis => (4, Box::from("unclosed parenthesis")),
            | Self::UnclosedBracket => (5, Box::from("unclosed bracket")),
            | Self::UnclosedBrace => (6, Box::from("unclosed brace")),
            | Self::MainNotFound => (7, Box::from("main not found")),
            | Self::Overflow => (9, Box::from("overflow on airthmetic operation")),
            | Self::NotImplemented(msg) => (10, Box::from(format!("feature not implemented yet: {msg}").as_str())),
            | Self::UnOpenedParenthesis => (8, Box::from("unopened parenthesis")),
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
    IncompleteOperator,
    TooManyArguments,
    InvalidOperator(&'msg str),
    DefinedChildNotLeaf,
    DefinedChildNotMacro,
    InvalidLeaf(&'msg str),
    StringsNotAllowed,
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
    fn get_code(&'msg self) -> (u32, Box<str>) {
        match self {
            Self::Internal(message) => (99, Box::from(format!("internal error: {message}.\nPlease raise an issue").as_str())),

            Self::DirectiveNameMissing => (1, Box::from("directive name missing")),
            Self::InvalidSharpPosition => (2, Box::from("invalid sharp position")),
            Self::DefinedSynthax => (3, Box::from("expected to find a macro between \" or <> in defined directive")),
            //
            Self::InvalidFileName(filename) => (11, Box::from(format!("invalid filename {filename}").as_str())),
            Self::FileNotFound(filename) => (12, Box::from(format!("file {filename} not found").as_str())),
            Self::FileNotReadable(filename) => (13, Box::from(format!("file {filename} not readable").as_str())),
            //
            Self::InvalidMacroName(name) => (21, Box::from(format!("invalid macro name {name}").as_str())),
            Self::MacroArgsNotClosed => (22, Box::from("macro arguments not closed")),
            Self::MacroNotDefined(name) => (23, Box::from(format!("macro {name} not defined").as_str())),
            //
            Self::IncompleteOperator => (31, Box::from("incomplete operator: missing argument")),
            Self::TooManyArguments => (32, Box::from("too many arguments")),
            Self::InvalidOperator(operator) => (33,Box::from(format!("invalid operator {operator}: not supported in preprocessor").as_str())),
            Self::DefinedChildNotLeaf => (34, Box::from("child of \"defined\" should be a leaf")),
            Self::DefinedChildNotMacro => (35, Box::from("a macro was expected after defined")),
            Self::InvalidLeaf(leaf) => (36, Box::from(format!("invalid leaf {leaf}").as_str())),
            Self::StringsNotAllowed => (37, Box::from("strings not allowed in preprocessor")),
            Self::EmptyParenthesis => (38, Box::from("empty parenthesis")),
            //
            Self::ElifWithoutIf => (41, Box::from("elif found without an if")),
            Self::ElseWithoutIf => (42, Box::from("else found without an if")),
            Self::EndifWithoutIf => (43, Box::from("endif found without an if")),
            //
            Self::DirectiveError(message) => (51, Box::from(format!("#error raised {message}").as_str())),
            Self::DirectiveWarning(message) => (52, Box::from(format!("#warning raised {message}").as_str())),
            Self::DirectiveUnknown(message) => (53, Box::from(format!("directive {message} unknown by compiler").as_str())),
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
