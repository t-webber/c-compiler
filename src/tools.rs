use crate::preprocessor::FilePosition;

pub fn compilation_error(current_position: &FilePosition, message: &str) -> String {
    String::from("\n")
        + &format!(
            "{}:{}:{} {message:?}",
            current_position.filepath, current_position.line, current_position.col
        )
}
