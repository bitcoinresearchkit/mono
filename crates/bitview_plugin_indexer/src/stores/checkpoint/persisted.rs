use brk_error::Result;
use brk_exit::Exit;
use std::fs;

use super::PendingStoresCheckpoint;

#[must_use = "publish this checkpoint after every related database is complete"]
pub struct PersistedStoresCheckpoint(pub(super) PendingStoresCheckpoint);

impl PersistedStoresCheckpoint {
    pub fn publish(self) -> Result<()> {
        debug_assert!(Exit::is_locked(), "stores checkpoint published outside the exit lock");
        let pending = self.0;
        pending.next_height.write(&pending.pending_path)?;
        fs::rename(&pending.pending_path, &pending.path)?;
        Ok(())
    }
}
