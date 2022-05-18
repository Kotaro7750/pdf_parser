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
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(&lexer.token_vec, &vec![])
}

#[test]
fn tokenize_integer() {
    let buffer = " +123 -123\r".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::new(TokenContent::Integer(123), 1),
            Token::new(TokenContent::Integer(-123), 6),
        ],
    )
}

#[test]
fn tokenize_float() {
    let buffer = "1.5 -23.4 +110.0 .5 4. -.002 0.0".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::new(TokenContent::Real(1.5), 0),
            Token::new(TokenContent::Real(-23.4), 4),
            Token::new(TokenContent::Real(110.0), 10),
            Token::new(TokenContent::Real(0.5), 17),
            Token::new(TokenContent::Real(4.0), 20),
            Token::new(TokenContent::Real(-0.002), 23),
            Token::new(TokenContent::Real(0.0), 29),
        ],
    )
}

#[test]
fn tokenize_hex_str() {
    let buffer = "<a0e0f>".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::new(TokenContent::HexStr(vec![160, 224, 240]), 0)],
    )
}

#[test]
fn tokenize_string() {
    let buffer = "(hoge \t \\\\ \\053 (\\0053))".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::new(
            TokenContent::String(vec![
                104, 111, 103, 101, 32, 9, 32, 92, 32, 43, 32, 40, 5, 51, 41,
            ]),
            0,
        )],
    )
}

#[test]
fn tokenize_array() {
    let buffer = "[123 (aa\\() -55.]".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::new(TokenContent::ArrayStart, 0),
            Token::new(TokenContent::Integer(123), 1),
            Token::new(TokenContent::String(vec![97, 97, 40]), 5),
            Token::new(TokenContent::Real(-55.0), 12),
            Token::new(TokenContent::ArrayEnd, 16),
        ],
    )
}

#[test]
fn tokenize_indirect_ref_1() {
    let buffer = "  1 0 R".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::new(TokenContent::IndirectRef(1, 0), 2)],
    )
}

#[test]
fn tokenize_indirect_ref_2() {
    let buffer = "123 1 0 R".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::new(TokenContent::Integer(123), 0),
            Token::new(TokenContent::IndirectRef(1, 0), 4),
        ],
    )
}

#[test]
fn tokenize_name() {
    let buffer = "/Name..;$@?! ".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![Token::new(
            TokenContent::Name(String::from("Name..;$@?!")),
            0,
        )],
    )
}

#[test]
fn tokenize_comment() {
    let buffer = "/Name%hogehoge /..<>(){}[]\r\n123".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::new(TokenContent::Name(String::from("Name")), 0),
            Token::new(TokenContent::Integer(123), 28),
        ],
    )
}

#[test]
fn tokenize_boolean_null() {
    let buffer = "null true false".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::new(TokenContent::Null, 0),
            Token::new(TokenContent::Boolean(true), 5),
            Token::new(TokenContent::Boolean(false), 10),
        ],
    )
}

#[test]
fn tokenize_indirect_obj() {
    // endobjの後は強制的に停止する
    let buffer = "1 0 obj\n123 endobj   hogehoge lkjdflkj)".as_bytes();
    let mut lexer = Lexer::new(buffer, 0);

    lexer.tokenize().unwrap();

    assert_eq_token_vec(
        &lexer.token_vec,
        &vec![
            Token::new(TokenContent::IndirectObjStart(1, 0), 0),
            Token::new(TokenContent::Integer(123), 8),
            Token::new(TokenContent::IndirectObjEnd, 12),
        ],
    )
}
