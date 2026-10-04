use std::ops::AddAssign;

use bitview_cohort::{CohortGroup, CohortId};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, Rw};

use super::CumulativeCohortSources;
use crate::{CachedSeries, SatsCents};

pub type CumulativeCohortValueSources<G, M = Rw> =
    SatsCents<CumulativeCohortSources<G, Sats, M>, CumulativeCohortSources<G, Cents, M>>;

impl<G: CohortGroup> CumulativeCohortValueSources<G>
where
    G::Of<Sats>: AddAssign + Clone + Default,
    G::Of<Cents>: AddAssign + Clone + Default,
{
    pub fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            sats: CumulativeCohortSources::import(db, &format!("{name}_sats"), version)?,
            cents: CumulativeCohortSources::import(db, &format!("{name}_cents"), version)?,
        })
    }

    pub fn sources(
        &self,
        cohort_id: CohortId,
    ) -> Option<SatsCents<&CachedSeries<Height, Sats>, &CachedSeries<Height, Cents>>> {
        Some(SatsCents {
            sats: self.sats.stored.get(cohort_id)?,
            cents: self.cents.stored.get(cohort_id)?,
        })
    }

    pub fn push_block(&mut self, sats: &G::Of<Sats>, cents: &G::Of<Cents>) {
        debug_assert!(
            G::iter(cents).all(|cents| !cents.is_nan()),
            "NaN cohort value"
        );
        self.sats.push_block(sats.clone());
        self.cents.push_block(cents.clone());
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.sats
            .stored_vecs_mut()
            .chain(self.cents.stored_vecs_mut())
    }
}
