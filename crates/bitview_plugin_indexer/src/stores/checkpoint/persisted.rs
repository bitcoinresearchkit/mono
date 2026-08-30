use brk_error::Result;
use std::fs;

use super::PendingStoresCheckpoint;

#[must_use = "publish this checkpoint after every related database is complete"]
pub(crate) struct PersistedStoresCheckpoint(pub(super) PendingStoresCheckpoint);

impl PersistedStoresCheckpoint {
    pub(crate) fn publish(self) -> Result<()> {
        let pending = self.0;
        pending.next_height.write(&pending.pending_path)?;
        fs::rename(&pending.pending_path, &pending.path)?;
        Ok(())
    }
}
