use std::collections::HashMap;

pub mod error;

use crate::lexer;
use crate::lexer::{Token, TokenContent};
use crate::object::{
    PdfArray, PdfBoolean, PdfDict, PdfIndirectObj, PdfIndirectRef, PdfInteger, PdfName, PdfNull,
    PdfReal, PdfStreamObj, PdfString,
};
use error::{Error, ErrorKind};

#[cfg(test)]
pub mod test;

#[derive(Debug, PartialEq)]
pub enum Object {
    Boolean(PdfBoolean),
    Integer(PdfInteger),
    Real(PdfReal),
    Name(PdfName),
    String(PdfString),
    Array(PdfArray),
    Null(PdfNull),
    IndirectRef(PdfIndirectRef),
    Dict(PdfDict),
    IndirectObj(PdfIndirectObj),
    StreamObj(PdfStreamObj),
}

pub struct Parser {
    token_i: usize,
    byte_offset: u64,
    token_vec: Vec<Token>,
}

impl Parser {
    pub fn new(buffer: &[u8], buffer_start_offset: u64) -> Result<Parser, Error> {
        if buffer.len() == 0 {
            panic!("buffer is empty");
        };

        let mut lexer = lexer::Lexer::new(buffer, buffer_start_offset);

        if let Err(e) = lexer.tokenize() {
            return Err(Error::new(ErrorKind::Lexer(e), buffer_start_offset));
        };

        if lexer.has_unbalanced_indirectobj() {
            return Err(Error::new(
                ErrorKind::IndirectObjMissMatch,
                buffer_start_offset,
            ));
        }

        let token_vec = lexer.token_vec;

        Ok(Parser {
            token_vec,
            byte_offset: buffer_start_offset,
            token_i: 0,
        })
    }

    pub fn parse(&mut self) -> Result<Object, error::Error> {
        Ok(self.parse_object()?)
    }

    fn next(&mut self) -> Option<&Token> {
        let i = self.token_i;

        self.token_i += 1;
        if self.token_vec.len() <= i {
            None
        } else {
            Some(&self.token_vec[i])
        }
    }

    fn current_token(&self) -> Option<&Token> {
        if self.token_vec.len() <= self.token_i {
            None
        } else {
            Some(&self.token_vec[self.token_i])
        }
    }

    fn parse_object(&mut self) -> Result<Object, error::Error> {
        let token = match self.next() {
            Some(token) => token,
            None => return Err(Error::new(ErrorKind::NoToken, self.byte_offset)),
        };

        if let TokenContent::Boolean(boolean) = token.content() {
            return Ok(Object::Boolean(PdfBoolean::new(
                *boolean,
                token.byte_offset,
            )));
        };

        if let TokenContent::Integer(int) = token.content() {
            return Ok(Object::Integer(PdfInteger::new(*int, token.byte_offset)));
        }

        if let TokenContent::Real(real) = token.content() {
            return Ok(Object::Real(PdfReal::new(*real)));
        }

        if let TokenContent::Name(str) = token.content() {
            return Ok(Object::Name(PdfName::new((*str).clone())));
        }

        if let TokenContent::Null = token.content() {
            return Ok(Object::Null(PdfNull));
        }

        if let TokenContent::HexStr(vec) = token.content() {
            return Ok(Object::String(PdfString::new((*vec).clone())));
        }

        if let TokenContent::String(vec) = token.content() {
            return Ok(Object::String(PdfString::new((*vec).clone())));
        }

        if let TokenContent::IndirectRef(obj_num, gen_num) = token.content() {
            return Ok(Object::IndirectRef(PdfIndirectRef::new(*obj_num, *gen_num)));
        }

        if let TokenContent::ArrayStart = token.content() {
            return Ok(Object::Array(PdfArray::new(self.parse_array_content()?)));
        }

        if let TokenContent::DictStart = token.content() {
            return Ok(Object::Dict(PdfDict::new(self.parse_dict_content()?)));
        }

        if let TokenContent::IndirectObjStart(_, _) = token.content() {
            let obj = self.parse_indirect_content()?;

            if let Some(Token {
                token_content: TokenContent::StreamObjStart(offset),
                byte_offset: _,
            }) = self.next()
            {
                let stream = match PdfStreamObj::new(obj, *offset) {
                    Ok(obj) => return Ok(Object::StreamObj(obj)),
                    Err(_) => {
                        return Err(Error::new(ErrorKind::InvalidStreamObj, self.byte_offset))
                    }
                };
            } else {
                return Ok(Object::IndirectObj(PdfIndirectObj::new(obj)));
            };
        }

        Err(Error::new(
            ErrorKind::UnexpectedToken(token.clone()),
            token.byte_offset,
        ))
    }

    fn parse_array_content(&mut self) -> Result<Vec<Object>, error::Error> {
        let mut may_token;
        let mut content: Vec<Object> = Vec::new();

        loop {
            may_token = self.current_token();

            if let None = may_token {
                return Err(Error::new(ErrorKind::NoToken, self.byte_offset));
            }

            let token = may_token.unwrap();

            if let TokenContent::EOL = token.content() {
                self.next();
                continue;
            }

            if let TokenContent::ArrayEnd = token.content() {
                self.next();
                return Ok(content);
            }

            content.push(self.parse_object()?);
        }
    }

    fn parse_dict_content(&mut self) -> Result<HashMap<String, Object>, error::Error> {
        let mut may_token;
        let mut content: HashMap<String, Object> = HashMap::new();

        let mut is_prev_name = false;
        let mut key: String = String::from("");

        loop {
            may_token = self.current_token();

            if let None = may_token {
                return Err(Error::new(ErrorKind::NoToken, self.byte_offset));
            }

            let token = may_token.unwrap();

            if let TokenContent::EOL = token.content() {
                self.next();
                continue;
            }

            if is_prev_name {
                content.insert(key.clone(), self.parse_object()?);
                is_prev_name = false;
            } else {
                if let TokenContent::Name(string) = token.content() {
                    key = string.clone();
                    // TODO キーの重複はどうする？
                    is_prev_name = true;
                    self.next();
                    continue;
                } else if let TokenContent::DictEnd = token.content() {
                    self.next();
                    return Ok(content);
                } else {
                    return Err(Error::new(
                        ErrorKind::UnexpectedToken(token.clone()),
                        token.byte_offset,
                    ));
                }
            }
        }
    }

    fn parse_indirect_content(&mut self) -> Result<Object, error::Error> {
        let obj = self.parse_object()?;

        match self.next() {
            Some(token) => match token {
                Token {
                    token_content: TokenContent::IndirectObjEnd,
                    byte_offset: _,
                } => Ok(obj),
                _ => Err(Error::new(
                    ErrorKind::UnexpectedToken(token.clone()),
                    token.byte_offset,
                )),
            },
            None => Err(Error::new(ErrorKind::NoToken, self.byte_offset)),
        }
    }
}
