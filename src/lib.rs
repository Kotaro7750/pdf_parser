use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::str;

mod error;
mod lexer;
mod parser;
mod raw_byte;

pub struct PDF<'a> {
    file: &'a mut File,
    size: u64,
}

pub struct Trailer {
    xref_start_offset: isize,
}

impl<'a> PDF<'a> {
    pub fn new(file: &'a mut File) -> Result<PDF<'a>, error::Error> {
        let size = PDF::get_file_size(file)?;

        let is_pdf = PDF::check_if_pdf(file)?;

        if !is_pdf {
            return Err(error::Error::NotPDF);
        }

        PDF::parse_trailer(file, size);

        Ok(PDF {
            file: file,
            size: size,
        })
    }

    fn get_file_size(file: &File) -> Result<u64, std::io::Error> {
        match file.metadata() {
            Ok(metadata) => Ok(metadata.len()),
            Err(err) => Err(err),
        }
    }

    fn check_if_pdf(file: &mut File) -> Result<bool, std::io::Error> {
        const PDF_HEADER_MAX_LENGTH: usize = 15;

        const SPACE_ASCII_CODE: u8 = 32;
        const LF_ASCII_CODE: u8 = 10;
        const CR_ASCII_CODE: u8 = 13;

        let mut buffer = [0; PDF_HEADER_MAX_LENGTH];

        let n: usize = match file.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "File cannot read",
                ))
            }
        };

        let mut space_i = n - 1;
        for i in 0..=(n - 1) {
            if (buffer[i] == SPACE_ASCII_CODE && buffer[i + 1] == LF_ASCII_CODE)
                || (buffer[i] == CR_ASCII_CODE && buffer[i + 1] == LF_ASCII_CODE)
            {
                space_i = i;
                break;
            }
        }

        if space_i == n - 1 {
            return Ok(false);
        }

        let may_version = match str::from_utf8(&buffer[..space_i]) {
            Ok(str) => str,
            Err(_) => return Ok(false),
        };

        let re = Regex::new(r"%PDF-\d+\.\d+").unwrap();

        Ok(re.is_match(may_version))
    }

    fn parse_trailer(file: &mut File, filesize: u64) -> Result<Trailer, error::Error> {
        // 少なくともファイル末尾1024バイトにEOFマーカーが表れることは保証していい
        // cf. version1.7の仕様書 Appendix H の Implementation Note 18
        let mut buffer: [u8; 1024] = [0; 1024];

        file.seek(std::io::SeekFrom::Start(filesize - 1024))?;

        let n = file.read(&mut buffer)?;

        let buffer = &buffer[..n];
        let buffer = raw_byte::cut_from(buffer, "%%EOF".as_bytes())?;

        println!("{:?}", buffer);

        Err(error::Error::NotPDF)
    }
}
