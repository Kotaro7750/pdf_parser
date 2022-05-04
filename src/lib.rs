use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

mod cross_reference;
mod error;
mod header;
mod lexer;
mod parser;
mod raw_byte;
mod trailer;

pub struct PDF<'a> {
    file: &'a mut File,
    size: u64,
    trailer: trailer::Trailer,
    xref: cross_reference::XRef,
}

impl<'a> PDF<'a> {
    pub fn new(file: &'a mut File) -> Result<PDF<'a>, error::Error> {
        let size = PDF::get_file_size(file)?;

        let is_pdf = header::expect_pdf(file)?;

        let trailer = trailer::parse_trailer(file, size)?;

        let mut xref = cross_reference::XRef::new(file, &trailer);

        Ok(PDF {
            file: file,
            size: size,
            trailer,
            xref,
        })
    }

    fn get_file_size(file: &File) -> Result<u64, std::io::Error> {
        Ok(file.metadata()?.len())
    }

    pub fn get_indirect_obj(&mut self) -> Result<parser::Object, error::Error> {
        let offset = self.xref.get_object_byte_offset(self.file, 10, 0);

        self.file.seek(SeekFrom::Start(offset));

        let mut buffer: [u8; 100] = [0; 100];

        let n = self.file.read(&mut buffer)?;

        println!("{:?}", buffer);

        let mut p = parser::Parser::new(&buffer).unwrap();
        let obj = p.parse().unwrap();

        println!("{:?}", obj);

        Ok(obj)
    }
}
