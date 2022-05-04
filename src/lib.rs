use std::fs::File;

mod error;
mod header;
mod lexer;
mod parser;
mod raw_byte;
mod trailer;

pub struct PDF<'a> {
    file: &'a mut File,
    size: u64,
}

impl<'a> PDF<'a> {
    pub fn new(file: &'a mut File) -> Result<PDF<'a>, error::Error> {
        let size = PDF::get_file_size(file)?;

        let is_pdf = header::expect_pdf(file)?;

        let trailer = trailer::parse_trailer(file, size)?;

        Ok(PDF {
            file: file,
            size: size,
        })
    }

    fn get_file_size(file: &File) -> Result<u64, std::io::Error> {
        Ok(file.metadata()?.len())
    }
}
