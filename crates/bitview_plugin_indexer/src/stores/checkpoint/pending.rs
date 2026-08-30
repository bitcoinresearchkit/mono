use brk_error::Result;
use brk_types::Height;
use std::path::PathBuf;

use super::PersistedStoresCheckpoint;

#[must_use = "dropping a pending checkpoint leaves the stores checkpoint invalid"]
pub(crate) struct PendingStoresCheckpoint {
    pub(super) next_height: Height,
    pub(super) path: PathBuf,
    pub(super) pending_path: PathBuf,
}

impl PendingStoresCheckpoint {
    pub(crate) fn persist(
        self,
        ingest: impl FnOnce() -> Result<()>,
    ) -> Result<PersistedStoresCheckpoint> {
        ingest()?;
        Ok(PersistedStoresCheckpoint(self))
    }
}
