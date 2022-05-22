use crate::header;
use crate::object;
use crate::page;
use crate::trailer::error as trailer_error;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Header(header::Error),
    Trailer(trailer_error::Error),
    PageTree(page::Error),
    Object(object::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<trailer_error::Error> for Error {
    fn from(e: trailer_error::Error) -> Self {
        Self::Trailer(e)
    }
}

impl From<header::Error> for Error {
    fn from(e: header::Error) -> Self {
        Self::Header(e)
    }
}

impl From<object::Error> for Error {
    fn from(e: object::Error) -> Self {
        Self::Object(e)
    }
}

impl From<page::Error> for Error {
    fn from(e: page::Error) -> Self {
        Self::PageTree(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::Header(e) => write!(f, "header error: {}", e),
            Error::Trailer(e) => write!(f, "error on trailer: {}", e),
            Error::PageTree(e) => write!(f, "Error on Page Tree: {:?}", e),
            Error::Object(e) => write!(f, "Error on Parsing Object: {}", e),
        }
    }
}
