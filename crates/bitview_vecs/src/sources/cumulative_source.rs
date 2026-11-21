use std::ops::AddAssign;

use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, PcoVecValue, Rw, StorageMode};

use crate::{CachedSeries, CumulativeState, IndexSources, import_cached};

/// A running total stored as a cumulative series; block and window views read it.
#[derive(Traversable)]
pub struct CumulativeSource<T: PcoVecValue, M: StorageMode = Rw> {
    #[traversable(flatten)]
    stored: CachedSeries<T, M>,
    last: M::WriteOnly<CumulativeState<T>>,
}

impl<T: PcoVecValue + AddAssign + Default> CumulativeSource<T> {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self {
            stored: import_cached(db, name, version, indexes)?,
            last: Default::default(),
        })
    }

    pub fn cumulative_source(&self) -> &CachedSeries<T> {
        &self.stored
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: T) {
        self.last.push(&mut self.stored, value);
    }

    /// A pure borrow: a length change reloads the cached total.
    pub(crate) fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.stored
    }
}
