use std::ops::AddAssign;

use bitview_cohort::{
    AgeRange, ByEpoch, Class, CohortContext, CohortId, UTXOAggregate, UTXOCoreValues,
    UTXOGroupsWithoutAmountOrType,
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
pub struct UTXOCoreSources<T: PcoVecValue, M: StorageMode = Rw> {
    #[traversable(flatten)]
    cohorts: UTXOGroupsWithoutAmountOrType<Option<CachedSeries<Height, T, M>>>,
}

impl<T: PcoVecValue + AddAssign> UTXOCoreSources<T> {
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Self::import(db, name, version, true)
    }

    /// Stores cohort sources while the caller supplies the canonical global total.
    pub fn forced_import_without_all(db: &Database, name: &str, version: Version) -> Result<Self> {
        Self::import(db, name, version, false)
    }

    fn import(db: &Database, name: &str, version: Version, include_all: bool) -> Result<Self> {
        Ok(Self {
            cohorts: UTXOGroupsWithoutAmountOrType::try_new(|cohort_id| {
                if !include_all && cohort_id == CohortId::All {
                    return Ok(None);
                }
                import_cached(
                    db,
                    &CohortContext::Utxo.metric_name(cohort_id, name),
                    version + Version::TWO,
                )
                .map(Some)
            })?,
        })
    }

    pub fn get(&self, cohort_id: CohortId) -> Option<&CachedSeries<Height, T>> {
        self.cohorts.get(cohort_id)?.as_ref()
    }

    pub fn min_len(&self) -> usize {
        self.cohorts
            .iter()
            .flatten()
            .map(AnyVec::len)
            .min()
            .unwrap_or(0)
    }

    pub fn push(&mut self, cohort_values: impl Into<UTXOCoreValues<T>>) {
        self.push_with_aggregate(cohort_values.into(), None);
    }

    pub fn push_exact(&mut self, values: UTXOCoreValues<T>, aggregate: UTXOAggregate<T>) {
        self.push_with_aggregate(values, Some(&aggregate));
    }

    pub(crate) fn push_with_aggregate(
        &mut self,
        cohort_values: UTXOCoreValues<T>,
        aggregate: Option<&UTXOAggregate<T>>,
    ) {
        let values = self.cohorts.map_with_id(|cohort_id, source| {
            source.as_ref().map(|_| {
                aggregate
                    .and_then(|values| values.get(cohort_id))
                    .copied()
                    .unwrap_or_else(|| cohort_values.value(cohort_id).expect("core cohort"))
            })
        });
        for (target, &value) in self.cohorts.iter_mut().zip(values.iter()) {
            if let (Some(target), Some(value)) = (target, value) {
                target.push(value);
            }
        }
    }

    pub fn collect_last(&self) -> Option<UTXOCoreValues<T>> {
        Some(UTXOCoreValues {
            age_range: AgeRange::try_from_fn(|id| {
                id.select(&self.cohorts.age)
                    .as_ref()
                    .and_then(ReadableVec::collect_last)
                    .ok_or(())
            })
            .ok()?,
            epoch: ByEpoch::try_from_fn(|id| {
                id.select(&self.cohorts.epoch)
                    .as_ref()
                    .and_then(ReadableVec::collect_last)
                    .ok_or(())
            })
            .ok()?,
            class: Class::try_from_fn(|id| {
                id.select(&self.cohorts.class)
                    .as_ref()
                    .and_then(ReadableVec::collect_last)
                    .ok_or(())
            })
            .ok()?,
        })
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.cohorts
            .iter_mut()
            .flatten()
            .map(|v| v as &mut dyn AnyStoredVec)
            .collect()
    }
}
