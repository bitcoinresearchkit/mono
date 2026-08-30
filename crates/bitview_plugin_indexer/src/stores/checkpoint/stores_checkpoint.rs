use brk_error::Result;
use brk_types::Height;
use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use super::{PendingStoresCheckpoint, PersistedStoresCheckpoint};

/// Sole completion marker for the shared stores database.
///
/// The file contains the next block height to index. A missing or malformed
/// file means a commit was interrupted and the stores must not be resumed.
#[derive(Debug, Clone)]
pub(crate) struct StoresCheckpoint {
    pub(super) path: PathBuf,
}

impl StoresCheckpoint {
    pub(crate) fn new(stores_path: &Path) -> Self {
        Self {
            path: stores_path.join("height"),
        }
    }

    pub(crate) fn next_height(&self) -> Result<Option<Height>> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let Ok(bytes) = <[u8; size_of::<u32>()]>::try_from(bytes) else {
            return Ok(None);
        };
        Ok(Some(Height::new(u32::from_le_bytes(bytes))))
    }

    pub(crate) fn initialize_empty(&self) -> Result<()> {
        let pending_path = self.invalidate()?;
        PersistedStoresCheckpoint(PendingStoresCheckpoint {
            next_height: Height::ZERO,
            path: self.path.clone(),
            pending_path,
        })
        .publish()
    }

    pub(crate) fn begin(&self, completed_height: Height) -> Result<PendingStoresCheckpoint> {
        let pending_path = self.invalidate()?;

        Ok(PendingStoresCheckpoint {
            next_height: completed_height.incremented(),
            path: self.path.clone(),
            pending_path,
        })
    }

    fn invalidate(&self) -> Result<PathBuf> {
        let pending_path = self.path.with_extension("pending");
        remove_if_exists(&self.path)?;
        remove_if_exists(&pending_path)?;

        Ok(pending_path)
    }
}

fn remove_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}
