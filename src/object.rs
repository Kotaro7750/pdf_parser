use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::slice;

use crate::cross_reference;
use crate::parser;
use crate::parser::Object;

// TODO 何のオブジェクトでエラーが出たのかわからない
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parser(parser::error::Error),
    NotInteger(String),
    NotName(String),
    NotDictionary(String),
    NotIndirectObj(String),
    NotIndirectRef(String),
    NotArray(String),
    NotStream(String),
    ObjectRestriction(String),
    DictKeyNotFound(String),
    DictTypeMissMatch(String, String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

// TODO マクロ展開をするマクロとかあるといいね
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PdfBoolean {
    payload: bool,
    byte_offset: u64,
}
impl PdfBoolean {
    pub fn new(b: bool, byte_offset: u64) -> Self {
        Self {
            payload: b,
            byte_offset,
        }
    }

    pub fn unpack(&self) -> bool {
        self.payload
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PdfInteger {
    payload: isize,
    byte_offset: u64,
}
impl PdfInteger {
    pub fn new(i: isize, byte_offset: u64) -> Self {
        Self {
            payload: i,
            byte_offset,
        }
    }

    pub fn ensure(obj: &Object) -> Result<&Self, Error> {
        match obj {
            Object::Integer(int) => Ok(int),
            _ => Err(Error::NotInteger(format!("{:?}", obj))),
        }
    }

    pub fn unpack(&self) -> isize {
        self.payload
    }
}

impl std::convert::TryFrom<PdfInteger> for u64 {
    type Error = ();
    fn try_from(value: PdfInteger) -> Result<Self, Self::Error> {
        if value.payload > 0 {
            Ok(value.payload as u64)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfReal {
    payload: f64,
    byte_offset: u64,
}
impl PdfReal {
    pub fn new(f: f64, byte_offset: u64) -> Self {
        Self {
            payload: f,
            byte_offset,
        }
    }

    pub fn unpack(&self) -> f64 {
        self.payload
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PdfName {
    payload: String,
    byte_offset: u64,
}
impl PdfName {
    pub fn new(s: String, byte_offset: u64) -> Self {
        Self {
            payload: s,
            byte_offset,
        }
    }

    pub fn ensure(obj: &Object) -> Result<&Self, Error> {
        match obj {
            Object::Name(name) => Ok(name),
            _ => Err(Error::NotName(format!("{:?}", obj))),
        }
    }

    pub fn as_str(&self) -> &str {
        self.payload.as_str()
    }
}

impl PartialEq<str> for PdfName {
    fn eq(&self, other: &str) -> bool {
        self.payload == other
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfString {
    payload: Vec<u8>,
    byte_offset: u64,
}
impl PdfString {
    pub fn new(s: Vec<u8>, byte_offset: u64) -> Self {
        Self {
            payload: s,
            byte_offset,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfArray(Vec<Object>);
impl PdfArray {
    pub fn new(vec: Vec<Object>) -> Self {
        Self(vec)
    }

    pub fn ensure(obj: &Object) -> Result<&Self, Error> {
        match obj {
            Object::Array(array) => Ok(array),
            _ => Err(Error::NotArray(format!("{:?}", obj))),
        }
    }
}

impl<'a> std::iter::IntoIterator for &'a PdfArray {
    type Item = &'a Object;
    type IntoIter = slice::Iter<'a, Object>;

    fn into_iter(self) -> slice::Iter<'a, Object> {
        self.0.iter()
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfNull;
impl PdfNull {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PdfIndirectRef {
    object_number: usize,
    generation_number: usize,
}
impl PdfIndirectRef {
    pub fn new(object_number: usize, generation_number: usize) -> Self {
        if object_number == 0 {
            panic!("object number must not be 0");
        }

        Self {
            object_number,
            generation_number,
        }
    }

    pub fn ensure(obj: &Object) -> Result<&Self, Error> {
        match obj {
            Object::IndirectRef(indirect_ref) => Ok(indirect_ref),
            _ => Err(Error::NotIndirectRef(format!("{:?}", obj))),
        }
    }

    pub fn get_indirect_obj(
        &self,
        file: &mut File,
        xref: &cross_reference::XRef,
    ) -> Result<Object, Error> {
        let offset = xref.get_object_byte_offset(file, self.object_number, self.generation_number);

        let mut buf_size = 200;
        let mut buffer: Vec<u8>;

        loop {
            file.seek(SeekFrom::Start(offset))?;

            buffer = vec![0; buf_size];

            let n = file.read(&mut buffer)?;

            let buffer = &buffer[..n];

            let mut p = match parser::Parser::new(&buffer, offset) {
                Ok(p) => p,
                // bufferが足りなくて途中で切れてしまうと字句解析自体も失敗することがある
                // TODO これだけでいいのか？
                Err(e) => {
                    let kind = e.kind;
                    match kind {
                        parser::error::ErrorKind::IndirectObjMissMatch
                        | parser::error::ErrorKind::Lexer(_) => {
                            buf_size += 200;
                            continue;
                        }
                        _ => panic!(""),
                    }
                }
            };

            // TODO エラーは書こう
            let obj = p.parse().unwrap();

            return Ok(obj);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfDict(HashMap<String, Object>);
impl PdfDict {
    pub fn new(hm: HashMap<String, Object>) -> Self {
        Self(hm)
    }

    pub fn ensure_with_key<'a>(
        obj: &'a Object,
        keys: Vec<&'static str>,
    ) -> Result<&'a Self, Error> {
        let dict = match obj {
            Object::Dict(obj) => obj,
            _ => return Err(Error::NotDictionary(format!("{:?}", obj))),
        };

        dict.assert_with_key(keys)?;

        Ok(dict)
    }

    pub fn ensure_type(&self, expected_type: &'static str) -> Result<(), Error> {
        let may_type_obj = self.0.get(&String::from("Type")).unwrap();

        let type_obj = PdfName::ensure(may_type_obj)?;

        // Typeというキーを持っていることは呼び出し側で保証する
        if type_obj == expected_type {
            Ok(())
        } else {
            Err(Error::DictTypeMissMatch(
                expected_type.to_string(),
                (&type_obj).payload.clone(),
            ))
        }
    }

    pub fn assert_with_key(&self, keys: Vec<&'static str>) -> Result<(), Error> {
        for ref key in keys {
            let key_str = String::from(*key);
            if let None = self.0.get(&key_str) {
                return Err(Error::DictKeyNotFound(key.to_string()));
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &'static str) -> Option<&Object> {
        self.0.get(key)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, Object> {
        self.0.iter()
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfIndirectObj(Box<Object>);
impl PdfIndirectObj {
    pub fn new(obj: Object) -> Self {
        Self(Box::new(obj))
    }

    pub fn ensure(obj: &Object) -> Result<&Self, Error> {
        match obj {
            Object::IndirectObj(obj) => Ok(obj),
            _ => return Err(Error::NotIndirectObj(format!("{:?}", obj))),
        }
    }

    pub fn get_object(&self) -> &Object {
        &*self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfStreamObj {
    pub dict: PdfDict,
    byte_offset: u64,
}
impl PdfStreamObj {
    pub fn new(obj: Object, byte_offset: u64) -> Result<Self, Error> {
        PdfDict::ensure_with_key(&obj, vec!["Length"])?;

        if let Object::Dict(dict) = obj {
            Ok(Self { dict, byte_offset })
        } else {
            panic!()
        }
    }

    pub fn ensure_stream(obj: &Object) -> Result<&Self, Error> {
        match obj {
            Object::StreamObj(stream_obj) => Ok(stream_obj),
            _ => Err(Error::NotStream(format!("{:?}", obj))),
        }
    }

    pub fn get_stream(
        &self,
        file: &mut File,
        xref: &cross_reference::XRef,
    ) -> Result<Vec<u8>, Error> {
        let length = match self.dict.get("Length").unwrap() {
            Object::Integer(int) => int.unpack(),
            Object::IndirectRef(indirect_ref) => {
                let may_indirect_obj = indirect_ref.get_indirect_obj(file, xref)?;
                let indirect_obj = PdfIndirectObj::ensure(&may_indirect_obj)?;

                PdfInteger::ensure(indirect_obj.get_object())?.unpack()
            }
            o => return Err(Error::NotInteger(format!("{:?}", o))),
        };

        if length < 0 {
            // TODO
        }

        let byte_vec = PdfStreamObj::get_stream_byte(file, self.byte_offset, length as u64)?;

        Ok(byte_vec)
    }

    fn get_stream_byte(file: &mut File, offset: u64, size: u64) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![0; size as usize];

        file.seek(SeekFrom::Start(offset))?;

        if file.read(&mut buffer)? as u64 != size {
            panic!("Cannot read all");
        };

        Ok(buffer)
    }
}
