use std::fmt;

use crate::object;
use crate::parser::error as parser_error;
use crate::parser::Object;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    EOFNotFound,
    StartXRefNotFound,
    TrailerNotFound,
    ParseXRefOffset(parser_error::Error),
    XRefOffsetNotInteger(Object),
    ParseTrailerDict(parser_error::Error),
    Object(object::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::Io(e) => write!(f, "Error of IO: {}", e),
            Error::EOFNotFound => write!(f, "EOF marker is not found"),
            Error::StartXRefNotFound => write!(f, "startxref is not found"),
            Error::TrailerNotFound => write!(f, "trailer is not found"),
            Error::ParseXRefOffset(e) => write!(
                f,
                "Error on parsing byte offset of cross reference table: {}",
                e
            ),
            Error::XRefOffsetNotInteger(obj) => write!(f, "Object '{:?}' is not integer", obj),
            Error::ParseTrailerDict(e) => write!(f, "Error on parsing trailer dictionary: {}", e),
            Error::Object(e) => write!(f, "Error in object: {:?}", e),
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
