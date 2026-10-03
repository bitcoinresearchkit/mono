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
    /// Call under the pipeline publication guard. Holding the cache lock during
    /// the rebuild prevents concurrent requests replaying the same window.
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
        *current = Some(Entry {
            tip,
            revision,
            oracle: oracle.clone(),
        });
        Ok(oracle)
    }
}
