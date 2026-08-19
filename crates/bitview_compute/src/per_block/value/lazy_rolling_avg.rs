use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, StoredF32, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{DeltaAvg, LazyDeltaVec, LazyVec, ReadOnlyClone, ReadableCloneableVec};

use crate::{
    AvgCentsToUsd, AvgSatsToBtc, CACHE_BUDGET, CachedWindowStartVec, DerivedResolutions,
    LazyPerBlock, LazyRollingAvgAmountFromHeight, LazyRollingAvgFromHeight, Resolutions, Windows,
};

/// Lazy rolling averages for all 4 windows, for Amount (sats + btc + cents + usd), all as f64.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyRollingAvgsAmountFromHeight(
    /// Arithmetic mean of the per-block values over the trailing window ending
    /// at the represented block; each block has equal weight. At time-period
    /// indexes, the value is taken at the period's final block.
    pub Windows<LazyRollingAvgAmountFromHeight>,
);

impl LazyRollingAvgsAmountFromHeight {
    pub fn new(
        name: &str,
        version: Version,
        cumulative_sats: &(impl ReadableCloneableVec<Height, Sats> + 'static),
        cumulative_cents: &(impl ReadableCloneableVec<Height, Cents> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &crate::IndexSources,
    ) -> Self {
        let cum_sats = cumulative_sats.read_only_boxed_clone();
        let cum_cents = cumulative_cents.read_only_boxed_clone();

        let make_slot = |suffix: &str, cached_start: &&CachedWindowStartVec| {
            let full_name = format!("{name}_{suffix}");
            let cached = cached_start.read_only_clone();
            let starts_version = cached.version();

            // Sats lazy rolling avg → f64
            let sats_avg = LazyDeltaVec::<Height, Sats, StoredF32, DeltaAvg>::new(
                &format!("{full_name}_sats"),
                version,
                cum_sats.clone(),
                starts_version,
                {
                    let cached = cached.clone();
                    move || cached.snapshot()
                },
            );
            let source = CACHE_BUDGET.wrap(sats_avg.clone());
            let sats_resolutions = Resolutions::from_height_source(
                &format!("{full_name}_sats"),
                source,
                version,
                indexes,
            );
            let sats = LazyRollingAvgFromHeight {
                height: sats_avg,
                resolutions: Box::new(sats_resolutions),
            };

            // Btc: f64 sats avg / 1e8
            let btc = LazyPerBlock {
                height: LazyVec::transformed::<AvgSatsToBtc>(
                    &full_name,
                    version,
                    sats.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<AvgSatsToBtc>(
                    &full_name,
                    version,
                    &sats.resolutions,
                )),
            };

            // Cents lazy rolling avg → f64
            let cents_avg = LazyDeltaVec::<Height, Cents, StoredF32, DeltaAvg>::new(
                &format!("{full_name}_cents"),
                version,
                cum_cents.clone(),
                starts_version,
                move || cached.snapshot(),
            );
            let source = CACHE_BUDGET.wrap(cents_avg.clone());
            let cents_resolutions = Resolutions::from_height_source(
                &format!("{full_name}_cents"),
                source,
                version,
                indexes,
            );
            let cents = LazyRollingAvgFromHeight {
                height: cents_avg,
                resolutions: Box::new(cents_resolutions),
            };

            // Usd: f64 cents avg / 100
            let usd = LazyPerBlock {
                height: LazyVec::transformed::<AvgCentsToUsd>(
                    &format!("{full_name}_usd"),
                    version,
                    cents.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<AvgCentsToUsd>(
                    &format!("{full_name}_usd"),
                    version,
                    &cents.resolutions,
                )),
            };

            LazyRollingAvgAmountFromHeight {
                btc,
                sats,
                usd,
                cents,
            }
        };

        Self(cached_starts.map_with_suffix(make_slot))
    }
}
