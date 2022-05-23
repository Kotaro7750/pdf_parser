use std::fmt;

use crate::lexer::error as lexer_error;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    byte_offset: u64,
}

impl Error {
    pub fn new(kind: ErrorKind, byte_offset: u64) -> Error {
        Error { kind, byte_offset }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.kind {
            ErrorKind::Lexer(_) => write!(f, "{}", self.kind),
            _ => write!(f, "{} at byte offset `{}`", self.kind, self.byte_offset),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum ErrorKind {
    NoToken,
    IndirectObjMissMatch,
    UnexpectedToken,
    InvalidStreamObj,
    Lexer(lexer_error::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ErrorKind::NoToken => write!(f, "token is missing"),
            ErrorKind::IndirectObjMissMatch => write!(f, "keyword obj and endobj is not matched"),
            ErrorKind::UnexpectedToken => write!(f, "unexpected token found"),
            ErrorKind::InvalidStreamObj => write!(f, "invalid stream object"),
            ErrorKind::Lexer(e) => write!(f, "cannot tokenize: {}", e),
        }
    }
}
