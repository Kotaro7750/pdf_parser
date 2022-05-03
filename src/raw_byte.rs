use crate::error;

// buffer中に表れるtargetのうち最初のものの先頭インデックスを返す
fn first_match_index(buffer: &[u8], target: &[u8]) -> Result<usize, error::Error> {
    if target.len() == 0 {
        return Ok(0);
    }

    let mut target_i = 0;

    for (i, byte) in buffer.iter().enumerate() {
        if target[target_i] == *byte {
            if target_i == target.len() - 1 {
                return Ok(i - target_i);
            }
            target_i += 1;
        } else {
            target_i = 0;
        }
    }

    Err(error::Error::TargetNotFound)
}

// buffer中に表れるtargetのうち最後のものの先頭インデックスを返す
fn last_match_index(buffer: &[u8], target: &[u8]) -> Result<usize, error::Error> {
    if target.len() == 0 {
        return Ok(buffer.len());
    }

    let mut target_i = target.len() - 1;

    for (i, byte) in buffer.iter().rev().enumerate() {
        if target[target_i] == *byte {
            if target_i == 0 {
                return Ok(buffer.len() - i - 1);
            }
            target_i -= 1;
        } else {
            target_i = target.len() - 1;
        }
    }

    Err(error::Error::TargetNotFound)
}

pub fn is_next_satisfy<F>(buffer: &[u8], i: usize, f: F) -> bool
where
    F: Fn(u8) -> bool,
{
    if (buffer.len() - 1) < (i + 1) {
        false
    } else {
        f(buffer[i + 1])
    }
}

pub fn extract_from<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = first_match_index(buffer, target)?;

    Ok(&(*buffer)[match_i..])
}

pub fn cut_from<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = first_match_index(buffer, target)?;

    Ok(&(*buffer)[..match_i])
}

pub fn extract_after<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = first_match_index(buffer, target)?;

    Ok(&(*buffer)[(match_i + target.len())..])
}

pub fn cut_after<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = first_match_index(buffer, target)?;

    Ok(&(*buffer)[..(match_i + target.len())])
}

pub fn extract_tail_from<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = last_match_index(buffer, target)?;

    Ok(&(*buffer)[match_i..])
}

pub fn cut_tail_from<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = last_match_index(buffer, target)?;

    Ok(&(*buffer)[..match_i])
}

pub fn extract_tail_after<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = last_match_index(buffer, target)?;

    Ok(&(*buffer)[(match_i + target.len())..])
}

pub fn cut_tail_after<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = last_match_index(buffer, target)?;

    Ok(&(*buffer)[..(match_i + target.len())])
}

#[cfg(test)]
mod tests {
    use super::*;

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

        if let Err(error::Error::TargetNotFound) = first_match_index(buffer, target) {
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

        if let Err(error::Error::TargetNotFound) = last_match_index(buffer, target) {
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
}
