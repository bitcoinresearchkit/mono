use brk_cohort::AddrTypeId;
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{StoredU64, Version};
use derive_more::{Deref, DerefMut};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, Database, Rw, StorageMode, WritableVec};

use crate::{
    indexes,
    internal::{ColumnarPerBlock, LazyColumnPerBlock, LazyPerBlock, WithAddrTypes},
};

use super::AddrTypeToAddrCount;

/// Per-block `StoredU64` counts with an aggregate `all` plus a per-address-type
/// breakdown. Shared primitive backing addr-count, empty-addr-count, and the
/// funded/total pairs used by exposed, reused, and respent.
#[derive(Deref, DerefMut, Traversable)]
pub struct AddrCountsVecs<M: StorageMode = Rw>(
    #[traversable(flatten)]
    pub  ColumnarPerBlock<
        StoredU64,
        AddrTypeId,
        WithAddrTypes<LazyColumnPerBlock<StoredU64, AddrTypeId>, LazyPerBlock<StoredU64>>,
        M,
    >,
);

impl AddrCountsVecs {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        Ok(Self(ColumnarPerBlock::forced_import(
            db,
            &format!("{name}_by_type"),
            version,
            |source| WithAddrTypes::from_columnar_source(name, version, source, indexes),
        )?))
    }

    pub fn min_stateful_len(&self) -> usize {
        self.height.len()
    }

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        rayon::iter::once(&mut self.height as &mut dyn AnyStoredVec)
    }

    pub fn reset_height(&mut self) -> Result<()> {
        self.height.reset()?;
        Ok(())
    }

    #[inline(always)]
    pub fn push_counts(&mut self, counts: &AddrTypeToAddrCount) {
        self.push(counts.row());
    }
}
