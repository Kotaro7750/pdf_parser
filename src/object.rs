use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use crate::cross_reference;
use crate::parser;
use crate::parser::Object;

// TODO 何のオブジェクトでエラーが出たのかわからない
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parser(parser::error::Error),
    NotDictionary(String),
    NotIndirectObj(String),
    NotIndirectRef(String),
    NotArray(String),
    ObjectRestriction(String),
    DictKeyNotFound(String),
    DictTypeMissMatch(String, String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub fn get_indirect_obj(
    file: &mut File,
    xref: &mut cross_reference::XRef,
    indirect: (u64, u64),
) -> Result<Object, Error> {
    let offset = xref.get_object_byte_offset(file, indirect.0, indirect.1);

    let mut buf_size = 200;
    let mut buffer: Vec<u8>;

    loop {
        file.seek(SeekFrom::Start(offset))?;

        buffer = vec![0; buf_size];

        let n = file.read(&mut buffer)?;

        let buffer = &buffer[..n];

        let mut p = match parser::Parser::new(&buffer) {
            Ok(p) => p,
            // bufferが足りなくて途中で切れてしまうと字句解析自体も失敗することがある
            // TODO これだけでいいのか？
            Err(parser::error::Error::IndirectObjMissMatch)
            | Err(parser::error::Error::Lexer(_)) => {
                buf_size += 200;
                continue;
            }
            Err(e) => {
                panic!("{}", e)
            }
        };

        // TODO エラーは書こう
        let obj = p.parse().unwrap();

        return Ok(obj);
    }
}

pub fn ensure_dict_with_key<'a>(
    obj: &'a Object,
    restriction: Vec<&'static str>,
) -> Result<&'a HashMap<String, Object>, Error> {
    let dict = match obj {
        Object::Dict(obj) => obj,
        _ => return Err(Error::NotDictionary(format!("{:?}", obj))),
    };

    for ref key in restriction {
        let key_str = String::from(*key);
        if let None = dict.get(&key_str) {
            return Err(Error::DictKeyNotFound(key.to_string()));
        }
    }

    Ok(dict)
}

pub fn ensure_indirect_obj(obj: &Object) -> Result<&Object, Error> {
    match obj {
        Object::IndirectObj(inner) => Ok(inner.as_ref()),
        _ => return Err(Error::NotIndirectObj(format!("{:?}", obj))),
    }
}

pub fn ensure_indirect_ref(obj: &Object) -> Result<(u64, u64), Error> {
    match obj {
        Object::IndirectRef(obj_num, gen_num) => Ok((*obj_num, *gen_num)),
        _ => Err(Error::NotIndirectRef(format!("{:?}", obj))),
    }
}

pub fn ensure_array(obj: &Object) -> Result<&Vec<Object>, Error> {
    match obj {
        Object::Array(vec) => Ok(vec),
        _ => Err(Error::NotArray(format!("{:?}", obj))),
    }
}

pub fn ensure_dict_type(
    hm: &HashMap<String, parser::Object>,
    expected_type: &'static str,
) -> Result<(), Error> {
    // Typeというキーを持っていることは呼び出し側で保証する
    if let parser::Object::Name(str) = hm.get(&String::from("Type")).unwrap() {
        if str == expected_type {
            Ok(())
        } else {
            Err(Error::DictTypeMissMatch(
                expected_type.to_string(),
                str.to_string(),
            ))
        }
    } else {
        Err(Error::ObjectRestriction(String::from("Type")))
    }
}
