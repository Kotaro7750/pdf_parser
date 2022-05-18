use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::str;

use crate::raw_byte;
use crate::trailer;

pub struct XRef {
    pub actual_start_offset: u64,
    pub from: u64,
    pub entry_num: u64,
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
        let from = u64::from_str_radix(str::from_utf8(from_buffer).unwrap(), 10).unwrap();

        // エントリ数のすぐ後にEOLが来るのでそのEOLから始まるバッファとの長さの差からエントリ数のバイト数を計算する
        let buffer = raw_byte::extract_after(buffer, " ".as_bytes()).unwrap();
        let start_at_eol = raw_byte::extract_from_eol(buffer).unwrap();
        let entry_num_len = buffer.len() - start_at_eol.len();

        // エントリ数をパースする
        let entry_num_buffer = &buffer[..entry_num_len];
        let entry_num = u64::from_str_radix(str::from_utf8(entry_num_buffer).unwrap(), 10).unwrap();

        // 2行の読み飛ばしが何バイトの読み飛ばしに相当するのかを計算しエントリが始まるバイトオフセットを計算する
        let eol_skipped = raw_byte::extract_after_eol(start_at_eol).unwrap();
        let actual_start_offset = (n - eol_skipped.len()) as u64 + trailer_dict.xref_start_offset;

        XRef {
            actual_start_offset,
            from,
            entry_num,
        }
    }

    fn parse_entry(buffer: [u8; 18]) -> (u64, u64, bool) {
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
        if (obj_num as u64) < self.from || (self.from + self.entry_num) <= obj_num as u64 {
            panic!("object is not in cross reference");
        }

        // 1エントリはきっかり20バイトである
        let byte_offset = self.actual_start_offset + ((obj_num as u64 - self.from) * 20) as u64;

        let mut buffer: [u8; 18] = [0; 18];

        file.seek(SeekFrom::Start(byte_offset)).unwrap();

        if file.read(&mut buffer).unwrap() != 18 {
            panic!("cannot read 18 byte");
        };

        let (offset, gen, is_n) = Self::parse_entry(buffer);

        if gen != gen_num as u64 {
            panic!("generation number mismatch");
        }

        offset
    }
}
