use crate::{groups::UtxoGroups, values::UtxoValues};
use bitview_cohort::{AmountRange, CohortContext, CohortId, SpendableType};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, import_cached};
use brk_error::Result;
use brk_types::{Height, Version};
use std::ops::AddAssign;
use vecdb::{
    AnyStoredVec, AnyVec, Database, PcoVecValue, ReadableVec, Rw, StorageMode, WritableVec,
};
#[derive(Traversable)]
pub struct UtxoSources<T: PcoVecValue, M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UtxoGroups<CachedSeries<Height, T, M>>,
}
impl<T: PcoVecValue + AddAssign> UtxoSources<T> {
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            cohorts: UtxoGroups::try_new(|id| {
                import_cached(
                    db,
                    &CohortContext::Utxo.metric_name(id, name),
                    version + Version::TWO,
                )
            })?,
        })
    }
    pub fn get(&self, id: CohortId) -> Option<&CachedSeries<Height, T>> {
        self.cohorts.get(id)
    }
    pub fn min_len(&self) -> usize {
        self.cohorts
            .iter()
            .map(AnyVec::len)
            .min()
            .unwrap_or_default()
    }
    pub fn push(&mut self, values: UtxoValues<T>) {
        for (out, value) in self
            .cohorts
            .utxo_amount
            .iter_mut()
            .zip(values.amount_range.iter())
        {
            out.push(*value);
        }
        for (out, value) in self.cohorts.type_.iter_mut().zip(values.type_.iter()) {
            out.push(*value);
        }
    }
    pub fn collect_last(&self) -> Option<UtxoValues<T>> {
        Some(UtxoValues {
            amount_range: AmountRange::try_from_fn(|id| {
                id.select(&self.cohorts.utxo_amount)
                    .collect_last()
                    .ok_or(())
            })
            .ok()?,
            type_: SpendableType::try_from_fn(|id| {
                id.select(&self.cohorts.type_).collect_last().ok_or(())
            })
            .ok()?,
        })
    }
    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.cohorts.iter_mut().map(|v| v as &mut dyn AnyStoredVec)
    }
}
