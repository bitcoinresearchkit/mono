use arc_swap::ArcSwapOption;
use brk_oracle::Oracle;
use brk_types::BlockHash;
use parking_lot::Mutex;

use crate::{Error, Result};

/// One warmed confirmed window. No response bodies or mempool-derived state.
#[derive(Default)]
pub struct LiveOracle {
    current: ArcSwapOption<Entry>,
    /// Held by the one request rebuilding the window; the others never wait for it.
    building: Mutex<()>,
}

struct Entry {
    tip: BlockHash,
    revision: u64,
    oracle: Oracle,
}

impl LiveOracle {
    /// `rebuild` reads the publication `revision` names (the caller's pin); a request still on an
    /// older publication never replaces a newer entry. One request rebuilds at a time: the
    /// others missing the entry meanwhile get `StateUpdating` rather than replaying the window.
    pub fn get_or_try_init(
        &self,
        tip: BlockHash,
        revision: u64,
        rebuild: impl FnOnce() -> Result<Oracle>,
    ) -> Result<Oracle> {
        let matches = |entry: &Entry| entry.tip == tip && entry.revision == revision;
        if let Some(entry) = self.current.load().as_deref()
            && matches(entry)
        {
            return Ok(entry.oracle.clone());
        }
        let _building = self.building.try_lock().ok_or(Error::StateUpdating)?;
        // Every store happens under `building`: this load sees the latest entry.
        let current = self.current.load_full();
        if let Some(entry) = current.as_deref()
            && matches(entry)
        {
            return Ok(entry.oracle.clone());
        }
        let oracle = rebuild()?;
        if current.is_none_or(|entry| revision > entry.revision) {
            self.current.store(Some(
                Entry {
                    tip,
                    revision,
                    oracle: oracle.clone(),
                }
                .into(),
            ));
        }
        Ok(oracle)
    }
}
