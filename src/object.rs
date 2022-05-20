use std::collections::HashMap;
use std::fs::File;
use std::slice;

use crate::cross_reference;
use crate::parser;
use crate::parser::Object;
use crate::util::read_partially;

// TODO 何のオブジェクトでエラーが出たのかわからない
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    ObjectTypeMissMatch {
        required_type: &'static str,
        byte_offset: u64,
    },
    DictKeyNotFound(&'static str),
    DictTypeMissMatch(String, String),
    InvalidStreamLength,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub trait PdfObject: std::fmt::Debug {
    fn byte_offset(&self) -> u64;
    fn type_missmatch_error(byte_offset: u64) -> Error;
}

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

// TODO ほぼボイラープレートコードなのでマクロの使いどきかもしれない
impl PdfObject for PdfBoolean {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "boolean",
            byte_offset,
        }
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
            _ => Err(PdfInteger::type_missmatch_error(obj.byte_offset())),
        }
    }

    pub fn unpack(&self) -> isize {
        self.payload
    }
}

impl PdfObject for PdfInteger {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "integer",
            byte_offset,
        }
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
impl PdfObject for PdfReal {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "real",
            byte_offset,
        }
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
            _ => Err(PdfName::type_missmatch_error(obj.byte_offset())),
        }
    }

    pub fn as_str(&self) -> &str {
        self.payload.as_str()
    }
}
impl PdfObject for PdfName {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "name",
            byte_offset,
        }
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
impl PdfObject for PdfString {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "string",
            byte_offset,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfArray {
    payload: Vec<Object>,
    byte_offset: u64,
}
impl PdfArray {
    pub fn new(arr: Vec<Object>, byte_offset: u64) -> Self {
        Self {
            payload: arr,
            byte_offset,
        }
    }

    pub fn ensure(obj: &Object) -> Result<&Self, Error> {
        match obj {
            Object::Array(array) => Ok(array),
            _ => Err(PdfArray::type_missmatch_error(obj.byte_offset())),
        }
    }
}
impl PdfObject for PdfArray {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "array",
            byte_offset,
        }
    }
}

impl<'a> std::iter::IntoIterator for &'a PdfArray {
    type Item = &'a Object;
    type IntoIter = slice::Iter<'a, Object>;

    fn into_iter(self) -> slice::Iter<'a, Object> {
        self.payload.iter()
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfNull {
    byte_offset: u64,
}
impl PdfNull {
    pub fn new(byte_offset: u64) -> Self {
        Self { byte_offset }
    }
}
impl PdfObject for PdfNull {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "null",
            byte_offset,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PdfIndirectRef {
    payload: (usize, usize),
    byte_offset: u64,
}
impl PdfIndirectRef {
    pub fn new(object_number: usize, generation_number: usize, byte_offset: u64) -> Self {
        if object_number == 0 {
            panic!("object number must not be 0");
        }

        Self {
            payload: (object_number, generation_number),
            byte_offset,
        }
    }

    pub fn ensure(obj: &Object) -> Result<&Self, Error> {
        match obj {
            Object::IndirectRef(indirect_ref) => Ok(indirect_ref),
            _ => Err(PdfIndirectRef::type_missmatch_error(obj.byte_offset())),
        }
    }

    pub fn get_indirect_obj(
        &self,
        file: &mut File,
        xref: &cross_reference::XRef,
    ) -> Result<Object, Error> {
        let offset = xref.get_object_byte_offset(file, self.payload.0, self.payload.1);

        let mut buf_size = 200;

        loop {
            let buffer = read_partially(file, offset, buf_size)?;
            let buffer = buffer.as_slice();

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
impl PdfObject for PdfIndirectRef {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "indirect ref",
            byte_offset,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfDict {
    payload: HashMap<String, Object>,
    byte_offset: u64,
}
impl PdfDict {
    pub fn new(hm: HashMap<String, Object>, byte_offset: u64) -> Self {
        Self {
            payload: hm,
            byte_offset,
        }
    }

    pub fn ensure_with_key<'a>(
        obj: &'a Object,
        keys: Vec<&'static str>,
    ) -> Result<&'a Self, Error> {
        let dict = match obj {
            Object::Dict(obj) => obj,
            _ => return Err(PdfDict::type_missmatch_error(obj.byte_offset())),
        };

        dict.assert_with_key(keys)?;

        Ok(dict)
    }

    pub fn ensure_type(&self, expected_type: &'static str) -> Result<(), Error> {
        let may_type_obj = self.payload.get(&String::from("Type")).unwrap();

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
            if let None = self.payload.get(&key_str) {
                return Err(Error::DictKeyNotFound(key));
            }
        }

        Ok(())
    }

    pub fn get(&self, key: &'static str) -> Option<&Object> {
        self.payload.get(key)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, Object> {
        self.payload.iter()
    }
}
impl PdfObject for PdfDict {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "dictionary",
            byte_offset,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PdfIndirectObj {
    payload: Box<Object>,
    byte_offset: u64,
}
impl PdfIndirectObj {
    pub fn new(obj: Object, byte_offset: u64) -> Self {
        Self {
            payload: Box::new(obj),
            byte_offset,
        }
    }

    pub fn ensure(obj: &Object) -> Result<&Self, Error> {
        match obj {
            Object::IndirectObj(obj) => Ok(obj),
            _ => return Err(PdfIndirectObj::type_missmatch_error(obj.byte_offset())),
        }
    }

    pub fn get_object(&self) -> &Object {
        &*self.payload
    }
}
impl PdfObject for PdfIndirectObj {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "indirect object",
            byte_offset,
        }
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
            _ => Err(PdfStreamObj::type_missmatch_error(obj.byte_offset())),
        }
    }

    pub fn get_stream(
        &self,
        file: &mut File,
        xref: &cross_reference::XRef,
    ) -> Result<Vec<u8>, Error> {
        let length = self.get_length_recursive(file, xref)?;

        let byte_vec = read_partially(file, self.byte_offset, length as u64)?;

        if byte_vec.len() != length {
            panic!("cannot read all");
        }
        Ok(byte_vec)
    }

    fn get_length_recursive(
        &self,
        file: &mut File,
        xref: &cross_reference::XRef,
    ) -> Result<usize, Error> {
        let length = match self.dict.get("Length").unwrap() {
            Object::Integer(integer) => integer.unpack(),
            Object::IndirectRef(indirect_ref) => {
                let may_indirect_obj = indirect_ref.get_indirect_obj(file, xref)?;
                let indirect_obj = PdfIndirectObj::ensure(&may_indirect_obj)?;

                PdfInteger::ensure(indirect_obj.get_object())?.unpack()
            }
            o => return Err(PdfInteger::type_missmatch_error(o.byte_offset())),
        };

        if length < 0 {
            return Err(Error::InvalidStreamLength);
        }

        Ok(length.try_into().unwrap())
    }
}
impl PdfObject for PdfStreamObj {
    fn byte_offset(&self) -> u64 {
        self.byte_offset
    }
    fn type_missmatch_error(byte_offset: u64) -> Error {
        Error::ObjectTypeMissMatch {
            required_type: "stream object",
            byte_offset,
        }
    }
}
