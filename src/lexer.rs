use crate::raw_byte;
use std::str;
use std::str::FromStr;

use crate::error as api_error;

pub mod error;
#[cfg(test)]
mod test;

#[derive(PartialEq, Debug)]
pub enum Token {
    EOL,
    Boolean(bool),
    Integer(isize),
    Real(f64),
    HexStr(Vec<u8>),
    String(Vec<u8>),
    Name(Vec<u8>),
    DictStart,
    DictEnd,
    ArrayStart,
    ArrayEnd,
    Null,
    IndirectRef(usize, usize),
}

// Streamオブジェクト・間接オブジェクト以外のオブジェクトの字句解析を行う
// Streamオブジェクトはバイト長が辞書によって指定されるためこの中でやろうとすると難しくなる
// 間接オブジェクトはStreamオブジェクトを含む可能性がある
// こういったオブジェクトは字句解析できるバイト列を切り取って範囲を絞って字句解析を行う
pub struct Lexer<'a> {
    buffer: &'a [u8],
    i: usize,
    token_head_i: usize,
    byte: u8,
    char: char,
    token_vec: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(buffer: &[u8]) -> Result<Lexer, api_error::Error> {
        if buffer.len() == 0 {
            Err(api_error::Error::CannotParse)
        } else {
            Ok(Lexer {
                buffer,
                i: 0,
                token_head_i: 0,
                byte: buffer[0],
                char: char::from(buffer[0]),
                token_vec: vec![],
            })
        }
    }

    pub fn tokenize(&mut self) -> Result<(), api_error::Error> {
        if let Err(e) = self._tokenize() {
            Err(api_error::Error::Lexer(e))
        } else {
            Ok(())
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

    fn confirm_token(&mut self, token: Token) {
        self.token_vec.push(token);
        self.token_head_i = self.i;
    }

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

    // もしtargetと一致するならカーソルをtargetの最後まで移動させる
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

    // オブジェクト境界で区切られたbufferが入力されることを想定
    // オブジェクトの途中で区切られたbufferを許容することも可能だがstreamの途中で切られたbufferが来てしまうとわけの分からないトークンを吐き続けてしまう
    fn _tokenize(&mut self) -> Result<(), error::Error> {
        let mut is_comment = false;

        while self.token_head_i < self.buffer.len() {
            // LF
            if let 10 = self.byte {
                self.move_next_byte();

                if is_comment {
                    is_comment = false;
                    self.skip_token();
                } else {
                    self.confirm_token(Token::EOL);
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
                    self.confirm_token(Token::EOL);
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

                return Err(error::Error::ParseNumber(String::from(str)));
            }

            // 名前
            if self.char == '/' {
                self.move_next_byte();

                while self.is_regular_char() {
                    if !self.move_next_byte() {
                        break;
                    }
                }

                //  この時点でtoken_head_iは/を指しているので token_head_i + 1
                self.confirm_token(Token::Name(
                    (&self.buffer[(self.token_head_i + 1)..self.i]).to_vec(),
                ));
                continue;
            }

            // 16進数文字列/辞書デリミタ
            if self.char == '<' {
                // 16進数文字列/辞書のデリミタのどちらであっても途中でbufferが終わってしまっていることには変わりはない
                if !self.move_next_byte() {
                    return Err(error::Error::FinishInObject);
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
                        return Err(error::Error::FinishInObject);
                    }
                }

                if self.char != '>' {
                    return Err(error::Error::UnexpectedByte(self.byte, '>'));
                }

                // token_head_iは<を指しているので token_head_i ではなく token_head_i + 1
                let token = Token::HexStr((&self.buffer[(self.token_head_i + 1)..self.i]).to_vec());

                self.move_next_byte();
                self.confirm_token(token);
                continue;
            }

            // 辞書デリミタ
            // 16進数文字列は既に処理されている
            if self.char == '>' {
                if !self.move_next_byte() {
                    return Err(error::Error::FinishInObject);
                }

                if self.char != '>' {
                    return Err(error::Error::UnexpectedByte(self.byte, '>'));
                }

                self.move_next_byte();
                self.confirm_token(Token::DictEnd);
                continue;
            }

            // 文字列リテラル
            // 文字列としての解釈は後に回す
            if self.char == '(' {
                if !self.move_next_byte() {
                    return Err(error::Error::FinishInObject);
                }

                let mut prev_backslash = false;
                let mut parenthes_depth = 0;

                while !(prev_backslash == false && parenthes_depth == 0 && self.char == ')') {
                    println!("{} {} {}", self.char, prev_backslash, parenthes_depth);
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
                        return Err(error::Error::FinishInObject);
                    }
                }

                // token_head_iは(を，iは)を指しているので token_head_i ではなく token_head_i + 1
                let token = Token::String((&self.buffer[(self.token_head_i + 1)..self.i]).to_vec());

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
                            *object_num as usize,
                            *generation_num as usize,
                        ));
                        continue;
                    }
                }

                return Err(error::Error::InvalidIndirectRef(may_obj_num, may_gen_num));
            }

            // Null
            if self.assume_and_move("null".as_bytes()) {
                if self.move_next_byte() {
                    if self.is_regular_char() {
                        let str = str::from_utf8(&self.buffer[self.token_head_i..self.i]).unwrap();
                        return Err(error::Error::UndefinedKeyword(String::from(str)));
                    }
                }

                self.confirm_token(Token::Null);
                continue;
            }

            // True
            if self.assume_and_move("true".as_bytes()) {
                if self.move_next_byte() {
                    if self.is_regular_char() {
                        let str = str::from_utf8(&self.buffer[self.token_head_i..self.i]).unwrap();
                        return Err(error::Error::UndefinedKeyword(String::from(str)));
                    }
                }

                self.confirm_token(Token::Boolean(true));
                continue;
            }

            // False
            if self.assume_and_move("false".as_bytes()) {
                if self.move_next_byte() {
                    if self.is_regular_char() {
                        let str = str::from_utf8(&self.buffer[self.token_head_i..self.i]).unwrap();
                        return Err(error::Error::UndefinedKeyword(String::from(str)));
                    }
                }

                self.confirm_token(Token::Boolean(false));
                continue;
            }

            return Err(error::Error::InvalidObjectHead(self.byte));
        }

        Ok(())
    }
}
