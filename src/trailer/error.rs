use std::fmt;

use crate::object;
use crate::parser::error as parser_error;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    EOFNotFound,
    StartXRefNotFound,
    TrailerNotFound,
    ParseXRefOffset(parser_error::Error),
    ParseTrailerDict(parser_error::Error),
    Object(object::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::Io(e) => write!(f, "io: {}", e),
            Error::EOFNotFound => write!(f, "EOF marker is not found"),
            Error::StartXRefNotFound => write!(f, "startxref is not found"),
            Error::TrailerNotFound => write!(f, "trailer is not found"),
            Error::ParseXRefOffset(e) => {
                write!(f, "parse byte offset of cross reference table: {}", e)
            }
            Error::ParseTrailerDict(e) => write!(f, "parse trailer dictionary: {}", e),
            Error::Object(e) => write!(f, "object: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<object::Error> for Error {
    fn from(e: object::Error) -> Self {
        Self::Object(e)
    }
}
