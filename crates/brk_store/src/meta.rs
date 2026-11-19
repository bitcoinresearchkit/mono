use std::{fs, path::Path};

use brk_error::{Error, Result};
use brk_types::Version;
use fjall::Keyspace;

/// Opens the keyspace and checks its version marker: a fresh keyspace gets one, an existing one
/// must match it. An unreadable marker, or data without one, is an error.
pub fn checked_open(
    path: &Path,
    version: Version,
    open_keyspace: impl FnOnce() -> Result<Keyspace>,
) -> Result<Keyspace> {
    fs::create_dir_all(path).map_err(|error| Error::from(error).opening_storage(path))?;

    let keyspace = open_keyspace()?;
    check_marker(path, version, &keyspace).map_err(|error| error.opening_storage(path))?;
    Ok(keyspace)
}

fn check_marker(path: &Path, version: Version, keyspace: &Keyspace) -> Result<()> {
    let marker = path.join("version");

    if !marker.try_exists()? {
        if !keyspace.is_empty()? {
            return Err(Error::MissingVersion {
                path: path.to_path_buf(),
            });
        }
        version.write(&marker)?;
        return Ok(());
    }

    let found = Version::try_from(marker.as_path())?;
    if found != version {
        return Err(Error::VersionMismatch {
            path: path.to_path_buf(),
            expected: usize::from(version),
            found: usize::from(found),
        });
    }

    Ok(())
}
