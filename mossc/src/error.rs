
use std::fmt;

#[derive(PartialEq, Eq)]
pub enum ErrorKind {
    Error, Syntax, TypeError, UndefinedSymbol, UndefinedType
}

pub struct Error {
    pub line: usize,
    pub col: usize,
    pub kind: ErrorKind,
    pub text: String
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Line {}, col {}:", self.line + 1, self.col + 1)?;
        let error_spec = match self.kind {
            ErrorKind::Syntax => "Syntax error",
            ErrorKind::Error => "Error",
            ErrorKind::TypeError => "Type error",
            ErrorKind::UndefinedSymbol => "Undefined variable",
            ErrorKind::UndefinedType => "Undefined type"
        };
        writeln!(f, "{}: {}", error_spec, self.text)
    }
}

pub fn scan_error(line: usize, col: usize, text: String) -> Error {
    Error {line, col, text, kind: ErrorKind::Syntax}
}

pub fn syntax_error(line: usize, col: usize, text: String) -> Error {
    Error {line, col, text, kind: ErrorKind::Syntax}
}

pub fn error(line: usize, col: usize, text: String) -> Error {
    Error {line, col, text, kind: ErrorKind::Error}
}

pub fn type_error(line: usize, col: usize, text: String) -> Error {
    Error {line, col, text, kind: ErrorKind::TypeError}
}

pub fn undefined_type(line: usize, col: usize, text: String) -> Error {
    Error {line, col, text, kind: ErrorKind::UndefinedType}
}

pub fn undefined_symbol(line: usize, col: usize, text: String) -> Error {
    Error {line, col, text, kind: ErrorKind::UndefinedSymbol}
}

