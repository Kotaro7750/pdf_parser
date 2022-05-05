use std::fs::File;
use std::io::Read;
use std::io::Seek;

use crate::object;
use crate::parser;
use crate::raw_byte;

pub mod error;

pub struct Trailer {
    pub xref_start_offset: u64,
    xref_entry_num: u64,
    root_catalog_ref: (u64, u64),
}

impl Trailer {
    pub fn get_root_catalog_ref(&self) -> (u64, u64) {
        (self.root_catalog_ref.0, self.root_catalog_ref.1)
    }
}

pub fn parse_trailer(file: &mut File, filesize: u64) -> Result<Trailer, error::Error> {
    // 少なくともファイル末尾1024バイトにEOFマーカーが表れることは保証していい
    // cf. version1.7の仕様書 Appendix H の Implementation Note 18
    let mut buffer: [u8; 1024] = [0; 1024];

    file.seek(std::io::SeekFrom::Start(filesize - 1024))?;

    let n = file.read(&mut buffer)?;

    let buffer = &buffer[..n];
    let buffer = raw_byte::cut_from(buffer, "%%EOF".as_bytes())?;

    let startxref_bufer = raw_byte::extract_tail_after(buffer, "startxref".as_bytes())?;

    // 相互参照テーブルの開始オフセットを取得
    let mut parser = match parser::Parser::new(startxref_bufer) {
        Ok(p) => p,
        Err(e) => return Err(error::Error::ParseXRefOffset(e)),
    };

    let xref_offset = match parser.parse() {
        Ok(parser::Object::Integer(int)) if int > 0 => int,
        Ok(obj) => {
            return Err(error::Error::XRefOffsetNotInteger(obj));
        }
        Err(e) => return Err(error::Error::ParseXRefOffset(e)),
    } as u64;

    // トレーラ辞書を取得
    let trailer_dict_buffer = raw_byte::extract_after(buffer, "trailer".as_bytes())?;
    let trailer_dict_buffer = raw_byte::cut_tail_from(trailer_dict_buffer, "startxref".as_bytes())?;

    let mut parser = match parser::Parser::new(trailer_dict_buffer) {
        Ok(p) => p,
        Err(e) => return Err(error::Error::ParseTrailerDict(e)),
    };

    let trailer_dict = match parser.parse() {
        Ok(obj) => obj,
        Err(e) => return Err(error::Error::ParseTrailerDict(e)),
    };

    let trailer_dict = object::ensure_dict_with_key(&trailer_dict, vec!["Size", "Root"])?;

    let xref_entry_num = match trailer_dict.get("Size").unwrap() {
        parser::Object::Integer(int) if *int > 0 => *int,
        _ => return Err(object::Error::ObjectRestriction(String::from("Size")))?,
    } as u64;

    let root_catalog_ref = match trailer_dict.get("Root").unwrap() {
        parser::Object::IndirectRef(obj_num, gen_num) => (*obj_num, *gen_num),
        _ => return Err(object::Error::ObjectRestriction(String::from("Root")))?,
    };

    Ok(Trailer {
        xref_start_offset: xref_offset,
        xref_entry_num,
        root_catalog_ref,
    })
}
