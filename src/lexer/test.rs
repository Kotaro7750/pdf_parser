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
    let mut lexer = Lexer::new(buffer, 0).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(&lexer.token_vec, &vec![])
}

#[test]
fn tokenize_integer() {
    let buffer = " +123 -123\r".as_bytes();
    let mut lexer = Lexer::new(buffer, 0).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::Integer(123), Token::Integer(-123)],
    )
}

#[test]
fn tokenize_float() {
    let buffer = "1.5 -23.4 +110.0 .5 4. -.002 0.0".as_bytes();
    let mut lexer = Lexer::new(buffer, 0).unwrap();

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
    let buffer = "<a0e0f>".as_bytes();
    let mut lexer = Lexer::new(buffer, 0).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(&lexer.token_vec, &vec![Token::HexStr(vec![160, 224, 240])])
}

#[test]
fn tokenize_string() {
    let buffer = "(hoge \t \\\\ \\053 (\\0053))".as_bytes();
    let mut lexer = Lexer::new(buffer, 0).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::String(vec![
            104, 111, 103, 101, 32, 9, 32, 92, 32, 43, 32, 40, 5, 51, 41,
        ])],
    )
}

#[test]
fn tokenize_array() {
    let buffer = "[123 (aa\\() -55.]".as_bytes();
    let mut lexer = Lexer::new(buffer, 0).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::ArrayStart,
            Token::Integer(123),
            Token::String(vec![97, 97, 40]),
            Token::Real(-55.0),
            Token::ArrayEnd,
        ],
    )
}

#[test]
fn tokenize_indirect_ref() {
    let buffer = "1 0 R".as_bytes();
    let mut lexer = Lexer::new(buffer, 0).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(&lexer.token_vec, &vec![Token::IndirectRef(1, 0)])
}

#[test]
fn tokenize_name() {
    let buffer = "/Name..;$@?! ".as_bytes();
    let mut lexer = Lexer::new(buffer, 0).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::Name(String::from("Name..;$@?!"))],
    )
}

#[test]
fn tokenize_comment() {
    let buffer = "/Name%hogehoge /..<>(){}[]\r\n123".as_bytes();
    let mut lexer = Lexer::new(buffer, 0).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::Name(String::from("Name")), Token::Integer(123)],
    )
}

#[test]
fn tokenize_bool_null() {
    let buffer = "null true false".as_bytes();
    let mut lexer = Lexer::new(buffer, 0).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::Null, Token::Boolean(true), Token::Boolean(false)],
    )
}

#[test]
fn tokenize_indirect_obj() {
    // endobjの後は強制的に停止する
    let buffer = "1 0 obj\n123 endobj   hogehoge lkjdflkj)".as_bytes();
    let mut lexer = Lexer::new(buffer, 0).unwrap();

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::IndirectObjStart(1, 0),
            Token::Integer(123),
            Token::IndirectObjEnd,
        ],
    )
}
