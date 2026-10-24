use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Convert, ReadableCloneableVec};

use crate::{
    IndexSources, LazyPerBlock, LazyRollingAvgAmountFromHeight, LazyRollingAvgFromHeight,
    PRICED_VERSION, PricedAvgUsd, PricedRollingAvgFromHeight, shapes::Windows,
};

/// Lazy rolling averages for all 4 windows, as mean sats/cents with BTC/USD views.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyRollingAvgsAmountFromHeight(
    /// Arithmetic mean of the per-block values over the trailing window ending
    /// at the represented block; each block has equal weight. At time-period
    /// indexes, the value is taken at the period's final block.
    pub Windows<LazyRollingAvgAmountFromHeight>,
);

impl LazyRollingAvgsAmountFromHeight {
    pub(crate) fn new(
        name: &str,
        version: Version,
        cumulative_sats: &impl ReadableCloneableVec<Height, Sats>,
        cumulative_cents: &impl ReadableCloneableVec<Height, Cents>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        // The window's sum is subtracted exactly before converting.
        let version = version + Version::ONE;
        Self(window_starts.map_with_suffix(|suffix, window_start| {
            let full_name = format!("{name}_{suffix}");

            let sats = LazyRollingAvgFromHeight::from_source(
                &format!("{full_name}_sats"),
                version,
                cumulative_sats,
                *window_start,
                indexes,
            );

            let btc =
                LazyPerBlock::from_resolutions::<Convert>(&full_name, version, &sats.resolutions);

            let cents = PricedRollingAvgFromHeight::from_source(
                &format!("{full_name}_cents"),
                version + PRICED_VERSION,
                cumulative_cents,
                *window_start,
                indexes,
            );

            let usd = LazyPerBlock::from_resolutions::<Convert>(
                &format!("{full_name}_usd"),
                version + PRICED_VERSION,
                &cents.resolutions,
            );

            LazyRollingAvgAmountFromHeight {
                btc,
                sats,
                usd: PricedAvgUsd(usd),
                cents,
            }
        }))
    }
}
