use std::ops::AddAssign;

use bitview_cohort::{
    AmountRange, CohortContext, CohortId, SpendableType, UTXOAggregate, UTXOValues,
};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, Database, PcoVecValue, ReadableVec, Rw, StorageMode, WritableVec,
};

use super::UTXOTypedSources;
use crate::{CachedSeries, import_cached};

#[derive(Deref, DerefMut, Traversable)]
pub struct UTXOSources<T: PcoVecValue, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub typed: UTXOTypedSources<T, M>,
    pub amount: AmountRange<CachedSeries<Height, T, M>>,
}

impl<T: PcoVecValue + AddAssign> UTXOSources<T> {
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            typed: UTXOTypedSources::forced_import(db, name, version)?,
            amount: AmountRange::try_new(|cohort_id| {
                import_cached(
                    db,
                    &CohortContext::Utxo.metric_name(cohort_id, name),
                    version + Version::TWO,
                )
            })?,
        })
    }

    pub fn get(&self, cohort_id: CohortId) -> Option<&CachedSeries<Height, T>> {
        match cohort_id {
            CohortId::Amount(id) => Some(id.select(&self.amount)),
            _ => self.typed.get(cohort_id),
        }
    }

    pub fn min_len(&self) -> usize {
        self.amount
            .iter()
            .map(AnyVec::len)
            .fold(self.typed.min_len(), usize::min)
    }

    pub fn push(&mut self, cohort_values: UTXOValues<T>) {
        self.push_with_aggregate(cohort_values, None);
    }

    pub fn push_exact(&mut self, cohort_values: UTXOValues<T>, aggregate: UTXOAggregate<T>) {
        self.push_with_aggregate(cohort_values, Some(&aggregate));
    }

    fn push_with_aggregate(
        &mut self,
        cohort_values: UTXOValues<T>,
        aggregate: Option<&UTXOAggregate<T>>,
    ) {
        for (target, &value) in self
            .amount
            .iter_mut()
            .zip(cohort_values.amount_range.iter())
        {
            target.push(value);
        }
        self.typed
            .push_with_aggregate(cohort_values.core, cohort_values.type_, aggregate);
    }

    pub fn push_partition<const ORIGIN: bool>(
        &mut self,
        values: UTXOValues<T>,
        aggregate: Option<&UTXOAggregate<T>>,
    ) {
        if !ORIGIN {
            for (target, &value) in self.amount.iter_mut().zip(values.amount_range.iter()) {
                target.push(value);
            }
        }
        self.typed
            .push_partition::<ORIGIN>(values.core, values.type_, aggregate);
    }

    pub fn collect_last(&self) -> Option<UTXOValues<T>> {
        Some(UTXOValues {
            amount_range: AmountRange::try_from_fn(|id| {
                id.select(&self.amount).collect_last().ok_or(())
            })
            .ok()?,
            type_: SpendableType::try_from_fn(|id| id.select(&self.type_).collect_last().ok_or(()))
                .ok()?,
            core: self.typed.core.collect_last()?,
        })
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.typed.collect_vecs_mut();
        vecs.extend(self.amount.iter_mut().map(|v| v as &mut dyn AnyStoredVec));
        vecs
    }
}
