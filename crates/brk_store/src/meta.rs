use std::{fs, path::Path};

use brk_error::{Error, Result};
use brk_types::Version;
use fjall::Keyspace;

/// Opens the keyspace and checks its version marker: a fresh keyspace gets one, an existing one
/// must match it. An unreadable marker, or data without one, is an error.
pub fn checked_open<F>(path: &Path, version: Version, open_partition_handle: F) -> Result<Keyspace>
where
    F: Fn() -> Result<Keyspace>,
{
    fs::create_dir_all(path)?;

    let partition = open_partition_handle()?;
    let marker = path.join("version");

    if !marker.try_exists()? {
        if !partition.is_empty()? {
            return Err(Error::MissingVersion {
                path: path.to_path_buf(),
            });
        }
        version.write(&marker)?;
        return Ok(partition);
    }

    let found = Version::try_from(marker.as_path())?;
    if found != version {
        return Err(Error::VersionMismatch {
            path: path.to_path_buf(),
            expected: usize::from(version),
            found: usize::from(found),
        });
    }

    Ok(partition)
}
