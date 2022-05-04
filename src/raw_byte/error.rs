use std::fmt;

pub enum Error {
    EmptyBuffer,
    EOLNotFound,
    TargetNotFound(Vec<u8>),
}

impl Error {
    fn common_fmt(self: &Error, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::EmptyBuffer => write!(f, "Buffer is Empty"),
            Error::EOLNotFound => write!(f, "EOL is not found in buffer"),
            Error::TargetNotFound(str) => write!(f, "Target '{:?}' not found in buffer", str),
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
