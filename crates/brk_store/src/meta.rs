use std::{
    fs,
    path::{Path, PathBuf},
};

use brk_error::Error;
use brk_types::Version;
use fjall::Keyspace;

#[derive(Debug, Clone)]
pub(super) struct StoreMeta {
    pathbuf: PathBuf,
}

impl StoreMeta {
    pub(super) fn checked_open<F>(
        path: &Path,
        version: Version,
        open_partition_handle: F,
    ) -> brk_error::Result<(Self, Keyspace)>
    where
        F: Fn() -> brk_error::Result<Keyspace>,
    {
        fs::create_dir_all(path)?;

        let partition = open_partition_handle()?;

        if let Ok(prev_version) = Version::try_from(Self::path_version_(path).as_path())
            && version != prev_version
        {
            return Err(Error::VersionMismatch {
                path: path.to_path_buf(),
                expected: usize::from(version),
                found: usize::from(prev_version),
            });
        }

        let slf = Self {
            pathbuf: path.to_owned(),
        };

        version.write(&slf.path_version())?;

        Ok((slf, partition))
    }

    pub(super) fn path(&self) -> &Path {
        &self.pathbuf
    }

    fn path_version(&self) -> PathBuf {
        Self::path_version_(&self.pathbuf)
    }
    fn path_version_(path: &Path) -> PathBuf {
        path.join("version")
    }
}
