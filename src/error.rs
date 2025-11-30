use std::fmt;

#[derive(Debug, Clone)]
pub enum PslError {
    LexError {
        line: usize,
        col: usize,
        message: String,
    },
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },
    RuntimeError(String),
    TypeError(String),
    NameError(String),
    AccessError(String),
}

impl fmt::Display for PslError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PslError::LexError { line, col, message } => {
                write!(f, "Lexer error at line {}, col {}: {}", line, col, message)
            }
            PslError::ParseError { line, col, message } => {
                write!(f, "Parser error at line {}, col {}: {}", line, col, message)
            }
            PslError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            PslError::TypeError(msg) => write!(f, "Type error: {}", msg),
            PslError::NameError(msg) => write!(f, "Name error: {}", msg),
            PslError::AccessError(msg) => write!(f, "Access error: {}", msg),
        }
    }
}

impl std::error::Error for PslError {}
