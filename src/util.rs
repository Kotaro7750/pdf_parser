use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

pub fn read_partially(file: &mut File, offset: u64, size: u64) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = vec![0; size as usize];

    file.seek(SeekFrom::Start(offset))?;

    let n = file.read(&mut buffer)?;
    if (n as u64) < size {
        buffer.truncate(n);
    }

    Ok(buffer)
}
