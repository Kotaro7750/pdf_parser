use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::str;

use crate::raw_byte;
use crate::trailer;

pub struct XRef {
    pub actual_start_offset: u64,
    pub from: usize,
    pub entry_num: usize,
}

impl XRef {
    pub fn new(file: &mut File, trailer_dict: &trailer::Trailer) -> XRef {
        // TODO なぜ30?
        let mut buffer: [u8; 30] = [0; 30];

        file.seek(SeekFrom::Start(trailer_dict.xref_start_offset))
            .unwrap();

        let n = file.read(&mut buffer).unwrap();

        let buffer = &buffer[..n];

        // xrefキーワードが書いてある行を読み飛ばす
        let buffer = raw_byte::extract_after(buffer, "xref".as_bytes()).unwrap();
        let buffer = raw_byte::extract_after_eol(buffer).unwrap();

        // 何番目のオブジェクトのエントリから始まるかをパースする
        let from_buffer = raw_byte::cut_from(buffer, " ".as_bytes()).unwrap();
        let from = usize::from_str_radix(str::from_utf8(from_buffer).unwrap(), 10).unwrap();

        // エントリ数のすぐ後にEOLが来るのでそのEOLから始まるバッファとの長さの差からエントリ数のバイト数を計算する
        let buffer = raw_byte::extract_after(buffer, " ".as_bytes()).unwrap();
        let start_at_eol = raw_byte::extract_from_eol(buffer).unwrap();
        let entry_num_len = buffer.len() - start_at_eol.len();

        // エントリ数をパースする
        let entry_num_buffer = &buffer[..entry_num_len];
        let entry_num =
            usize::from_str_radix(str::from_utf8(entry_num_buffer).unwrap(), 10).unwrap();

        // 2行の読み飛ばしが何バイトの読み飛ばしに相当するのかを計算しエントリが始まるバイトオフセットを計算する
        let eol_skipped = raw_byte::extract_after_eol(start_at_eol).unwrap();
        let actual_start_offset = (n - eol_skipped.len()) as u64 + trailer_dict.xref_start_offset;

        XRef {
            actual_start_offset,
            from,
            entry_num,
        }
    }
}
