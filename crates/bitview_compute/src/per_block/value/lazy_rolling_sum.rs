use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{DeltaSub, LazyDeltaVec, LazyVec, ReadOnlyClone, ReadableCloneableVec};

use crate::{
    CachedWindowStartVec, CentsUnsignedToDollars, DerivedResolutions, LazyPerBlock,
    LazyRollingSumAmountFromHeight, LazyRollingSumFromHeight, Resolutions, SatsToBitcoin, Windows,
};

/// Lazy rolling sums for all 4 windows, for Amount (sats + btc + cents + usd).
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyRollingSumsAmountFromHeight(
    /// Total of the per-block values over the trailing window ending at the
    /// represented block. At time-period indexes, the value is taken at the
    /// period's final block.
    pub Windows<LazyRollingSumAmountFromHeight>,
);

impl LazyRollingSumsAmountFromHeight {
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

            // Sats lazy rolling sum
            let sats_sum = LazyDeltaVec::<Height, Sats, Sats, DeltaSub>::new(
                &format!("{full_name}_sats"),
                version,
                cum_sats.clone(),
                starts_version,
                {
                    let cached = cached.clone();
                    move || cached.snapshot()
                },
            );
            let sats_resolutions = Resolutions::from_height_source(
                &format!("{full_name}_sats"),
                sats_sum.clone(),
                version,
                indexes,
            );
            let sats = LazyRollingSumFromHeight {
                height: sats_sum,
                resolutions: Box::new(sats_resolutions),
            };

            // Btc lazy from sats
            let btc = LazyPerBlock {
                height: LazyVec::transformed::<SatsToBitcoin>(
                    &full_name,
                    version,
                    sats.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<SatsToBitcoin>(
                    &full_name,
                    version,
                    &sats.resolutions,
                )),
            };

            // Cents rolling sum
            let cents_sum = LazyDeltaVec::<Height, Cents, Cents, DeltaSub>::new(
                &format!("{full_name}_cents"),
                version,
                cum_cents.clone(),
                starts_version,
                move || cached.snapshot(),
            );
            let cents_resolutions = Resolutions::from_height_source(
                &format!("{full_name}_cents"),
                cents_sum.clone(),
                version,
                indexes,
            );
            let cents = LazyRollingSumFromHeight {
                height: cents_sum,
                resolutions: Box::new(cents_resolutions),
            };

            // Usd lazy from cents
            let usd = LazyPerBlock {
                height: LazyVec::transformed::<CentsUnsignedToDollars>(
                    &format!("{full_name}_usd"),
                    version,
                    cents.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<
                    CentsUnsignedToDollars,
                >(
                    &format!("{full_name}_usd"), version, &cents.resolutions
                )),
            };

            LazyRollingSumAmountFromHeight {
                btc,
                sats,
                usd,
                cents,
            }
        };

        Self(cached_starts.map_with_suffix(make_slot))
    }
}
