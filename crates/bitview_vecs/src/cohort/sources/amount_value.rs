use bitview_cohort::{AmountRange, CohortContext, CohortId};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::AmountSources;
use crate::{CachedSeries, SatsCents};

#[derive(Deref, DerefMut, Traversable)]
pub struct AmountValueSources<S: Clone, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    series: AmountRange<S>,
    #[traversable(hidden)]
    stored: SatsCents<AmountSources<Sats, (), M>, AmountSources<Cents, (), M>>,
}

impl<S: Clone> AmountValueSources<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn import(
        db: &Database,
        storage_name: &str,
        context: CohortContext,
        metric: &str,
        version: Version,
        mut build: impl FnMut(&str, &CachedSeries<Height, Sats>, &CachedSeries<Height, Cents>) -> S,
    ) -> Result<Self> {
        let sats = AmountSources::import(
            db,
            &format!("{storage_name}_sats"),
            context,
            metric,
            version,
            |_, _| (),
        )?;
        let cents = AmountSources::import(
            db,
            &format!("{storage_name}_cents"),
            context,
            metric,
            version,
            |_, _| (),
        )?;
        let series = AmountRange::from_fn(|id| {
            let name = context.metric_name(CohortId::Amount(id), metric);
            build(&name, id.select(&sats.stored), id.select(&cents.stored))
        });
        Ok(Self {
            series,
            stored: SatsCents { sats, cents },
        })
    }

    pub fn push_cumulative(&mut self, sats: &AmountRange<Sats>, cents: &AmountRange<Cents>) {
        debug_assert!(
            cents.iter().all(|cents| !cents.is_nan()),
            "NaN amount value"
        );
        self.stored.sats.push_cumulative(sats);
        self.stored.cents.push_cumulative(cents);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .sats
            .stored_vecs_mut()
            .chain(self.stored.cents.stored_vecs_mut())
    }
}
