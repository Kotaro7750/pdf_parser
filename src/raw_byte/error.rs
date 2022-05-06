use std::fmt;

#[derive(Debug)]
pub enum Error {
    EmptyBuffer,
    EOLNotFound,
    TargetNotFound(Vec<u8>),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::EmptyBuffer => write!(f, "buffer is empty"),
            Error::EOLNotFound => write!(f, "EOL marker is not found in buffer"),
            Error::TargetNotFound(str) => write!(f, "`{:?}` not found in buffer", str),
        }
    }
}
