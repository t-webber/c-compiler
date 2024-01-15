#[derive(Default, Debug)]

pub struct FilePosition {
    pub line: u32,
    pub col: u32,
    pub filename: String,
    pub filepath: String,
}

#[derive(Default, Debug)]
#[warn(clippy::enum_variant_names)]
pub enum Error<'a> {
    #[default]
    Unknown,
    IncompleteOperator,
    DefinedChildNotMacro,
    DefinedChildNotLeaf,
    DirectiveError(&'a str),
    DirectiveWarning(&'a str),
    DirectiveNotImplemented(&'a str),
    DirectiveNameMissing,
    MacroNameNotFound(&'a str),
    InvalidLeaf(&'a str),
}

#[rustfmt::skip]
impl<'a> Error<'a> {
    fn get_code(&'a self) -> (u32, String) {
        match self {
            Error::Unknown => (00, String::from("unknown error")),
            Error::DirectiveError(message) => (111, format!("directive error {message}")),
            Error::DirectiveWarning(message) => (112, format!("directive warning {message}")),
            Error::DirectiveNotImplemented(message) => (113, format!("directive {message} not implemented")),
            Error::DirectiveNameMissing => (114, String::from("a name was excpected after #")),
            Error::IncompleteOperator => (121, String::from("a binary operator is missing a child")),
            Error::DefinedChildNotMacro => (122, String::from("a macro was expected after defined")),
            Error::DefinedChildNotLeaf => (123, String::from("child of \"defined\" should be a leaf")),
            Error::InvalidLeaf(message) => (124, format!("invalid leaf ({message})")),
            Error::MacroNameNotFound(name) => (131, format!("macro {name} not found")),
        }
    }
}

pub fn compilation_error(current_position: &FilePosition, error: Error) -> String {
    let (code, message) = error.get_code();
    String::from("\n")
        + &format!(
            "[ERROR: {}]\t{}:{}:{}   {:?}",
            code, current_position.filepath, current_position.line, current_position.col, message
        )
}
