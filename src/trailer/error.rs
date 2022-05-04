use std::fmt;

use crate::parser::error as parser_error;
use crate::parser::Object;
use crate::raw_byte::error as raw_byte_error;

pub enum Error {
    EmptyBuffer,
    Io(std::io::Error),
    TargetNotFound(Vec<u8>),
    ParseXRefOffset(parser_error::Error),
    XRefOffsetNotInteger(Object),
    ParseTrailerDict(parser_error::Error),
    TrailerDictNotDict(Object),
    InvalidTrailerDict(String),
}

impl Error {
    fn common_fmt(self: &Error, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::EmptyBuffer => write!(f, "Buffer is empty"),
            Error::Io(e) => write!(f, "Error of IO: {}", e),
            Error::TargetNotFound(str) => write!(f, "Target '{:?}' not found in buffer", str),
            Error::ParseXRefOffset(e) => write!(
                f,
                "Error on parsing byte offset of cross reference table: {}",
                e
            ),
            Error::XRefOffsetNotInteger(obj) => write!(f, "Object '{:?}' is not integer", obj),
            Error::ParseTrailerDict(e) => write!(f, "Error on parsing trailer dictionary: {}", e),
            Error::TrailerDictNotDict(obj) => write!(f, "Object '{:?}' is not dictionary", obj),
            Error::InvalidTrailerDict(str) => {
                write!(f, "Trailer dictionary must contain '{}'", str)
            }
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

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<raw_byte_error::Error> for Error {
    fn from(e: raw_byte_error::Error) -> Self {
        match e {
            raw_byte_error::Error::EmptyBuffer => Self::EmptyBuffer,
            raw_byte_error::Error::TargetNotFound(vec) => Self::TargetNotFound(vec),
        }
    }
}
