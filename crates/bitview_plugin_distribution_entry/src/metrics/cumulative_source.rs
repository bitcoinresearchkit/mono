use std::ops::AddAssign;

use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, CumulativeState, import_cached};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, Database, PcoVecValue, ReadableVec, Rw, StorageMode, WritableVec,
};

/// A cumulative writer; public block and window views borrow its stored source.
#[derive(Traversable)]
pub(crate) struct CumulativeSource<T: PcoVecValue, M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub stored: CachedSeries<Height, T, M>,
    last: M::WriteOnly<CumulativeState<T>>,
}

impl<T: PcoVecValue + Copy + Default + AddAssign> CumulativeSource<T> {
    pub fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            stored: import_cached(db, name, version)?,
            last: Default::default(),
        })
    }

    pub fn cumulative_source(&self) -> &CachedSeries<Height, T> {
        &self.stored
    }

    pub fn push_block(&mut self, value: T) {
        let stored = &mut self.stored;
        let cumulative =
            self.last
                .accumulate(stored.len(), || stored.collect_last(), |sum| *sum += value);
        stored.push(cumulative);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.last = Default::default();
        &mut self.stored
    }
}
