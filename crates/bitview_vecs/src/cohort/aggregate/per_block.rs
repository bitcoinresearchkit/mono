use bitview_cohort::UTXOAggregate;
use bitview_traversable::Traversable;
use brk_types::Height;
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, AnyVec, PcoVecValue, Rw, StorageMode, WritableVec};

use crate::CachedSeries;

/// Aggregate cohort views backed by one stored height source per cohort.
#[derive(Deref, DerefMut, Traversable)]
pub struct AggregatePerBlock<V: Clone, T: PcoVecValue, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub(crate) series: UTXOAggregate<V>,
    #[traversable(hidden)]
    pub stored: UTXOAggregate<CachedSeries<Height, T, M>>,
}

impl<V: Clone, T: PcoVecValue> AggregatePerBlock<V, T> {
    pub fn push(&mut self, values: UTXOAggregate<T>) {
        for (target, &value) in self.stored.iter_mut().zip(values.iter()) {
            target.push(value);
        }
    }

    pub fn len(&self) -> usize {
        self.stored.iter().map(AnyVec::len).min().unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.stored
            .iter_mut()
            .map(|v| v as &mut dyn AnyStoredVec)
            .collect()
    }
}
