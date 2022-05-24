use std::cmp;

#[cfg(test)]
pub mod test;

// buffer中に表れるtargetバイト列のうち先頭から見て最初のものの先頭インデックスを返す
fn first_match_index(buffer: &[u8], target: &[u8]) -> Option<usize> {
    if target.is_empty() {
        return Some(0);
    }

    let mut target_i = 0;

    for (i, byte) in buffer.iter().enumerate() {
        if target[target_i] == *byte {
            if target_i == target.len() - 1 {
                return Some(i - target_i);
            }
            target_i += 1;
        } else {
            target_i = 0;
        }
    }

    None
}

// buffer中に表れるtargetバイト列のうち先頭から見て最後のものの先頭インデックスを返す
fn last_match_index(buffer: &[u8], target: &[u8]) -> Option<usize> {
    if target.is_empty() {
        return Some(buffer.len());
    }

    let mut target_i = target.len() - 1;

    for (i, byte) in buffer.iter().rev().enumerate() {
        if target[target_i] == *byte {
            if target_i == 0 {
                return Some((buffer.len() - 1) - i);
            }
            target_i -= 1;
        } else {
            target_i = target.len() - 1;
        }
    }

    None
}

// (EOLの開始インデックス,EOLのバイト数)を返す
fn first_match_eol(buffer: &[u8]) -> Option<(usize, usize)> {
    let lf_i = first_match_index(buffer, "\n".as_bytes());

    let cr_i = first_match_index(buffer, "\r".as_bytes());

    match (cr_i, lf_i) {
        (Some(cr_i), Some(lf_i)) => {
            // CRLFはまとめて一つのEOLマーカーとみなす
            if lf_i == cr_i + 1 {
                Some((cr_i, 2))
            } else {
                Some((cmp::min(cr_i, lf_i), 1))
            }
        }
        (Some(cr_i), None) => Some((cr_i, 1)),
        (None, Some(lf_i)) => Some((lf_i, 1)),
        (None, None) => None,
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

pub fn extract_after_eol(buffer: &[u8]) -> Option<&[u8]> {
    if let Some((eol_i, eol_size)) = first_match_eol(buffer) {
        Some(&buffer[(eol_i + eol_size)..])
    } else {
        None
    }
}

pub fn cut_after_eol(buffer: &[u8]) -> Option<&[u8]> {
    if let Some((eol_i, _)) = first_match_eol(buffer) {
        Some(&buffer[..eol_i])
    } else {
        None
    }
}

pub fn extract_from_eol(buffer: &[u8]) -> Option<&[u8]> {
    if let Some((eol_i, _)) = first_match_eol(buffer) {
        Some(&buffer[eol_i..])
    } else {
        None
    }
}

pub fn extract_from<'a>(buffer: &'a [u8], target: &[u8]) -> Option<&'a [u8]> {
    if let Some(match_i) = first_match_index(buffer, target) {
        Some(&buffer[match_i..])
    } else {
        None
    }
}

pub fn cut_from<'a>(buffer: &'a [u8], target: &[u8]) -> Option<&'a [u8]> {
    if let Some(match_i) = first_match_index(buffer, target) {
        Some(&buffer[..match_i])
    } else {
        None
    }
}

pub fn extract_after<'a>(buffer: &'a [u8], target: &[u8]) -> Option<&'a [u8]> {
    if let Some(match_i) = first_match_index(buffer, target) {
        Some(&buffer[(match_i + target.len())..])
    } else {
        None
    }
}

pub fn cut_after<'a>(buffer: &'a [u8], target: &[u8]) -> Option<&'a [u8]> {
    if let Some(match_i) = first_match_index(buffer, target) {
        Some(&buffer[..(match_i + target.len())])
    } else {
        None
    }
}

pub fn extract_tail_from<'a>(buffer: &'a [u8], target: &[u8]) -> Option<&'a [u8]> {
    if let Some(match_i) = last_match_index(buffer, target) {
        Some(&buffer[match_i..])
    } else {
        None
    }
}

pub fn cut_tail_from<'a>(buffer: &'a [u8], target: &[u8]) -> Option<&'a [u8]> {
    if let Some(match_i) = last_match_index(buffer, target) {
        Some(&buffer[..match_i])
    } else {
        None
    }
}

pub fn extract_tail_after<'a>(buffer: &'a [u8], target: &[u8]) -> Option<&'a [u8]> {
    if let Some(match_i) = last_match_index(buffer, target) {
        Some(&buffer[(match_i + target.len())..])
    } else {
        None
    }
}

pub fn cut_tail_after<'a>(buffer: &'a [u8], target: &[u8]) -> Option<&'a [u8]> {
    if let Some(match_i) = last_match_index(buffer, target) {
        Some(&buffer[..(match_i + target.len())])
    } else {
        None
    }
}
