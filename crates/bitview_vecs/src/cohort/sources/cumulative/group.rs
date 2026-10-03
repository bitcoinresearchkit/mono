use std::ops::AddAssign;

use bitview_cohort::CohortGroup;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, PcoVecValue, Rw, StorageMode};

use super::super::CohortSources;
use crate::CumulativeState;

/// Per-cohort running totals of `G`, stored as cumulative series.
#[derive(Traversable)]
pub struct CumulativeCohortSources<G: CohortGroup, T: PcoVecValue, M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub stored: CohortSources<G, T, M>,
    last: M::WriteOnly<CumulativeState<G::Of<T>>>,
}

impl<G: CohortGroup, T: PcoVecValue> CumulativeCohortSources<G, T>
where
    G::Of<T>: AddAssign + Clone + Default,
{
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            stored: CohortSources::forced_import(db, name, version)?,
            last: Default::default(),
        })
    }

    #[inline(always)]
    pub fn push_block(&mut self, cohort_values: G::Of<T>) {
        let len = self.stored.min_len();
        let cumulative = self.last.accumulate(
            len,
            || self.stored.collect_last(),
            |values| *values += cohort_values,
        );
        self.stored.push(&cumulative);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.last = Default::default();
        self.stored.stored_vecs_mut()
    }
}
