use std::ops::AddAssign;

use bitview_cohort::{AmountRange, CohortContext};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, AnyVec, Database, PcoVecValue, ReadableCloneableVec, ReadableVec, Rw,
    StorageMode, WritableVec,
};

use crate::{CachedSeries, CumulativeState, import_cached};

/// Named amount cohorts and views derived from their individual sources.
#[derive(Deref, DerefMut, Traversable)]
pub struct AmountSources<T: PcoVecValue, S: Clone, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: AmountRange<S>,
    #[traversable(hidden)]
    pub(crate) stored: AmountRange<CachedSeries<Height, T, M>>,
    last: M::WriteOnly<CumulativeState<AmountRange<T>>>,
}

impl<T: PcoVecValue + AddAssign, S: Clone> AmountSources<T, S> {
    #[allow(clippy::too_many_arguments)]
    pub fn forced_import(
        db: &Database,
        storage_name: &str,
        context: CohortContext,
        metric: &str,
        version: Version,
        mut build: impl FnMut(&str, &dyn ReadableCloneableVec<Height, T>) -> S,
    ) -> Result<Self> {
        let stored = AmountRange::try_new(|cohort_id| {
            let cohort = cohort_id.name();
            import_cached(
                db,
                &format!("{storage_name}_{cohort}"),
                version + Version::ONE,
            )
        })?;
        let series = AmountRange::from_fn(|id| {
            build(
                &context.metric_name(id.cohort(), metric),
                id.select(&stored),
            )
        });
        Ok(Self {
            series,
            stored,
            last: Default::default(),
        })
    }

    pub fn push(&mut self, values: AmountRange<T>) {
        for (target, &value) in self.stored.iter_mut().zip(values.iter()) {
            target.push(value);
        }
    }

    pub fn push_cumulative(&mut self, delta: &AmountRange<T>)
    where
        T: Default,
    {
        let len = self.min_len();
        let values = self.last.accumulate(
            len,
            || AmountRange::try_from_fn(|id| id.select(&self.stored).collect_last().ok_or(())).ok(),
            |values| {
                for (value, &delta) in values.iter_mut().zip(delta.iter()) {
                    *value += delta;
                }
            },
        );
        self.push(values);
    }

    fn min_len(&self) -> usize {
        self.stored.iter().map(AnyVec::len).min().unwrap_or(0)
    }
    pub fn checkpoint(&self, height: Height) -> Option<AmountRange<T>> {
        AmountRange::try_from_fn(|id| id.select(&self.stored).collect_one(height).ok_or(())).ok()
    }

    pub fn reset(&mut self) -> Result<()> {
        self.last = Default::default();
        for target in self.stored.iter_mut() {
            target.reset()?;
        }
        Ok(())
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.last = Default::default();
        self.stored.iter_mut().map(|v| v as &mut dyn AnyStoredVec)
    }
}
