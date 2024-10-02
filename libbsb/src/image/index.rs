use std::io::BufRead;
use std::io::{self, Seek, SeekFrom};

#[allow(clippy::module_name_repetitions)]
pub fn read_index(
    _typein: i32,
    file: &mut (impl BufRead + Seek),
    height: u16,
) -> io::Result<(Vec<u64>, u64)> {
    let end_of_index = file.seek(SeekFrom::End(-4))?;
    // so the offset is kept in the last 4 bytes of the file
    // in the form of 4 successive u8s. imgkap states that it uses big-endian
    let mut offset = [0; 4];
    file.read_exact(&mut offset)?;
    let start_of_index = u64::from(u32::from_be_bytes(offset));
    if (end_of_index - start_of_index) / 4 != u64::from(height) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid index table size",
        ));
    }

    let mut index = vec![0u64; height as usize];

    file.seek(SeekFrom::Start(start_of_index))?;

    for index_element in index.iter_mut().take(usize::from(height)) {
        let mut buf = [0; 4];
        file.read_exact(&mut buf)?;
        *index_element = u64::from(u32::from_be_bytes(buf));
    }

    Ok((index, start_of_index))
}
