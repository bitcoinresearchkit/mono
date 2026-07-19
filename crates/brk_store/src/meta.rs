use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use brk_error::{Error, Result};
use brk_types::Version;
use fjall::Keyspace;

use super::Height;

#[derive(Debug, Clone)]
pub struct StoreMeta {
    pathbuf: PathBuf,
    version: Version,
    height: Arc<AtomicU64>,
}

impl StoreMeta {
    const NO_HEIGHT: u64 = u64::MAX;

    pub fn checked_open<F>(
        path: &Path,
        version: Version,
        open_partition_handle: F,
    ) -> Result<(Self, Keyspace)>
    where
        F: Fn() -> Result<Keyspace>,
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
            version,
            height: Arc::new(AtomicU64::new(
                Height::try_from(Self::path_height_(path).as_path())
                    .map(u64::from)
                    .unwrap_or(Self::NO_HEIGHT),
            )),
        };

        slf.version.write(&slf.path_version())?;

        Ok((slf, partition))
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn export(&self, height: Height) -> io::Result<()> {
        height.write(&self.path_height())?;
        self.height.store(height.into(), Ordering::Release);
        Ok(())
    }

    pub fn export_sync(&self, height: Height) -> io::Result<()> {
        let path = self.path_height();
        height.write(&path)?;
        fs::File::open(path)?.sync_data()?;
        self.height.store(height.into(), Ordering::Release);
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.pathbuf
    }

    fn path_version(&self) -> PathBuf {
        Self::path_version_(&self.pathbuf)
    }
    fn path_version_(path: &Path) -> PathBuf {
        path.join("version")
    }

    #[inline]
    pub fn height(&self) -> Option<Height> {
        let height = self.height.load(Ordering::Acquire);
        (height != Self::NO_HEIGHT).then(|| Height::from(height))
    }
    #[inline]
    pub fn needs(&self, height: Height) -> bool {
        self.height().is_none_or(|self_height| height > self_height)
    }
    #[inline]
    pub fn has(&self, height: Height) -> bool {
        !self.needs(height)
    }
    pub fn reset(&self) -> io::Result<()> {
        let path = self.path_height();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        self.height.store(Self::NO_HEIGHT, Ordering::Release);
        Ok(())
    }
    fn path_height(&self) -> PathBuf {
        Self::path_height_(&self.pathbuf)
    }
    fn path_height_(path: &Path) -> PathBuf {
        path.join("height")
    }
}
