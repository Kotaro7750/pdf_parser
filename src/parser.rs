use std::collections::HashMap;

pub mod error;
use crate::lexer;
use crate::lexer::Token;

#[cfg(test)]
pub mod test;

#[derive(PartialEq, Debug)]
pub enum Object {
    Boolean(bool),
    Integer(isize),
    Real(f64),
    Name(String),
    String(Vec<u8>),
    Array(Vec<Object>),
    Null,
    IndirectRef(u64, u64),
    Dict(HashMap<String, Object>),
    IndirectObj(Box<Object>),
    // ストリームバイト列が始まるバイトオフセット
    StreamObj(Box<Object>, u64),
}

pub struct Parser {
    token_i: usize,
    token_vec: Vec<Token>,
}

impl Parser {
    pub fn new(buffer: &[u8], buffer_start_offset: u64) -> Result<Parser, error::Error> {
        if buffer.len() == 0 {
            return Err(error::Error::EmptyBuffer);
        };

        let mut lexer = lexer::Lexer::new(buffer, buffer_start_offset);

        if let Err(e) = lexer.tokenize() {
            return Err(error::Error::Lexer(e));
        };

        if lexer.has_unbalanced_indirectobj() {
            return Err(error::Error::IndirectObjMissMatch);
        }

        let token_vec = lexer.token_vec;

        Ok(Parser {
            token_vec,
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
            None => return Err(error::Error::NoToken),
        };

        if let Token::Boolean(boolean) = token {
            return Ok(Object::Boolean(*boolean));
        };

        if let Token::Integer(int) = token {
            return Ok(Object::Integer(*int));
        }

        if let Token::Real(real) = token {
            return Ok(Object::Real(*real));
        }

        if let Token::Name(str) = token {
            return Ok(Object::Name((*str).clone()));
        }

        if let Token::Null = token {
            return Ok(Object::Null);
        }

        if let Token::HexStr(vec) = token {
            return Ok(Object::String((*vec).clone()));
        }

        if let Token::String(vec) = token {
            return Ok(Object::String((*vec).clone()));
        }

        if let Token::IndirectRef(obj_num, gen_num) = token {
            return Ok(Object::IndirectRef(*obj_num, *gen_num));
        }

        if let Token::ArrayStart = token {
            return Ok(Object::Array(self.parse_array_content()?));
        }

        if let Token::DictStart = token {
            return Ok(Object::Dict(self.parse_dict_content()?));
        }

        if let Token::IndirectObjStart(_, _) = token {
            let obj = self.parse_indirect_content()?;

            return if let Some(Token::StreamObjStart(offset)) = self.next() {
                Ok(Object::StreamObj(obj, *offset))
            } else {
                Ok(Object::IndirectObj(obj))
            };
        }

        Err(error::Error::UnexpectedToken(token.clone()))
    }

    fn parse_array_content(&mut self) -> Result<Vec<Object>, error::Error> {
        let mut may_token;
        let mut content: Vec<Object> = Vec::new();

        loop {
            may_token = self.current_token();

            if let None = may_token {
                return Err(error::Error::NoToken);
            }

            let token = may_token.unwrap();

            if let Token::EOL = token {
                self.next();
                continue;
            }

            if let Token::ArrayEnd = token {
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
                return Err(error::Error::NoToken);
            }

            let token = may_token.unwrap();

            if let Token::EOL = token {
                self.next();
                continue;
            }

            if is_prev_name {
                content.insert(key.clone(), self.parse_object()?);
                is_prev_name = false;
            } else {
                if let Token::Name(string) = token {
                    key = string.clone();
                    // TODO キーの重複はどうする？
                    is_prev_name = true;
                    self.next();
                    continue;
                } else if let Token::DictEnd = token {
                    self.next();
                    return Ok(content);
                } else {
                    return Err(error::Error::UnexpectedToken(token.clone()));
                }
            }
        }
    }

    fn parse_indirect_content(&mut self) -> Result<Box<Object>, error::Error> {
        let obj = self.parse_object()?;

        if let Some(Token::IndirectObjEnd) = self.next() {
            Ok(Box::new(obj))
        } else {
            // TODO
            Err(error::Error::NoToken)
        }
    }
}
