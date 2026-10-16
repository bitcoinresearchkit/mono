use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

use super::features::{CountVecs, TransactionCounts};

/// Per-block transaction counts by version.
#[derive(Traversable)]
pub struct VersionCountVecs<M: StorageMode = Rw> {
    /// Transactions whose version is exactly 1.
    pub v1: CountVecs<M>,
    /// Transactions whose version is exactly 2.
    pub v2: CountVecs<M>,
    /// Transactions whose version is exactly 3.
    pub v3: CountVecs<M>,
    /// Transactions whose version is not 1, 2, or 3. This category combines
    /// every other value; use individual raw transaction data to inspect the
    /// original version.
    pub other: CountVecs<M>,
}

impl VersionCountVecs {
    pub fn import(db: &Database, version: Version) -> Result<Self> {
        let (v1, v2, v3, other) = parallel_import! {
            v1 = CountVecs::import(db, "v1_tx_count", version),
            v2 = CountVecs::import(db, "v2_tx_count", version),
            v3 = CountVecs::import(db, "v3_tx_count", version),
            other = CountVecs::import(db, "other_version_tx_count", version),
        };
        Ok(Self { v1, v2, v3, other })
    }

    pub fn push(&mut self, height: Height, counts: &TransactionCounts) {
        for (vec, count) in [
            (&mut self.v1, counts.v1),
            (&mut self.v2, counts.v2),
            (&mut self.v3, counts.v3),
            (&mut self.other, counts.other_version),
        ] {
            vec.block.debug_checked_push(height, count.into());
        }
    }

    pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
        [
            &*self.v1.block as &dyn AnyStoredVec,
            &*self.v2.block,
            &*self.v3.block,
            &*self.other.block,
        ]
        .into_iter()
    }
}
