use regex::Regex;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::str;

use crate::raw_byte;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidHeader(Vec<u8>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::Io(e) => write!(f, "io error: {}", e),
            Error::InvalidHeader(_) => write!(f, "invalid pdf header"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub fn validate_pdf_header(file: &mut File) -> Result<(), Error> {
    const PDF_HEADER_MAX_LENGTH: usize = 15;

    let mut buffer = [0; PDF_HEADER_MAX_LENGTH];
    let n: usize = file.read(&mut buffer)?;
    let buffer = &buffer[..n];

    let buffer = match raw_byte::cut_after_eol(buffer) {
        Some(buffer) => buffer,
        None => return Err(Error::InvalidHeader(buffer.to_vec())),
    };

    let may_version = match str::from_utf8(buffer) {
        Ok(str) => str,
        Err(_) => return Err(Error::InvalidHeader(buffer.to_vec())),
    };

    let re = Regex::new(r"%PDF-\d+\.\d+").unwrap();

    if re.is_match(may_version) {
        Ok(())
    } else {
        Err(Error::InvalidHeader(buffer.to_vec()))
    }
}
