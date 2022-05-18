use std::cmp;
use std::fs::File;
use std::io::Read;
use std::io::Seek;

use crate::object;
use crate::parser;
use crate::raw_byte;

pub mod error;

pub struct Trailer {
    pub xref_start_offset: u64,
    xref_entry_num: object::PdfInteger,
    root_catalog_ref: object::PdfIndirectRef,
}

impl Trailer {
    pub fn get_root_catalog_ref(&self) -> object::PdfIndirectRef {
        self.root_catalog_ref.clone()
    }
}

pub fn parse_trailer(file: &mut File, filesize: u64) -> Result<Trailer, error::Error> {
    // 少なくともファイル末尾1024バイトにEOFマーカーが表れることは保証していい
    // cf. version1.7の仕様書 Appendix H の Implementation Note 18
    let mut buffer: [u8; 1024] = [0; 1024];
    let byte_offset = cmp::max(filesize, 1024) - 1024;

    file.seek(std::io::SeekFrom::Start(byte_offset))?;

    let n = file.read(&mut buffer)?;

    let buffer = &buffer[..n];
    let buffer = match raw_byte::cut_from(buffer, "%%EOF".as_bytes()) {
        Some(buffer) => buffer,
        None => return Err(error::Error::EOFNotFound),
    };

    let may_trailer_dict = parse_trailer_dict(buffer, byte_offset)?;
    let trailer_dict = object::PdfDict::ensure_with_key(&may_trailer_dict, vec!["Size", "Root"])?;

    let xref_entry_num = object::PdfInteger::ensure(trailer_dict.get("Size").unwrap())?.clone();
    let root_catalog_ref =
        object::PdfIndirectRef::ensure(trailer_dict.get("Root").unwrap())?.clone();

    let xref_start_offset = parse_xref_offset(buffer, byte_offset)?;

    Ok(Trailer {
        xref_start_offset,
        xref_entry_num,
        root_catalog_ref,
    })
}

fn parse_xref_offset(buffer: &[u8], byte_offset: u64) -> Result<u64, error::Error> {
    let startxref_bufer = match raw_byte::extract_tail_after(buffer, "startxref".as_bytes()) {
        Some(buffer) => buffer,
        None => return Err(error::Error::StartXRefNotFound),
    };
    // バッファの長さの差からバッファ先頭のファイル中バイトオフセットを計算する
    let startxref_byte_offset = (buffer.len() - startxref_bufer.len()) as u64 + byte_offset;

    let mut parser = match parser::Parser::new(startxref_bufer, startxref_byte_offset) {
        Ok(p) => p,
        Err(e) => return Err(error::Error::ParseXRefOffset(e)),
    };

    let xref_byte_offset = match parser.parse() {
        // XXX ファイルサイズを超えるオフセットが指定されたときの対処
        Ok(obj) => object::PdfInteger::ensure(&obj)?.clone(),
        Err(e) => return Err(error::Error::ParseXRefOffset(e)),
    };

    if xref_byte_offset.unpack() <= 0 {
        panic!()
    }

    Ok(u64::try_from(xref_byte_offset).unwrap())
}

fn parse_trailer_dict(buffer: &[u8], byte_offset: u64) -> Result<parser::Object, error::Error> {
    let trailer_dict_buffer = match raw_byte::extract_after(buffer, "trailer".as_bytes()) {
        Some(buffer) => buffer,
        None => return Err(error::Error::TrailerNotFound),
    };

    let trailer_dict_byte_offset = (buffer.len() - trailer_dict_buffer.len()) as u64 + byte_offset;

    let trailer_dict_buffer =
        match raw_byte::cut_tail_from(trailer_dict_buffer, "startxref".as_bytes()) {
            Some(buffer) => buffer,
            None => return Err(error::Error::StartXRefNotFound),
        };

    let mut parser = match parser::Parser::new(trailer_dict_buffer, trailer_dict_byte_offset) {
        Ok(p) => p,
        Err(e) => return Err(error::Error::ParseTrailerDict(e)),
    };

    match parser.parse() {
        Ok(obj) => Ok(obj),
        Err(e) => return Err(error::Error::ParseTrailerDict(e)),
    }
}
