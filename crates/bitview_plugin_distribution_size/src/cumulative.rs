use std::ops::AddAssign;

use crate::values::SizeValues;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, PcoVecValue, Rw, StorageMode};

use crate::sources::SizeSources;
use bitview_vecs::CumulativeState;

#[derive(Traversable)]
pub struct CumulativeSizeSources<T, M: StorageMode = Rw>
where
    T: PcoVecValue,
{
    #[traversable(flatten)]
    pub stored: SizeSources<T, M>,
    last: M::WriteOnly<CumulativeState<SizeValues<T>>>,
}

impl<T> CumulativeSizeSources<T>
where
    T: PcoVecValue + AddAssign + Copy + Default,
{
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            stored: SizeSources::forced_import(db, name, version)?,
            last: Default::default(),
        })
    }

    #[inline(always)]
    pub fn push_block(&mut self, cohort_values: impl Into<SizeValues<T>>) {
        let len = self.stored.min_len();
        let cumulative = self.last.accumulate(
            len,
            || self.stored.collect_last(),
            |values| *values += cohort_values.into(),
        );
        self.stored.push(cumulative);
    }

    pub fn min_len(&self) -> usize {
        self.stored.min_len()
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.last = Default::default();
        self.stored.stored_vecs_mut()
    }
}
