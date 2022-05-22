use std::fs::File;
use std::str;

use crate::object;
use crate::parser;
use crate::raw_byte;
use crate::util::read_partially;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    XrefNotFound,
    SubsectionNotFound,
    Parser(parser::error::Error),
    Object(object::Error),
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<parser::error::Error> for Error {
    fn from(e: parser::error::Error) -> Self {
        Self::Parser(e)
    }
}
impl From<object::Error> for Error {
    fn from(e: object::Error) -> Self {
        Self::Object(e)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Io(e) => write!(f, "io: {}", e),
            Self::XrefNotFound => write!(f, "xref is not found"),
            Self::SubsectionNotFound => {
                write!(f, "subsection line is not found")
            }
            Self::Parser(e) => write!(f, "parser: {}", e),
            Self::Object(e) => write!(f, "object: {}", e),
        }
    }
}

pub struct XRef {
    actual_start_offset: u64,
    from: usize,
    entry_num: usize,
}

impl XRef {
    pub fn new(file: &mut File, xref_start_offset: u64) -> Result<Self, Error> {
        // 30バイトあればxrefキーワードとヘッダ行を読み込めるという見込み
        let buffer = read_partially(file, xref_start_offset, 30)?;
        let buffer = buffer.as_slice();
        let n = buffer.len();

        let buffer = Self::extract_after_xref_line(buffer)?;
        let xref_line_length = n - buffer.len();

        let (from, entry_num) =
            Self::parse_subsection_line(buffer, xref_start_offset + xref_line_length as u64)?;

        let buffer = raw_byte::extract_after_eol(buffer).unwrap();
        let actual_start_offset = xref_start_offset + (n - buffer.len()) as u64;

        Ok(XRef {
            actual_start_offset,
            from,
            entry_num,
        })
    }

    fn extract_after_xref_line(buffer: &[u8]) -> Result<&[u8], Error> {
        let buffer = match raw_byte::extract_after(buffer, "xref".as_bytes()) {
            Some(buffer) => buffer,
            None => return Err(Error::XrefNotFound),
        };

        match raw_byte::extract_after_eol(buffer) {
            Some(buffer) => Ok(buffer),
            None => Err(Error::XrefNotFound),
        }
    }

    fn parse_subsection_line(buffer: &[u8], byte_offset: u64) -> Result<(usize, usize), Error> {
        let subsection_line = match raw_byte::cut_after_eol(buffer) {
            Some(buf) => buf,
            None => return Err(Error::SubsectionNotFound),
        };

        let from = Self::parse_subsection_from(subsection_line, byte_offset)?;
        let object_num = Self::parse_subsection_object_num(subsection_line, byte_offset)?;

        Ok((from, object_num))
    }

    fn parse_subsection_from(
        subsection_line: &[u8],
        subsection_start_byte_offset: u64,
    ) -> Result<usize, Error> {
        let from_buffer = match raw_byte::cut_from(subsection_line, " ".as_bytes()) {
            Some(buf) => buf,
            None => return Err(Error::SubsectionNotFound),
        };

        let mut p = parser::Parser::new(from_buffer, subsection_start_byte_offset)?;
        let from_obj = p.parse()?;

        let from = object::PdfInteger::ensure(&from_obj)?;
        from.assert_not_negative()?;

        Ok(from.unpack() as usize)
    }

    fn parse_subsection_object_num(
        subsection_line: &[u8],
        subsection_start_byte_offset: u64,
    ) -> Result<usize, Error> {
        let object_num_buffer = match raw_byte::extract_after(subsection_line, " ".as_bytes()) {
            Some(buf) => buf,
            None => return Err(Error::SubsectionNotFound),
        };
        let object_num_byte_offset =
            subsection_start_byte_offset + (subsection_line.len() - object_num_buffer.len()) as u64;

        let mut p = parser::Parser::new(object_num_buffer, object_num_byte_offset)?;
        let object_num_obj = p.parse()?;

        let object_num = object::PdfInteger::ensure(&object_num_obj)?;
        object_num.assert_natural()?;

        Ok(object_num.unpack() as usize)
    }

    fn parse_entry(buffer: &[u8]) -> (u64, u64, bool) {
        if buffer.len() != 18 {
            panic!("cross reference entry must be 18 byte");
        }
        let n_buf = &buffer[..10];
        let g_buf = &buffer[11..16];
        let t_byte = buffer[17];

        (
            u64::from_str_radix(str::from_utf8(n_buf).unwrap(), 10).unwrap(),
            u64::from_str_radix(str::from_utf8(g_buf).unwrap(), 10).unwrap(),
            match t_byte {
                110 => true,
                103 => false,
                _ => panic!("{} is not supported type", t_byte),
            },
        )
    }

    pub fn get_object_byte_offset(&self, file: &mut File, obj_num: usize, gen_num: usize) -> u64 {
        if obj_num < self.from || (self.from + self.entry_num) <= obj_num {
            panic!("object is not in cross reference");
        }

        // 1エントリはきっかり20バイトである
        let byte_offset = self.actual_start_offset + ((obj_num - self.from) * 20) as u64;

        let buffer = read_partially(file, byte_offset, 18).unwrap();
        if buffer.len() != 18 {
            panic!("cannot read 18 byte");
        };
        let buffer = buffer.as_slice();

        let (offset, gen, is_n) = Self::parse_entry(buffer);

        if gen != gen_num as u64 {
            panic!("generation number mismatch");
        }

        offset
    }
}
