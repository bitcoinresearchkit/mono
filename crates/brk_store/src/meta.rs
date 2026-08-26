use std::{
    fs,
    path::{Path, PathBuf},
};

use brk_error::{Error, Result};
use brk_types::Version;
use fjall::Keyspace;

pub fn checked_open<F>(path: &Path, version: Version, open_partition_handle: F) -> Result<Keyspace>
where
    F: Fn() -> Result<Keyspace>,
{
    fs::create_dir_all(path)?;

    let partition = open_partition_handle()?;

    if let Ok(prev_version) = Version::try_from(version_path(path).as_path())
        && version != prev_version
    {
        return Err(Error::VersionMismatch {
            path: path.to_path_buf(),
            expected: usize::from(version),
            found: usize::from(prev_version),
        });
    }

    version.write(&version_path(path))?;

    Ok(partition)
}

fn version_path(path: &Path) -> PathBuf {
    path.join("version")
}
