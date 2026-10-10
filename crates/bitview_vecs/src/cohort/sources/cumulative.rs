use std::ops::AddAssign;

use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, Database, PcoVecValue, ReadableVec, Rw, StorageMode, WritableVec,
};

use crate::{CachedSeries, CumulativeState, import_cached};

/// A running total stored as a cumulative series; block and window views read it.
#[derive(Traversable)]
pub struct CumulativeSource<T: PcoVecValue, M: StorageMode = Rw> {
    #[traversable(flatten)]
    stored: CachedSeries<Height, T, M>,
    last: M::WriteOnly<CumulativeState<T>>,
}

impl<T: PcoVecValue + AddAssign + Default> CumulativeSource<T> {
    pub fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            stored: import_cached(db, name, version)?,
            last: Default::default(),
        })
    }

    pub fn cumulative_source(&self) -> &CachedSeries<Height, T> {
        &self.stored
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: T) {
        let len = self.stored.len();
        let cumulative =
            self.last
                .accumulate(len, || self.stored.collect_last(), |total| *total += value);
        self.stored.push(cumulative);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.last = Default::default();
        &mut self.stored
    }
}
