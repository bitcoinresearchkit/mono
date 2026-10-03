use bitview_cohort::{AmountRange, CohortContext, CohortId};
use bitview_transforms::{StoredU64ToCents, StoredU64ToSats};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, StoredU64, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{AnyStoredVec, Database, LazyVec, Rw, StorageMode};

use super::AmountSources;
use crate::SatsCents;

#[derive(Deref, DerefMut, Traversable)]
pub struct AmountValueSources<S: Clone, M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    series: AmountRange<S>,
    #[traversable(hidden)]
    stored: SatsCents<AmountSources<StoredU64, (), M>>,
}

impl<S: Clone> AmountValueSources<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn import(
        db: &Database,
        storage_name: &str,
        context: CohortContext,
        metric: &str,
        version: Version,
        mut build: impl FnMut(
            &str,
            LazyVec<Height, Sats, Height, StoredU64>,
            LazyVec<Height, Cents, Height, StoredU64>,
        ) -> S,
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
            let sats = LazyVec::transformed::<StoredU64ToSats>(
                &format!("{name}_cumulative_sats"),
                version,
                id.select(&sats.stored).read_only_boxed_clone(),
            );
            let cents = LazyVec::transformed::<StoredU64ToCents>(
                &format!("{name}_cumulative_cents"),
                version,
                id.select(&cents.stored).read_only_boxed_clone(),
            );
            build(&name, sats, cents)
        });
        Ok(Self {
            series,
            stored: SatsCents { sats, cents },
        })
    }

    pub fn push_cumulative(&mut self, sats: &AmountRange<Sats>, cents: &AmountRange<Cents>) {
        self.stored
            .sats
            .push_cumulative(&AmountRange::from_fn(|id| {
                StoredU64::from(u64::from(*id.select(sats)))
            }));
        self.stored
            .cents
            .push_cumulative(&AmountRange::from_fn(|id| {
                StoredU64::from(u64::from(*id.select(cents)))
            }));
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .sats
            .stored_vecs_mut()
            .chain(self.stored.cents.stored_vecs_mut())
    }
}
use vecdb::ReadableCloneableVec;
