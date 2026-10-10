use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CumulativeSource, LazyValuePerBlockCumulativeRolling, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

/// One cohort's flow of bitcoin, stored as running totals `{name}_cumulative_{sats,cents}`.
#[derive(Traversable)]
pub struct CumulativeValue<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub value: LazyValuePerBlockCumulativeRolling,
    #[traversable(hidden)]
    pub sats: CumulativeSource<Sats, M>,
    #[traversable(hidden)]
    pub cents: CumulativeSource<Cents, M>,
}

impl CumulativeValue {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = |unit: &str| format!("{name}_cumulative_{unit}");
        let sats = CumulativeSource::import(db, &stored("sats"), version + Version::TWO)?;
        let cents = CumulativeSource::import(db, &stored("cents"), version + Version::TWO)?;
        let value = LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
            name,
            version,
            sats.cumulative_source(),
            cents.cumulative_source(),
            mappings,
            window_starts,
        );
        Ok(Self { value, sats, cents })
    }

    #[inline(always)]
    pub fn push_block(&mut self, sats: Sats, cents: Cents) {
        debug_assert!(!cents.is_nan(), "NaN cohort value");
        self.sats.push_block(sats);
        self.cents.push_block(cents);
    }

    pub fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 2] {
        [self.sats.stored_mut(), self.cents.stored_mut()]
    }
}
