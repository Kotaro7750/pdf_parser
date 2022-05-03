use crate::lexer::error as lexer_error;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    NotPDF,
    ParseInteger(std::num::ParseIntError),
    Utf8Error(std::str::Utf8Error),
    TargetNotFound,
    CannotParse,
    Lexer(lexer_error::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseInteger(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Utf8Error(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::NotPDF => write!(f, "This file is not valid pdf file"),
            Error::ParseInteger(e) => write!(f, "{}", e),
            Error::Utf8Error(e) => write!(f, "{}", e),
            Error::TargetNotFound => write!(f, "Target Not Found"),
            Error::CannotParse => write!(f, "Cannot Parse"),
            Error::Lexer(e) => write!(f, "Error in Lexer: {}", e),
        }
    }
}
