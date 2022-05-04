use std::fmt;

use crate::lexer::error as lexer_error;
use crate::lexer::Token;
use crate::parser::Object;

pub enum Error {
    EmptyBuffer,
    NoToken,
    IndirectObjMissMatch,
    NotIndirectObj(Object),
    UnexpectedToken(Token),
    Lexer(lexer_error::Error),
}

impl Error {
    fn common_fmt(self: &Error, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::EmptyBuffer => write!(f, "Buffer is Empty"),
            Error::NoToken => write!(f, "Token is missing"),
            Error::NotIndirectObj(obj) => write!(f, "{:?} is not indirect obj", obj),
            Error::IndirectObjMissMatch => write!(f, "keyword obj and endobj is not matched"),
            Error::UnexpectedToken(token) => write!(f, "UnexpectedToken is found: {:?}", token),
            Error::Lexer(e) => write!(f, "Error in Lexer: {}", e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.common_fmt(f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.common_fmt(f)
    }
}
