use super::Token;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    EmptyBuffer,
    UnexpectedByte(u8, char),
    UndefinedKeyword(Vec<u8>),
    FinishInObject,
    CannotConfirmStream,
    ParseNumber(String),
    ParseName(Vec<u8>),
    ParseHexString(Vec<u8>),
    InvalidIndirectRef(Option<Token>, Option<Token>),
    InvalidIndirectObj(Option<Token>, Option<Token>),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::EmptyBuffer => write!(f, "buffer is empty"),
            Error::UnexpectedByte(byte, expected) => {
                write!(f, "unexpected byte `{}`: expect `{}`", byte, expected)
            }
            Error::UndefinedKeyword(vec) => write!(f, "undefined keyword `{:?}`", vec),
            Error::ParseNumber(string) => write!(f, "cannot parse `{}` as number", string),
            Error::FinishInObject => write!(f, "buffer is finished within object"),
            Error::CannotConfirmStream => {
                write!(f, "cannot confirm stream buffer start in this buffer range")
            }
            Error::ParseName(vec) => {
                write!(f, "cannot parse byte `{:?}` into valid name", vec)
            }
            Error::ParseHexString(vec) => {
                write!(f, "cannot parse byte `{:?}` into hex string", vec)
            }
            Error::InvalidIndirectRef(may_obj_num, may_gen_num) => {
                write!(
                    f,
                    "R keyword is used wrong context (object number: `{}`, generation number: `{}`)",
                    match may_obj_num {
                        Some(t) => format!("{:?}", t),
                        None => format!("none"),
                    },
                    match may_gen_num {
                        Some(t) => format!("{:?}", t),
                        None => format!("none"),
                    }
                )
            }
            Error::InvalidIndirectObj(may_obj_num, may_gen_num) => {
                write!(
                    f,
                    "obj keyword is used wrong context (object number: `{}`, generation number: `{}`)",
                    match may_obj_num {
                        Some(t) => format!("{:?}", t),
                        None => format!("none"),
                    },
                    match may_gen_num {
                        Some(t) => format!("{:?}", t),
                        None => format!("none"),
                    }
                )
            }
        }
    }
}
