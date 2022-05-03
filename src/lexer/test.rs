use super::*;

fn eq_token_vec(v1: &Vec<Token>, v2: &Vec<Token>) -> bool {
    if v1.len() != v2.len() {
        false
    } else {
        v1.iter().zip(v2).all(|(a, b)| a == b)
    }
}

fn assert_eq_token_vec(v1: &Vec<Token>, v2: &Vec<Token>) {
    if !eq_token_vec(v1, v2) {
        panic!("left: {:?} right: {:?}", v1, v2);
    }
}

#[test]
fn tokenize_space_eol() {
    let buffer = "\0\t\n\x0c\r \r\n \n\r".as_bytes();
    let mut lexer = Lexer::new(buffer).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::EOL, Token::EOL, Token::EOL, Token::EOL, Token::EOL],
    )
}

#[test]
fn tokenize_integer() {
    let buffer = " +123 -123\r".as_bytes();
    let mut lexer = Lexer::new(buffer).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::Integer(123), Token::Integer(-123), Token::EOL],
    )
}

#[test]
fn tokenize_float() {
    let buffer = "1.5 -23.4 +110.0 .5 4. -.002 0.0".as_bytes();
    let mut lexer = Lexer::new(buffer).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::Real(1.5),
            Token::Real(-23.4),
            Token::Real(110.0),
            Token::Real(0.5),
            Token::Real(4.0),
            Token::Real(-0.002),
            Token::Real(0.0),
        ],
    )
}

#[test]
fn tokenize_hex_str() {
    let buffer = "<a0e0>".as_bytes();
    let mut lexer = Lexer::new(buffer).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::HexStr("a0e0".as_bytes().to_vec())],
    )
}

#[test]
fn tokenize_string() {
    let buffer = "( (aaaa) \\) \\( \\\\)".as_bytes();
    let mut lexer = Lexer::new(buffer).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::String(" (aaaa) \\) \\( \\\\".as_bytes().to_vec())],
    )
}

#[test]
fn tokenize_array() {
    let buffer = "[123 (aa\\() -55.]".as_bytes();
    let mut lexer = Lexer::new(buffer).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::ArrayStart,
            Token::Integer(123),
            Token::String("aa\\(".as_bytes().to_vec()),
            Token::Real(-55.0),
            Token::ArrayEnd,
        ],
    )
}

#[test]
fn tokenize_indirect_obj() {
    let buffer = "1 0 R".as_bytes();
    let mut lexer = Lexer::new(buffer).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(&lexer.token_vec, &vec![Token::IndirectRef(1, 0)])
}

#[test]
fn tokenize_name() {
    let buffer = "/Name..;$@?! ".as_bytes();
    let mut lexer = Lexer::new(buffer).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::Name("Name..;$@?!".as_bytes().to_vec())],
    )
}

#[test]
fn tokenize_comment() {
    let buffer = "/Name%hogehoge /..<>(){}[]\r\n123".as_bytes();
    let mut lexer = Lexer::new(buffer).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::Name("Name".as_bytes().to_vec()), Token::Integer(123)],
    )
}

#[test]
fn tokenize_bool_null() {
    let buffer = "null true false".as_bytes();
    let mut lexer = Lexer::new(buffer).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::Null, Token::Boolean(true), Token::Boolean(false)],
    )
}
