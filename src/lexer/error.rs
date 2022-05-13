use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedByte,
    UndefinedKeyword,
    FinishInObject,
    ConfirmStream,
    ParseNumber,
    ParseName,
    ParseHexString,
    InvalidIndirectRef,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::UnexpectedByte => write!(f, "encounter unexpected byte"),
            Self::UndefinedKeyword => write!(f, "encounter undefined keyword"),
            Self::FinishInObject => write!(f, "buffer terminated in object"),
            Self::ConfirmStream => write!(
                f,
                "buffer terminated without confirming whether stream object"
            ),
            Self::ParseNumber => write!(f, "cannot parse as number"),
            Self::ParseName => write!(f, "cannot parse as name"),
            Self::ParseHexString => write!(f, "cannot parse as hex string"),
            Self::InvalidIndirectRef => write!(f, "encounter invalid indirect reference"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
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
