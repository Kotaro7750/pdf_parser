use super::*;

#[test]
fn skip_eol_1() {
    let buffer = "  kjkj\n".as_bytes();

    let buffer = extract_after_eol(buffer).unwrap();
    assert_eq!(buffer, "".as_bytes());
}

#[test]
fn extract_after_eol_2() {
    let buffer = "  hogehoeg \r\nhoge".as_bytes();

    let buffer = extract_after_eol(buffer).unwrap();
    assert_eq!(buffer, "hoge".as_bytes());
}

#[test]
fn extract_after_eol_3() {
    let buffer = "hoge\r   \nfuga".as_bytes();

    let buffer = extract_after_eol(buffer).unwrap();
    assert_eq!(buffer, "   \nfuga".as_bytes());
}

#[test]
fn first_match_index_1() {
    let buffer = "0123abcd\n { target\r hoge".as_bytes();
    let target = "target".as_bytes();

    let i = first_match_index(buffer, target).unwrap();
    assert_eq!(i, 12);
}

#[test]
fn first_match_index_2() {
    let buffer = "0123abcd\n { target\r hoge".as_bytes();
    let target = "".as_bytes();

    let i = first_match_index(buffer, target).unwrap();
    assert_eq!(i, 0);
}

#[test]
fn first_match_index_3() {
    let buffer = "hogehoge".as_bytes();
    let target = "too long target ".as_bytes();

    if let Err(error::Error::TargetNotFound(_)) = first_match_index(buffer, target) {
    } else {
        panic!();
    }
}

#[test]
fn last_match_index_1() {
    let buffer = "0123abcd\n { target\r target2 hoge".as_bytes();
    let target = "target".as_bytes();

    let i = last_match_index(buffer, target).unwrap();
    assert_eq!(i, 20);
}

#[test]
fn last_match_index_2() {
    let buffer = "0123abcd\n { target\r hoge".as_bytes();
    let target = "".as_bytes();

    let i = last_match_index(buffer, target).unwrap();
    assert_eq!(i, 24);
}

#[test]
fn last_match_index_3() {
    let buffer = "hogehoge".as_bytes();
    let target = "too long target ".as_bytes();

    if let Err(error::Error::TargetNotFound(_)) = last_match_index(buffer, target) {
    } else {
        panic!();
    }
}

#[test]
fn extract_from_1() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "target".as_bytes();

    let i = extract_from(buffer, target).unwrap();
    assert_eq!(i, "target jjjj\n\rhoge".as_bytes());
}

#[test]
fn extract_from_2() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "".as_bytes();

    let i = extract_from(buffer, target).unwrap();
    assert_eq!(i, "hogehoge target jjjj\n\rhoge".as_bytes());
}

#[test]
fn cut_from_1() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "target".as_bytes();

    let i = cut_from(buffer, target).unwrap();
    assert_eq!(i, "hogehoge ".as_bytes());
}

#[test]
fn cut_from_2() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "".as_bytes();

    let i = cut_from(buffer, target).unwrap();
    assert_eq!(i, "".as_bytes());
}

#[test]
fn extract_after_1() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "target".as_bytes();

    let i = extract_after(buffer, target).unwrap();
    assert_eq!(i, " jjjj\n\rhoge".as_bytes());
}

#[test]
fn extract_after_2() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "".as_bytes();

    let i = extract_after(buffer, target).unwrap();
    assert_eq!(i, "hogehoge target jjjj\n\rhoge".as_bytes());
}

#[test]
fn cut_after_1() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "target".as_bytes();

    let i = cut_after(buffer, target).unwrap();
    assert_eq!(i, "hogehoge target".as_bytes());
}

#[test]
fn cut_after_2() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "".as_bytes();

    let i = cut_after(buffer, target).unwrap();
    assert_eq!(i, "".as_bytes());
}

#[test]
fn extract_tail_from_1() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "target".as_bytes();

    let i = extract_tail_from(buffer, target).unwrap();
    assert_eq!(i, "target jjjj\n\rhoge".as_bytes());
}

#[test]
fn extract_tail_from_2() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "".as_bytes();

    let i = extract_tail_from(buffer, target).unwrap();
    assert_eq!(i, "".as_bytes());
}

#[test]
fn cut_tail_from_1() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "target".as_bytes();

    let i = cut_tail_from(buffer, target).unwrap();
    assert_eq!(i, "hogehoge ".as_bytes());
}

#[test]
fn cut_tail_from_2() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "".as_bytes();

    let i = cut_tail_from(buffer, target).unwrap();
    assert_eq!(i, "hogehoge target jjjj\n\rhoge".as_bytes());
}

#[test]
fn extract_tail_after_1() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "target".as_bytes();

    let i = extract_tail_after(buffer, target).unwrap();
    assert_eq!(i, " jjjj\n\rhoge".as_bytes());
}

#[test]
fn extract_tail_after_2() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "".as_bytes();

    let i = extract_tail_after(buffer, target).unwrap();
    assert_eq!(i, "".as_bytes());
}

#[test]
fn cut_tail_after_1() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "target".as_bytes();

    let i = cut_tail_after(buffer, target).unwrap();
    assert_eq!(i, "hogehoge target".as_bytes());
}

#[test]
fn cut_tail_after_2() {
    let buffer = "hogehoge target jjjj\n\rhoge".as_bytes();
    let target = "".as_bytes();

    let i = cut_tail_after(buffer, target).unwrap();
    assert_eq!(i, "hogehoge target jjjj\n\rhoge".as_bytes());
}
