use regex::Regex;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::str;

pub enum Error {
    Io(std::io::Error),
    EOLNotEncounter(Vec<u8>),
    ConvertUTF8(Vec<u8>),
    NotHeader(String),
}

impl Error {
    fn common_fmt(self: &Error, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::Io(e) => write!(f, "Error on IO: {}", e),
            Error::EOLNotEncounter(vec) => write!(f, "Not encountered EOL marker in {:?}", vec),
            Error::ConvertUTF8(vec) => write!(f, "Cannot convert to utf8 from {:?}", vec),
            Error::NotHeader(str) => write!(f, "'{}' is not valid pdf header", str),
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

pub fn expect_pdf(file: &mut File) -> Result<(), Error> {
    const PDF_HEADER_MAX_LENGTH: usize = 15;

    const SPACE_ASCII_CODE: u8 = 32;
    const LF_ASCII_CODE: u8 = 10;
    const CR_ASCII_CODE: u8 = 13;

    let mut buffer = [0; PDF_HEADER_MAX_LENGTH];

    let n: usize = file.read(&mut buffer)?;

    let mut space_i = n - 1;
    for i in 0..=(n - 1) {
        if buffer[i] == LF_ASCII_CODE || buffer[i] == CR_ASCII_CODE {
            space_i = i;
            break;
        }
    }

    if space_i == n - 1 {
        return Err(Error::EOLNotEncounter(buffer.to_vec()));
    }

    let buffer = &buffer[..space_i];

    let may_version = match str::from_utf8(buffer) {
        Ok(str) => str,
        Err(_) => return Err(Error::ConvertUTF8(buffer.to_vec())),
    };

    let re = Regex::new(r"%PDF-\d+\.\d+").unwrap();

    if re.is_match(may_version) {
        Ok(())
    } else {
        Err(Error::NotHeader(String::from(may_version)))
    }
}
