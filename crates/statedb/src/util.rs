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
