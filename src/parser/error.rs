use std::fmt;

use crate::lexer::error as lexer_error;
use crate::lexer::Token;

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
        write!(f, "{} at byte offset `{}`", self.kind, self.byte_offset)
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum ErrorKind {
    NoToken,
    IndirectObjMissMatch,
    UnexpectedToken(Token),
    InvalidStreamObj,
    Lexer(lexer_error::Error),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ErrorKind::NoToken => write!(f, "token is missing"),
            ErrorKind::IndirectObjMissMatch => write!(f, "keyword obj and endobj is not matched"),
            ErrorKind::UnexpectedToken(token) => write!(f, "unexpected token found `{}`", token),
            ErrorKind::InvalidStreamObj => write!(f, "invalid stream object"),
            ErrorKind::Lexer(e) => write!(f, "cannot tokenize: {}", e),
        }
    }
}
