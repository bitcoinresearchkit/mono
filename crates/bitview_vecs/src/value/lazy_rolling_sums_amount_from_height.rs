use bitview_collections::Windows;
use bitview_transforms::Convert;
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::ReadableCloneableVec;

use crate::{IndexSources, LazyPerBlock, LazyRollingSumAmountFromHeight, LazyRollingSumFromHeight};

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
        cumulative_sats: &impl ReadableCloneableVec<Height, Sats>,
        cumulative_cents: &impl ReadableCloneableVec<Height, Cents>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self(window_starts.map_with_suffix(|suffix, window_start| {
            let full_name = format!("{name}_{suffix}");

            // Sats lazy rolling sum
            let sats = LazyRollingSumFromHeight::from_source(
                &format!("{full_name}_sats"),
                version,
                cumulative_sats,
                *window_start,
                indexes,
            );

            // Btc lazy from sats
            let btc =
                LazyPerBlock::from_resolutions::<Convert>(&full_name, version, &sats.resolutions);

            // Cents rolling sum
            let cents = LazyRollingSumFromHeight::from_source(
                &format!("{full_name}_cents"),
                version,
                cumulative_cents,
                *window_start,
                indexes,
            );

            // Usd lazy from cents
            let usd = LazyPerBlock::from_resolutions::<Convert>(
                &format!("{full_name}_usd"),
                version,
                &cents.resolutions,
            );

            LazyRollingSumAmountFromHeight {
                btc,
                sats,
                usd,
                cents,
            }
        }))
    }
}
