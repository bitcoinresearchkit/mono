use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::ReadableCloneableVec;

use crate::{
    IndexSources, LazyPerBlock, LazyRollingAvgAmountFromHeight, LazyRollingAvgFromHeight,
    ops::{AvgCentsToUsd, AvgSatsToBtc},
    shapes::Windows,
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

            // Sats rolling average, stored as a float.
            let sats = LazyRollingAvgFromHeight::from_source(
                &format!("{full_name}_sats"),
                version,
                cumulative_sats,
                *window_start,
                indexes,
            );

            // BTC from the sats average.
            let btc = LazyPerBlock::from_resolutions::<AvgSatsToBtc>(
                &full_name,
                version,
                &sats.resolutions,
            );

            // Cents rolling average, stored as a float.
            let cents = LazyRollingAvgFromHeight::from_source(
                &format!("{full_name}_cents"),
                version,
                cumulative_cents,
                *window_start,
                indexes,
            );

            // USD from the cents average.
            let usd = LazyPerBlock::from_resolutions::<AvgCentsToUsd>(
                &format!("{full_name}_usd"),
                version,
                &cents.resolutions,
            );

            LazyRollingAvgAmountFromHeight {
                btc,
                sats,
                usd,
                cents,
            }
        }))
    }
}
