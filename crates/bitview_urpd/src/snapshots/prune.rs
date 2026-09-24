use std::{fs, io::ErrorKind, path::Path};

use brk_error::Result;
use brk_types::Date;

/// Invalidate dated snapshots before recomputation. `None` removes all dates.
pub fn prune_snapshots(dir: &Path, from: Option<Date>) -> Result<()> {
    let files = match fs::read_dir(dir) {
        Ok(files) => files,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    for file in files {
        let file = file?;
        if let Some(date) = file
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<Date>().ok())
            && from.is_none_or(|from| date >= from)
            && file.file_type()?.is_file()
        {
            fs::remove_file(file.path())?;
        }
    }
    Ok(())
}
