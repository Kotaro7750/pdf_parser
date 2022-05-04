use super::*;

#[test]
fn parse_integer() {
    let buffer = "123".as_bytes();

    let mut parser = Parser::new(buffer).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(obj, Object::Integer(123));
}

#[test]
fn parse_real() {
    let buffer = "-123.".as_bytes();

    let mut parser = Parser::new(buffer).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(obj, Object::Real(-123.0));
}

#[test]
fn parse_boolean() {
    let buffer = "true".as_bytes();

    let mut parser = Parser::new(buffer).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(obj, Object::Boolean(true));
}

#[test]
fn parse_null() {
    let buffer = "null".as_bytes();

    let mut parser = Parser::new(buffer).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(obj, Object::Null);
}

#[test]
fn parse_indirect_ref() {
    let buffer = "1 0 R".as_bytes();

    let mut parser = Parser::new(buffer).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(obj, Object::IndirectRef(1, 0));
}

#[test]
fn parse_array_1() {
    let buffer = "[  123  true \n  -12.[2 1 R\nnull] ]".as_bytes();

    let mut parser = Parser::new(buffer).unwrap();
    let obj = parser.parse_object().unwrap();

    assert_eq!(
        obj,
        Object::Array(vec![
            Object::Integer(123),
            Object::Boolean(true),
            Object::Real(-12.0),
            Object::Array(vec![Object::IndirectRef(2, 1), Object::Null])
        ])
    );
}
