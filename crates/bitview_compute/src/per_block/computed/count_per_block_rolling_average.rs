use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{StoredU32, StoredU64, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use crate::{CachedWindowStartVec, StoredU64ToStoredU32, Windows};

use super::PerBlockCumulativeAverage;

#[derive(Deref, DerefMut, Traversable)]
pub struct CountPerBlockRollingAverage<M: StorageMode = Rw>(
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    PerBlockCumulativeAverage<StoredU32, StoredU64, M, StoredU64ToStoredU32>,
);

impl CountPerBlockRollingAverage {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        PerBlockCumulativeAverage::forced_import(db, name, version, indexes, cached_starts)
            .map(Self)
    }
}
