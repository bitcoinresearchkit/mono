use bitview_cohort::{
    AgeRange, ByEpoch, Class, CohortContext, CohortId, CreationCohorts, UTXOCoreValues,
};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{
    AnyStoredVec, AnyVec, Database, PcoVecValue, ReadableVec, Rw, StorageMode, WritableVec,
};

use crate::{CachedSeries, import_cached};

/// Independently stored cohort sources, composed from the domain's named groups.
#[derive(Traversable)]
pub struct CreationSources<T: PcoVecValue, M: StorageMode = Rw> {
    #[traversable(flatten)]
    cohorts: CreationCohorts<CachedSeries<Height, T, M>>,
}

impl<T: PcoVecValue> CreationSources<T> {
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            cohorts: CreationCohorts::try_new(|id| {
                import_cached(
                    db,
                    &CohortContext::Utxo.metric_name(id, name),
                    version + Version::TWO,
                )
            })?,
        })
    }

    pub fn get(&self, cohort_id: CohortId) -> Option<&CachedSeries<Height, T>> {
        self.cohorts.get(cohort_id)
    }

    pub(crate) fn min_len(&self) -> usize {
        self.cohorts.iter().map(AnyVec::len).min().unwrap_or(0)
    }

    pub fn push(&mut self, values: impl Into<UTXOCoreValues<T>>) {
        let values = values.into();
        for (target, &value) in self.cohorts.age.iter_mut().zip(values.age_range.iter()) {
            target.push(value);
        }
        for (target, &value) in self.cohorts.epoch.iter_mut().zip(values.epoch.iter()) {
            target.push(value);
        }
        for (target, &value) in self.cohorts.class.iter_mut().zip(values.class.iter()) {
            target.push(value);
        }
    }

    pub(crate) fn collect_last(&self) -> Option<UTXOCoreValues<T>> {
        Some(UTXOCoreValues {
            age_range: AgeRange::try_from_fn(|id| {
                id.select(&self.cohorts.age).collect_last().ok_or(())
            })
            .ok()?,
            epoch: ByEpoch::try_from_fn(|id| {
                id.select(&self.cohorts.epoch).collect_last().ok_or(())
            })
            .ok()?,
            class: Class::try_from_fn(|id| id.select(&self.cohorts.class).collect_last().ok_or(()))
                .ok()?,
        })
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.cohorts
            .iter_mut()
            .map(|v| v as &mut dyn AnyStoredVec)
            .collect()
    }
}
