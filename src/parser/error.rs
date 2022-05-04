use std::fmt;

use crate::lexer::error as lexer_error;

pub enum Error {
    EmptyBuffer,
    NoToken,
    UnexpectedToken,
    Lexer(lexer_error::Error),
}

impl Error {
    fn common_fmt(self: &Error, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::EmptyBuffer => write!(f, "Buffer is Empty"),
            Error::NoToken => write!(f, "Token is missing"),
            Error::UnexpectedToken => write!(f, "UnexpectedToken is found"),
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
