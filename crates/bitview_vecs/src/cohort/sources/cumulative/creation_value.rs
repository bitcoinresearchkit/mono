use bitview_cohort::{CohortId, UTXOCoreValues};
use bitview_transforms::{StoredU64ToCents, StoredU64ToSats};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, StoredU64, Version};
use vecdb::{AnyStoredVec, Database, LazyVec, Rw};

use super::CumulativeCreationSources;
use crate::SatsCents;

pub type CumulativeCreationValueSources<M = Rw> =
    SatsCents<CumulativeCreationSources<StoredU64, M>>;

impl CumulativeCreationValueSources {
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            sats: CumulativeCreationSources::forced_import(db, &format!("{name}_sats"), version)?,
            cents: CumulativeCreationSources::forced_import(db, &format!("{name}_cents"), version)?,
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

    pub fn push_block(
        &mut self,
        sats: impl Into<UTXOCoreValues<Sats>>,
        cents: impl Into<UTXOCoreValues<Cents>>,
    ) {
        self.sats
            .push_block(sats.into().map(|value| StoredU64::from(u64::from(*value))));
        self.cents
            .push_block(cents.into().map(|value| StoredU64::from(u64::from(*value))));
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.sats.collect_vecs_mut();
        vecs.extend(self.cents.collect_vecs_mut());
        vecs
    }
}
use vecdb::ReadableCloneableVec;
