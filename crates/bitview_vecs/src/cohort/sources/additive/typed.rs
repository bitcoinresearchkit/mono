use std::ops::AddAssign;

use bitview_cohort::{
    CohortContext, CohortId, SpendableType, SpendableTypeId, UTXOAggregate, UTXOCoreValues,
};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, AnyVec, Database, PcoVecValue, Rw, StorageMode, WritableVec};

use super::UTXOCoreSources;
use crate::{CachedSeries, import_cached};

#[derive(Deref, DerefMut, Traversable)]
pub struct UTXOTypedSources<T: PcoVecValue, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub core: UTXOCoreSources<T, M>,
    #[traversable(rename = "type")]
    pub type_: SpendableType<CachedSeries<Height, T, M>>,
}

impl<T: PcoVecValue + AddAssign> UTXOTypedSources<T> {
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            core: UTXOCoreSources::forced_import(db, name, version)?,
            type_: SpendableType::try_new(|cohort_id| {
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
            CohortId::Type(output_type) => {
                SpendableTypeId::from_output_type(output_type).map(|id| id.select(&self.type_))
            }
            _ => self.core.get(cohort_id),
        }
    }

    pub fn min_len(&self) -> usize {
        self.type_
            .iter()
            .map(AnyVec::len)
            .fold(self.core.min_len(), usize::min)
    }

    pub fn push(&mut self, core: UTXOCoreValues<T>, type_: SpendableType<T>) {
        self.push_with_aggregate(core, type_, None);
    }

    pub(crate) fn push_with_aggregate(
        &mut self,
        core: UTXOCoreValues<T>,
        type_: SpendableType<T>,
        aggregate: Option<&UTXOAggregate<T>>,
    ) {
        self.core.push_with_aggregate(core, aggregate);
        for (target, &value) in self.type_.iter_mut().zip(type_.iter()) {
            target.push(value);
        }
    }

    pub fn push_partition<const ORIGIN: bool>(
        &mut self,
        core: UTXOCoreValues<T>,
        type_: SpendableType<T>,
        aggregate: Option<&UTXOAggregate<T>>,
    ) {
        if ORIGIN {
            self.core.push_with_aggregate(core, aggregate);
        } else {
            for (target, &value) in self.type_.iter_mut().zip(type_.iter()) {
                target.push(value);
            }
        }
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.core.collect_vecs_mut();
        vecs.extend(self.type_.iter_mut().map(|v| v as &mut dyn AnyStoredVec));
        vecs
    }
}
