use bitview_cohort::{AgeRange, CohortContext};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};

use vecdb::{
    AnyStoredVec, BytesVec, BytesVecValue, Database, ImportableVec, Rw, StorageMode, WritableVec,
};

/// Exact raw inputs for disjoint age bands.
#[derive(Traversable)]
pub struct DisjointAgeSources<T: BytesVecValue, M: StorageMode = Rw> {
    pub age: AgeRange<M::Stored<BytesVec<Height, T>>>,
}

impl<T: BytesVecValue + Copy> DisjointAgeSources<T> {
    pub fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            age: AgeRange::try_new(|id| {
                BytesVec::import(
                    db,
                    &CohortContext::Utxo.metric_name(id, name),
                    version + Version::ONE,
                )
            })?,
        })
    }

    pub fn push_age(&mut self, values: &AgeRange<T>) {
        for (target, &value) in self.age.iter_mut().zip(values.iter()) {
            target.push(value);
        }
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.age
            .iter_mut()
            .map(|v| v as &mut dyn AnyStoredVec)
            .collect()
    }
}
