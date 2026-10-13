use brk_oracle::Oracle;
use brk_types::BlockHash;
use parking_lot::Mutex;

use crate::Result;

/// One warmed confirmed window. No response bodies or mempool-derived state.
#[derive(Default)]
pub struct LiveOracle {
    current: Mutex<Option<Entry>>,
}

struct Entry {
    tip: BlockHash,
    revision: u64,
    oracle: Oracle,
}

impl LiveOracle {
    /// `rebuild` reads the publication `revision` names (the caller's pin); a request still on an
    /// older publication never replaces a newer entry. Holding the cache lock during the rebuild
    /// prevents concurrent requests replaying the same window.
    pub fn get_or_try_init(
        &self,
        tip: BlockHash,
        revision: u64,
        rebuild: impl FnOnce() -> Result<Oracle>,
    ) -> Result<Oracle> {
        let mut current = self.current.lock();
        if let Some(entry) = current.as_ref()
            && entry.tip == tip
            && entry.revision == revision
        {
            return Ok(entry.oracle.clone());
        }
        let oracle = rebuild()?;
        if current
            .as_ref()
            .is_none_or(|entry| revision > entry.revision)
        {
            *current = Some(Entry {
                tip,
                revision,
                oracle: oracle.clone(),
            });
        }
        Ok(oracle)
    }
}
