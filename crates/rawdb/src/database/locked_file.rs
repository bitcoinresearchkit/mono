use std::{fs::File, path::Path};

use crate::Result;

/// Opens persistent storage without truncating it and excludes other instances.
pub(crate) fn open_locked_file(path: &Path) -> Result<File> {
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    file.try_lock()?;
    Ok(file)
}
