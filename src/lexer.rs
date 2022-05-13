use std::str;
use std::str::FromStr;

use crate::raw_byte;
use error::{Error, ErrorKind};

pub mod error;
#[cfg(test)]
mod test;

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    EOL,
    Boolean(bool),
    Integer(isize),
    Real(f64),
    HexStr(Vec<u8>),
    String(Vec<u8>),
    Name(String),
    DictStart,
    DictEnd,
    ArrayStart,
    ArrayEnd,
    Null,
    IndirectRef(u64, u64),
    IndirectObjStart(u64, u64),
    IndirectObjEnd,
    // streamキーワードが表れたらそこで字句解析を終了するのでEndはない
    // Streamバイト列が始まるバイトオフセット
    StreamObjStart(u64),
}

fn parse_name(buffer: &[u8]) -> Result<String, ()> {
    if !buffer.is_ascii() {
        Err(())
    } else {
        // 全部ASCIIなのでunwrapしても問題ない
        Ok(String::from_utf8(buffer.to_vec()).unwrap())
    }
}

// 並んだバイト列をエスケープシーケンスを解釈したバイト列にして返す
fn parse_string(buffer: &[u8]) -> Result<Vec<u8>, Error> {
    let mut vec: Vec<u8> = vec![];
    let mut i = 0;

    let mut octal_string = String::new();
    let mut is_in_octal = false;

    let mut prev_backslash = false;

    while i < buffer.len() {
        let byte = buffer[i];

        if is_in_octal {
            // 3桁既に読んでいたり8進数文字以外の文字が出てきたら確定させる
            if octal_string.len() == 3 || !(0x30 <= byte && byte <= 0x37) {
                let octal_value = u16::from_str_radix(octal_string.as_str(), 8).unwrap();
                let octal_value: u8 = octal_value.to_be_bytes()[1];

                vec.push(octal_value);

                is_in_octal = false;
                octal_string = String::new();
            } else {
                octal_string.push_str(str::from_utf8(&buffer[i..(i + 1)]).unwrap());
                i += 1;
                continue;
            }
        }

        if prev_backslash {
            match byte {
                // nならLF
                0x6e => vec.push(0x0a),
                // rならCR
                0x72 => vec.push(0x0d),
                // tならTAB
                0x74 => vec.push(0x0b),
                // bならBS
                0x62 => vec.push(0x08),
                // fならFF
                0x66 => vec.push(0x0c),
                // (なら(
                0x28 => vec.push(0x28),
                // )なら)
                0x29 => vec.push(0x29),
                // \なら\
                0x5c => vec.push(0x5c),
                // 数字なら8進数として解釈し始める
                0x30..=0x37 => {
                    is_in_octal = true;
                    octal_string.push_str(str::from_utf8(&buffer[i..(i + 1)]).unwrap());
                }
                // それ以外ならバックスラッシュは無視する
                _ => vec.push(byte),
            }

            prev_backslash = false;
            i += 1;
            continue;
        }

        // バックスラッシュが表れたときには適切にエスケープする必要がある
        if byte == 0x5c {
            prev_backslash = true;
            i += 1;
            continue;
        }

        vec.push(byte);
        i += 1;
        continue;
    }

    Ok(vec)
}

// 並んだバイト列を16進数文字列の1桁とみなし，2桁をまとめて1バイトの16進数として解釈した数列を返す
// 奇数桁しかない場合には最後に暗黙の0を補う
fn parse_hex_string(buffer: &[u8]) -> Result<Vec<u8>, ()> {
    let mut vec: Vec<u8> = vec![];
    let mut i = 0;

    let mut hex_string: String = String::new();

    while i < buffer.len() {
        let byte = buffer[i];
        if !byte.is_ascii_hexdigit() {
            return Err(());
        }

        // byteをASCII文字と見て文字（列）にする
        // ASCII文字であることは確定しているのでunwrapしていい
        let str_slice = str::from_utf8(&buffer[i..i + 1]).unwrap();

        if i % 2 == 0 {
            hex_string = String::from(str_slice);
        } else {
            hex_string.push_str(str_slice);
            // 16進数文字列であることは確定しているのでunwrapしていい
            vec.push(u8::from_str_radix(hex_string.as_str(), 16).unwrap());
            hex_string = String::new();
        }

        i += 1;
    }

    // 奇数桁だったら最終桁に0を補う
    if buffer.len() % 2 == 1 {
        hex_string.push_str("0");
        vec.push(u8::from_str_radix(hex_string.as_str(), 16).unwrap());
    }

    Ok(vec)
}

// PDFオブジェクトの字句解析を行う
pub struct Lexer<'a> {
    buffer: &'a [u8],
    buffer_start_offset: u64,
    i: usize,
    token_head_i: usize,
    byte: u8,
    char: char,
    has_indirect_obj_start: bool,
    has_indirect_obj_end: bool,
    pub token_vec: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(buffer: &[u8], buffer_start_offset: u64) -> Lexer {
        if buffer.len() == 0 {
            panic!("buffer is empty");
        }

        Lexer {
            buffer,
            buffer_start_offset,
            i: 0,
            token_head_i: 0,
            byte: buffer[0],
            char: char::from(buffer[0]),
            has_indirect_obj_start: false,
            has_indirect_obj_end: false,
            token_vec: vec![],
        }
    }

    fn move_next_byte(&mut self) -> bool {
        self.i += 1;

        if self.buffer.len() <= self.i {
            false
        } else {
            self.byte = self.buffer[self.i];
            self.char = char::from(self.byte);

            true
        }
    }

    // tokenをトークン列に加えトークン先頭カーソルを移動させる
    fn confirm_token(&mut self, token: Token) {
        self.token_vec.push(token);
        self.token_head_i = self.i;
    }

    // 現在のトークンを無視してトークン先頭カーソルを移動させる
    fn skip_token(&mut self) {
        self.token_head_i = self.i;
    }

    fn is_number_char(&self) -> bool {
        match self.char {
            '0'..='9' | '+' | '-' | '.' => true,
            _ => false,
        }
    }

    fn is_regular_char(&self) -> bool {
        match self.char {
            '\0' | '\t' | '\n' | '\x12' | '\r' | ' ' | '(' | ')' | '<' | '>' | '[' | ']' | '{'
            | '}' | '/' | '%' => false,
            _ => true,
        }
    }

    // カーソル下のバイト列がtargetバイト列と一致するならカーソルをtargetの最後まで移動させる
    // 一致しないなら何もしない
    fn assume_and_move(&mut self, target: &[u8]) -> bool {
        if target.len() == 0 {
            return false;
        }

        for i in 0..target.len() {
            if self.buffer.len() <= (self.i + i) {
                return false;
            }

            if self.buffer[self.i + i] != target[i] {
                return false;
            }
        }

        self.i += target.len() - 1;
        true
    }

    // 現在キーワード末尾を指しているカーソルの次の文字がbuffer終端やデリミタであることを期待
    fn expect_keyword_end(&mut self) -> Result<(), Error> {
        if self.move_next_byte() && self.is_regular_char() {
            Err(self.construct_error(ErrorKind::UndefinedKeyword))
        } else {
            Ok(())
        }
    }

    fn construct_error(&self, kind: ErrorKind) -> Error {
        Error::new(kind, self.buffer_start_offset + self.token_head_i as u64)
    }

    pub fn has_unbalanced_indirectobj(&self) -> bool {
        self.has_indirect_obj_start ^ self.has_indirect_obj_end
    }

    // オブジェクト境界で区切られたbufferが入力されることを想定してbufferを字句解析する
    // 基本的に1つのオブジェクトの字句解析を目的としているのでstreamキーワードやendobjキーワードが来たらその時点で強制的に終了する
    pub fn tokenize(&mut self) -> Result<(), Error> {
        let mut is_comment = false;

        while self.token_head_i < self.buffer.len() {
            // LF
            if let 10 = self.byte {
                self.move_next_byte();

                if is_comment {
                    is_comment = false;
                    self.skip_token();
                } else {
                    self.skip_token();
                }

                continue;
            }

            // CR
            if let 13 = self.byte {
                // CR LFという並びの場合には一つのEOLとして扱う
                if raw_byte::is_next_satisfy(self.buffer, self.i, |b| b == 10) {
                    self.move_next_byte();
                }

                self.move_next_byte();

                if is_comment {
                    is_comment = false;
                    self.skip_token();
                } else {
                    self.skip_token();
                }

                continue;
            }

            // コメント中では改行以外は飛ばす
            if is_comment {
                self.move_next_byte();
                self.skip_token();
                continue;
            }

            // Space
            if let 0 | 9 | 12 | 32 = self.byte {
                self.move_next_byte();
                self.skip_token();
                continue;
            }

            // コメントはスペース1つとみなす
            if self.char == '%' {
                is_comment = true;
                self.move_next_byte();
                self.skip_token();
                continue;
            }

            // Integer/Real
            if self.is_number_char() {
                // 少し雑だが数字を構成する要素以外の要素が出るまで飛ばしその後に数字としてパースする
                // これだと "....." みたいな文字列もひとまず数字だと思いこんでしまうが後で弾く
                while self.is_number_char() {
                    if !self.move_next_byte() {
                        break;
                    }
                }

                let str = str::from_utf8(&self.buffer[self.token_head_i..self.i]).unwrap();

                if let Ok(int) = isize::from_str_radix(str, 10) {
                    self.confirm_token(Token::Integer(int));
                    continue;
                }

                if let Ok(real) = f64::from_str(str) {
                    self.confirm_token(Token::Real(real));
                    continue;
                }

                return Err(self.construct_error(ErrorKind::ParseNumber));
            }

            // 名前
            if self.char == '/' {
                self.move_next_byte();

                while self.is_regular_char() {
                    if !self.move_next_byte() {
                        break;
                    }
                }

                match parse_name(&self.buffer[(self.token_head_i + 1)..self.i]) {
                    Ok(name) => self.confirm_token(Token::Name(name)),
                    Err(_) => return Err(self.construct_error(ErrorKind::ParseName)),
                }
                continue;
            }

            // 16進数文字列/辞書デリミタ
            if self.char == '<' {
                // 16進数文字列/辞書のデリミタのどちらであっても途中でbufferが終わってしまっていることには変わりはない
                if !self.move_next_byte() {
                    return Err(self.construct_error(ErrorKind::FinishInObject));
                }

                // 辞書デリミタなら次のトークンへ
                if self.char == '<' {
                    self.move_next_byte();
                    self.confirm_token(Token::DictStart);
                    continue;
                }

                // ここからは16進数文字列のみ
                while self.char.is_ascii_hexdigit() {
                    if !self.move_next_byte() {
                        return Err(self.construct_error(ErrorKind::FinishInObject));
                    }
                }

                if self.char != '>' {
                    return Err(self.construct_error(ErrorKind::UnexpectedByte));
                }

                let token = match parse_hex_string(&self.buffer[(self.token_head_i + 1)..self.i]) {
                    Ok(hex_string) => Token::HexStr(hex_string),
                    Err(_) => return Err(self.construct_error(ErrorKind::ParseHexString)),
                };

                self.move_next_byte();
                self.confirm_token(token);
                continue;
            }

            // 辞書デリミタ
            // 16進数文字列は既に処理されている
            if self.char == '>' {
                if !self.move_next_byte() {
                    return Err(self.construct_error(ErrorKind::FinishInObject));
                }

                if self.char != '>' {
                    return Err(self.construct_error(ErrorKind::FinishInObject));
                }

                self.move_next_byte();
                self.confirm_token(Token::DictEnd);
                continue;
            }

            // 文字列リテラル
            // 文字列としての解釈は後に回す
            if self.char == '(' {
                if !self.move_next_byte() {
                    return Err(self.construct_error(ErrorKind::FinishInObject));
                }

                let mut prev_backslash = false;
                let mut parenthes_depth = 0;

                while !(prev_backslash == false && parenthes_depth == 0 && self.char == ')') {
                    // エスケープされていない(はエスケープされていない)に対応させる必要がある
                    if prev_backslash == false && self.char == '(' {
                        parenthes_depth += 1;
                    }

                    // エスケープされていない)は対応関係を更新する
                    if prev_backslash == false && self.char == ')' {
                        if parenthes_depth != 0 {
                            parenthes_depth -= 1;
                        }
                    }

                    // バックスラッシュを呼んだときには次の文字をエスケープする必要がある
                    // ただしバックスラッシュの連続はバックスラッシュそのものを表すため無視する
                    if prev_backslash == false && self.char == '\\' {
                        prev_backslash = true;
                    } else {
                        prev_backslash = false;
                    }

                    if !self.move_next_byte() {
                        return Err(self.construct_error(ErrorKind::FinishInObject));
                    }
                }

                // token_head_iは(を，iは)を指しているので token_head_i ではなく token_head_i + 1
                let token =
                    Token::String(parse_string(&self.buffer[(self.token_head_i + 1)..self.i])?);

                self.move_next_byte();
                self.confirm_token(token);
                continue;
            }

            // 配列
            if self.char == '[' {
                self.move_next_byte();
                self.confirm_token(Token::ArrayStart);
                continue;
            }

            // 配列の終わりは最終要素の直後に来る可能性がある
            if self.char == ']' {
                self.move_next_byte();
                self.confirm_token(Token::ArrayEnd);
                continue;
            }

            // 間接参照
            if self.char == 'R' {
                let may_gen_num = self.token_vec.pop();
                let may_obj_num = self.token_vec.pop();

                if let (Some(Token::Integer(object_num)), Some(Token::Integer(generation_num))) =
                    (&may_obj_num, &may_gen_num)
                {
                    if *object_num > 0 && *generation_num >= 0 {
                        self.move_next_byte();
                        self.confirm_token(Token::IndirectRef(
                            *object_num as u64,
                            *generation_num as u64,
                        ));
                        continue;
                    }
                }

                return Err(self.construct_error(ErrorKind::InvalidIndirectRef));
            }

            // 間接オブジェクト
            if self.assume_and_move("obj".as_bytes()) {
                let may_gen_num = self.token_vec.pop();
                let may_obj_num = self.token_vec.pop();

                if let (Some(Token::Integer(object_num)), Some(Token::Integer(generation_num))) =
                    (&may_obj_num, &may_gen_num)
                {
                    if *object_num > 0 && *generation_num >= 0 {
                        self.move_next_byte();
                        self.confirm_token(Token::IndirectObjStart(
                            *object_num as u64,
                            *generation_num as u64,
                        ));
                        self.has_indirect_obj_start = true;
                        continue;
                    }
                }
            }

            // 間接参照オブジェクトを読み終えたら強制的に字句解析を終了
            if self.assume_and_move("endobj".as_bytes()) {
                self.expect_keyword_end()?;

                self.confirm_token(Token::IndirectObjEnd);
                self.has_indirect_obj_end = true;
                return Ok(());
            }

            // ストリームオブジェクト
            // ストリームバイト列が始まるファイル内オフセットを計算して字句解析を終了する
            if self.assume_and_move("stream".as_bytes()) {
                // 現在mというバイトを指している
                // バイトmの1バイトとその後のEOLが最大2バイトあるのでStream先頭を把握するためには残り3バイト以上あることが必須
                if (self.buffer.len() - self.i) < 3 {
                    return Err(self.construct_error(ErrorKind::ConfirmStream));
                }

                self.move_next_byte();

                // streamキーワードの後は，LFかCRLFのみでCR単体は受け付けない
                // cf. 仕様書 3.2.7 Stream Objects
                if self.assume_and_move("\n".as_bytes()) || self.assume_and_move("\r\n".as_bytes())
                {
                    // パースしやすいように直前で間接オブジェクト自体は終了していることにする
                    self.confirm_token(Token::IndirectObjEnd);
                    self.has_indirect_obj_end = true;

                    // 現在iはEOLを指しておりストリーム先頭はその次なので+1
                    self.confirm_token(Token::StreamObjStart(
                        self.buffer_start_offset + self.i as u64 + 1,
                    ));

                    return Ok(());
                } else {
                    return Err(self.construct_error(ErrorKind::UnexpectedByte));
                }
            }

            // Null
            if self.assume_and_move("null".as_bytes()) {
                self.expect_keyword_end()?;

                self.confirm_token(Token::Null);
                continue;
            }

            // True
            if self.assume_and_move("true".as_bytes()) {
                self.expect_keyword_end()?;

                self.confirm_token(Token::Boolean(true));
                continue;
            }

            // False
            if self.assume_and_move("false".as_bytes()) {
                self.expect_keyword_end()?;

                self.confirm_token(Token::Boolean(false));
                continue;
            }

            return Err(self.construct_error(ErrorKind::UndefinedKeyword));
        }

        Ok(())
    }
}
