use std::collections::HashMap;

use super::*;
use crate::object::*;

#[test]
fn parse_integer() {
    let buffer = "123".as_bytes();

    let mut parser = Parser::new(buffer, 0).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(obj, Object::Integer(PdfInteger::new(123, 0)));
}

#[test]
fn parse_real() {
    let buffer = "-123.".as_bytes();

    let mut parser = Parser::new(buffer, 0).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(obj, Object::Real(PdfReal::new(-123.0, 0)));
}

#[test]
fn parse_boolean() {
    let buffer = "true".as_bytes();

    let mut parser = Parser::new(buffer, 0).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(obj, Object::Boolean(PdfBoolean::new(true, 0)));
}

#[test]
fn parse_null() {
    let buffer = "null".as_bytes();

    let mut parser = Parser::new(buffer, 0).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(obj, Object::Null(PdfNull::new()));
}

#[test]
fn parse_indirect_ref() {
    let buffer = "1 0 R".as_bytes();

    let mut parser = Parser::new(buffer, 0).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(obj, Object::IndirectRef(PdfIndirectRef::new(1, 0)));
}

#[test]
fn parse_string_1() {
    let buffer = "(hoge)".as_bytes();

    let mut parser = Parser::new(buffer, 0).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(
        obj,
        Object::String(PdfString::new(vec![104, 111, 103, 101]))
    );
}

#[test]
fn parse_array_1() {
    let buffer = "[  123  true \n  -12.[2 1 R\nnull] ]".as_bytes();

    let mut parser = Parser::new(buffer, 0).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(
        obj,
        Object::Array(PdfArray::new(vec![
            Object::Integer(PdfInteger::new(123, 3)),
            Object::Boolean(PdfBoolean::new(true, 8)),
            Object::Real(PdfReal::new(-12.0, 16)),
            Object::Array(PdfArray::new(vec![
                Object::IndirectRef(PdfIndirectRef::new(2, 1)),
                Object::Null(PdfNull::new())
            ]))
        ]))
    );
}

#[test]
fn parse_dict_1() {
    let buffer = "<</hoge 1 0 R\n/fuga <</arr [123\n/name]>>>>".as_bytes();

    let mut parser = Parser::new(buffer, 0).unwrap();
    let obj = parser.parse_object().unwrap();

    let mut hm = HashMap::new();
    hm.insert(
        String::from("hoge"),
        Object::IndirectRef(PdfIndirectRef::new(1, 0)),
    );

    let mut inner_hm = HashMap::new();
    inner_hm.insert(
        String::from("arr"),
        Object::Array(PdfArray::new(vec![
            Object::Integer(PdfInteger::new(123, 28)),
            Object::Name(PdfName::new(String::from("name"))),
        ])),
    );

    hm.insert(String::from("fuga"), Object::Dict(PdfDict::new(inner_hm)));

    assert_eq!(obj, Object::Dict(PdfDict::new(hm)));
}
