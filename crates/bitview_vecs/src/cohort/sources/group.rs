use bitview_cohort::{CohortContext, CohortGroup, CohortId};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, Database, PcoVecValue, ReadableVec, Rw, StorageMode, WritableVec,
};

use crate::{CachedSeries, import_cached};

/// One independently stored series per cohort of `G`.
#[derive(Traversable)]
pub struct CohortSources<G: CohortGroup, T: PcoVecValue, M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: G::Of<CachedSeries<Height, T, M>>,
}

impl<G: CohortGroup, T: PcoVecValue> CohortSources<G, T> {
    pub fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            cohorts: G::try_new(|id| {
                import_cached(
                    db,
                    &CohortContext::Utxo.metric_name(id, name),
                    version + Version::TWO,
                )
            })?,
        })
    }

    pub fn get(&self, id: CohortId) -> Option<&CachedSeries<Height, T>> {
        G::get(&self.cohorts, id)
    }

    pub(crate) fn min_len(&self) -> usize {
        G::iter(&self.cohorts).map(AnyVec::len).min().unwrap_or(0)
    }

    pub fn push(&mut self, values: &G::Of<T>) {
        for (target, value) in G::iter_mut(&mut self.cohorts).zip(G::iter(values)) {
            target.push(*value);
        }
    }

    pub(crate) fn collect_last(&self) -> Option<G::Of<T>> {
        G::try_new(|id| {
            G::get(&self.cohorts, id)
                .and_then(|vec| vec.collect_last())
                .ok_or(())
        })
        .ok()
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        G::iter_mut(&mut self.cohorts).map(|v| v as &mut dyn AnyStoredVec)
    }
}
