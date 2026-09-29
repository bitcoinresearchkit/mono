use std::{
    fs::{self, File, OpenOptions},
    io::{Error, ErrorKind, Result, Write},
    path::Path,
};

pub(crate) fn invalid(message: &str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let next = path.with_extension("next");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&next)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&next, path)?;
    File::open(path.parent().unwrap())?.sync_all()
}
pub(crate) fn checked(bytes: &[u8]) -> Result<&[u8]> {
    if bytes.len() < 4 {
        return Err(invalid("truncated checksum"));
    }
    let (body, tail) = bytes.split_at(bytes.len() - 4);
    if crc32fast::hash(body) != u32::from_le_bytes(tail.try_into().unwrap()) {
        return Err(invalid("checksum mismatch"));
    }
    Ok(body)
}
pub(crate) fn checksum(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&crc32fast::hash(bytes).to_le_bytes());
}

/// Publish another name for a durable, immutable snapshot without encoding it again.
pub(crate) fn atomic_link(source: &Path, target: &Path) -> Result<()> {
    let next = target.with_extension("next");
    match fs::remove_file(&next) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    fs::hard_link(source, &next)?;
    fs::rename(&next, target)?;
    File::open(target.parent().unwrap())?.sync_all()
}
