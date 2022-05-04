pub mod error;
use crate::lexer;
use crate::lexer::Token;

#[cfg(test)]
pub mod test;

#[derive(PartialEq, Debug)]
enum Object {
    Boolean(bool),
    Integer(isize),
    Real(f64),
    Array(Vec<Object>),
    Null,
    IndirectRef(usize, usize),
}

pub struct Parser {
    token_i: usize,
    token_vec: Vec<Token>,
}

impl Parser {
    pub fn new(buffer: &[u8]) -> Result<Parser, error::Error> {
        if buffer.len() == 0 {
            return Err(error::Error::EmptyBuffer);
        };

        let mut lexer = match lexer::Lexer::new(buffer) {
            Ok(lexer) => lexer,
            Err(e) => return Err(error::Error::Lexer(e)),
        };

        if let Err(e) = lexer.tokenize() {
            return Err(error::Error::Lexer(e));
        };

        let token_vec = lexer.token_vec;

        Ok(Parser {
            token_vec,
            token_i: 0,
        })
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

    pub fn parse_object(&mut self) -> Result<Object, error::Error> {
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

        if let Token::Null = token {
            return Ok(Object::Null);
        }

        if let Token::IndirectRef(obj_num, gen_num) = token {
            return Ok(Object::IndirectRef(*obj_num, *gen_num));
        }

        if let Token::ArrayStart = token {
            return Ok(Object::Array(self.parse_array_content()?));
        }

        Err(error::Error::UnexpectedToken)
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
}
