use bitview_collections::Windows;
use bitview_primitives::{StoredU32, StoredU64};
use bitview_transforms::StoredU64ToStoredU32;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};

use crate::{IndexSources, PerBlockCumulativeAverage};

#[derive(Deref, DerefMut, Traversable)]
pub struct CountPerBlockRollingAverage<M: StorageMode = Rw>(
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    PerBlockCumulativeAverage<StoredU32, StoredU64, M, StoredU64ToStoredU32>,
);

impl CountPerBlockRollingAverage {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        PerBlockCumulativeAverage::import(db, name, version, indexes, window_starts).map(Self)
    }
}
