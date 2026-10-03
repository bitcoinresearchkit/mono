use std::ops::AddAssign;

use crate::values::UtxoValues;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, PcoVecValue, Rw, StorageMode};

use crate::sources::UtxoSources;
use bitview_vecs::CumulativeState;

#[derive(Traversable)]
pub struct CumulativeUtxoSources<T, M: StorageMode = Rw>
where
    T: PcoVecValue,
{
    #[traversable(flatten)]
    pub stored: UtxoSources<T, M>,
    last: M::WriteOnly<CumulativeState<UtxoValues<T>>>,
}

impl<T> CumulativeUtxoSources<T>
where
    T: PcoVecValue + AddAssign + Copy + Default,
{
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            stored: UtxoSources::forced_import(db, name, version)?,
            last: Default::default(),
        })
    }

    #[inline(always)]
    pub fn push_block(&mut self, cohort_values: impl Into<UtxoValues<T>>) {
        let len = self.stored.min_len();
        let cumulative = self.last.accumulate(
            len,
            || self.stored.collect_last(),
            |values| *values += cohort_values.into(),
        );
        self.stored.push(cumulative);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.last = Default::default();
        self.stored.stored_vecs_mut()
    }
}
