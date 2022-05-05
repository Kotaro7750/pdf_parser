use super::Token;
use std::fmt;

pub enum Error {
    EmptyBuffer,
    UnexpectedByte(u8, char),
    UndefinedKeyword(String),
    ParseNumber(String),
    FinishInObject,
    CannotConfirmStream,
    InvalidObjectHead(u8),
    InvalidName(Vec<u8>),
    InvalidIndirectRef(Option<Token>, Option<Token>),
    InvalidIndirectObj(Option<Token>, Option<Token>),
}

impl Error {
    fn common_fmt(self: &Error, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::EmptyBuffer => write!(f, "Buffer is Empty"),
            Error::UnexpectedByte(byte, expected) => write!(
                f,
                "Encounter unexpected byte {}: Expected {}",
                byte, expected
            ),
            Error::UndefinedKeyword(string) => write!(f, "UndefinedKeyword {}", string),
            Error::ParseNumber(string) => write!(f, "Cannot parse '{}' as Number", string),
            Error::FinishInObject => write!(f, "Buffer is finished within object"),
            Error::CannotConfirmStream => write!(f, "Cannot confirm whether object is stream"),
            Error::InvalidObjectHead(byte) => {
                write!(f, "Encounter not object header byte {}", byte)
            }
            Error::InvalidName(vec) => {
                write!(f, "Cannot convert byte '{:?}' into valid Name", vec)
            }
            Error::InvalidIndirectRef(may_obj_num, may_gen_num) => {
                write!(
                    f,
                    "R keyword is used wrong context (Object number: {}, Generation Number: {})",
                    match may_obj_num {
                        Some(t) => format!("{:?}", t),
                        None => format!("None"),
                    },
                    match may_gen_num {
                        Some(t) => format!("{:?}", t),
                        None => format!("None"),
                    }
                )
            }
            Error::InvalidIndirectObj(may_obj_num, may_gen_num) => {
                write!(
                    f,
                    "obj keyword is used wrong context (Object number: {}, Generation Number: {})",
                    match may_obj_num {
                        Some(t) => format!("{:?}", t),
                        None => format!("None"),
                    },
                    match may_gen_num {
                        Some(t) => format!("{:?}", t),
                        None => format!("None"),
                    }
                )
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
