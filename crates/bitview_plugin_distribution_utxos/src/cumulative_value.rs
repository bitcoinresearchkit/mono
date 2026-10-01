use crate::values::UtxoValues;
use bitview_cohort::CohortId;
use bitview_transforms::{StoredU64ToCents, StoredU64ToSats};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, StoredU64, Version};
use vecdb::{AnyStoredVec, Database, LazyVec, Rw};

use crate::cumulative::CumulativeUtxoSources;
use bitview_traversable::Traversable;
use bitview_vecs::SatsCents;
use vecdb::StorageMode;

#[derive(Traversable)]
pub struct CumulativeSizeValueSources<M: StorageMode = Rw> {
    pub sats: CumulativeUtxoSources<StoredU64, M>,
    pub cents: CumulativeUtxoSources<StoredU64, M>,
}

impl CumulativeSizeValueSources {
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            sats: CumulativeUtxoSources::forced_import(db, &format!("{name}_sats"), version)?,
            cents: CumulativeUtxoSources::forced_import(db, &format!("{name}_cents"), version)?,
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
        sats: impl Into<UtxoValues<Sats>>,
        cents: impl Into<UtxoValues<Cents>>,
    ) {
        self.sats
            .push_block(sats.into().map(|value| StoredU64::from(u64::from(*value))));
        self.cents
            .push_block(cents.into().map(|value| StoredU64::from(u64::from(*value))));
    }

    pub fn min_len(&self) -> usize {
        self.sats.min_len().min(self.cents.min_len())
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.sats
            .stored_vecs_mut()
            .chain(self.cents.stored_vecs_mut())
    }
}
use vecdb::ReadableCloneableVec;
