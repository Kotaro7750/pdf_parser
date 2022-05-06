use std::cmp;

pub mod error;
#[cfg(test)]
pub mod test;

// buffer中に表れるtargetバイト列のうち先頭から見て最初のものの先頭インデックスを返す
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

    Err(error::Error::TargetNotFound(target.to_vec()))
}

// buffer中に表れるtargetバイト列のうち先頭から見て最後のものの先頭インデックスを返す
fn last_match_index(buffer: &[u8], target: &[u8]) -> Result<usize, error::Error> {
    if target.len() == 0 {
        return Ok(buffer.len());
    }

    let mut target_i = target.len() - 1;

    for (i, byte) in buffer.iter().rev().enumerate() {
        if target[target_i] == *byte {
            if target_i == 0 {
                return Ok((buffer.len() - 1) - i);
            }
            target_i -= 1;
        } else {
            target_i = target.len() - 1;
        }
    }

    Err(error::Error::TargetNotFound(target.to_vec()))
}

// (EOLの開始インデックス,EOLのバイト数)を返す
fn first_match_eol(buffer: &[u8]) -> Result<(usize, usize), error::Error> {
    let lf_i = match first_match_index(buffer, "\n".as_bytes()) {
        Ok(i) => Some(i),
        _ => None,
    };

    let cr_i = match first_match_index(buffer, "\r".as_bytes()) {
        Ok(i) => Some(i),
        _ => None,
    };

    match (cr_i, lf_i) {
        (Some(cr_i), Some(lf_i)) => {
            // CRLFはまとめて一つのEOLマーカーとみなす
            if lf_i == cr_i + 1 {
                Ok((cr_i, 2))
            } else {
                Ok((cmp::min(cr_i, lf_i), 1))
            }
        }
        (Some(cr_i), None) => Ok((cr_i, 1)),
        (None, Some(lf_i)) => Ok((lf_i, 1)),
        (None, None) => Err(error::Error::EOLNotFound),
    }
}

pub fn is_next_satisfy<F>(buffer: &[u8], i: usize, f: F) -> bool
where
    F: Fn(u8) -> bool,
{
    if (buffer.len() - 1) <= i {
        false
    } else {
        f(buffer[i + 1])
    }
}

pub fn extract_after_eol(buffer: &[u8]) -> Result<&[u8], error::Error> {
    let (eol_i, eol_size) = first_match_eol(buffer)?;

    Ok(&buffer[(eol_i + eol_size)..])
}

pub fn extract_from_eol(buffer: &[u8]) -> Result<&[u8], error::Error> {
    let (eol_i, _) = first_match_eol(buffer)?;

    Ok(&buffer[eol_i..])
}

pub fn extract_from<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = first_match_index(buffer, target)?;

    Ok(&buffer[match_i..])
}

pub fn cut_from<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = first_match_index(buffer, target)?;

    Ok(&buffer[..match_i])
}

pub fn extract_after<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = first_match_index(buffer, target)?;

    Ok(&buffer[(match_i + target.len())..])
}

pub fn cut_after<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = first_match_index(buffer, target)?;

    Ok(&buffer[..(match_i + target.len())])
}

pub fn extract_tail_from<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = last_match_index(buffer, target)?;

    Ok(&buffer[match_i..])
}

pub fn cut_tail_from<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = last_match_index(buffer, target)?;

    Ok(&buffer[..match_i])
}

pub fn extract_tail_after<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = last_match_index(buffer, target)?;

    Ok(&buffer[(match_i + target.len())..])
}

pub fn cut_tail_after<'a>(buffer: &'a [u8], target: &[u8]) -> Result<&'a [u8], error::Error> {
    let match_i = last_match_index(buffer, target)?;

    Ok(&buffer[..(match_i + target.len())])
}
