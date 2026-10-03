use bitview_cohort::{CohortGroup, CohortId};
use bitview_transforms::{StoredU64ToCents, StoredU64ToSats};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, StoredU64, Version};
use vecdb::{AnyStoredVec, Database, LazyVec, ReadableCloneableVec, Rw};

use super::CumulativeCohortSources;
use crate::SatsCents;

pub type CumulativeCohortValueSources<G, M = Rw> =
    SatsCents<CumulativeCohortSources<G, StoredU64, M>>;

impl<G: CohortGroup> CumulativeCohortValueSources<G>
where
    G::Of<StoredU64>: std::ops::AddAssign + Clone + Default,
{
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            sats: CumulativeCohortSources::forced_import(db, &format!("{name}_sats"), version)?,
            cents: CumulativeCohortSources::forced_import(db, &format!("{name}_cents"), version)?,
        })
    }

    pub fn sources(
        &self,
        cohort_id: CohortId,
        name: &str,
        version: Version,
    ) -> Option<
        SatsCents<
            LazyVec<Height, Sats, Height, StoredU64>,
            LazyVec<Height, Cents, Height, StoredU64>,
        >,
    > {
        Some(SatsCents {
            sats: LazyVec::transformed::<StoredU64ToSats>(
                &format!("{name}_cumulative_sats"),
                version,
                self.sats.stored.get(cohort_id)?.read_only_boxed_clone(),
            ),
            cents: LazyVec::transformed::<StoredU64ToCents>(
                &format!("{name}_cumulative_cents"),
                version,
                self.cents.stored.get(cohort_id)?.read_only_boxed_clone(),
            ),
        })
    }

    pub fn push_block(&mut self, sats: &G::Of<Sats>, cents: &G::Of<Cents>) {
        self.sats
            .push_block(G::map(sats, |value| StoredU64::from(u64::from(*value))));
        self.cents
            .push_block(G::map(cents, |value| StoredU64::from(u64::from(*value))));
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.sats
            .stored_vecs_mut()
            .chain(self.cents.stored_vecs_mut())
    }
}
