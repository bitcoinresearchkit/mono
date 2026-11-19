use bitview_compute::{NumericValue, Quantity};
use bitview_primitives::{CentsFract, SatsFract};
use bitview_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Convert, DeltaAvg, ReadableCloneableVec};

use crate::{
    IndexSources, LazyDeltaFromHeight, LazyPerBlock, PricedAvgUsd, PricedRollingAvgFromHeight,
    Value, priced_window, shapes::Windows,
};

/// Added to a window mean's version: the window's sum is subtracted exactly before converting.
const VERSION: Version = Version::ONE;

/// One mean per trailing window, in the units `A` holds.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct RollingAvgs<A>(
    /// Arithmetic mean of the per-block values over the trailing window ending
    /// at the represented block; each block has equal weight. At time-period
    /// indexes, the value is taken at the period's final block.
    pub Windows<A>,
);

/// A window's mean `(cum[h] - cum[start - 1]) / (h - start + 1)` over a cumulative source, using
/// zero before genesis, in the quantity's mean type, and its resolution views.
pub type LazyRollingAvgFromHeight<T> = LazyDeltaFromHeight<T, <T as Quantity>::Fract, DeltaAvg>;
/// An amount's mean over a window, as mean sats and cents with BTC and USD views.
pub type LazyRollingAvgAmountFromHeight = Value<
    LazyRollingAvgFromHeight<Sats>,
    PricedRollingAvgFromHeight<Cents>,
    LazyPerBlock<Bitcoin, SatsFract>,
    PricedAvgUsd<LazyPerBlock<Dollars, CentsFract>>,
>;

pub type LazyRollingAvgsFromHeight<T> = RollingAvgs<LazyRollingAvgFromHeight<T>>;
pub type LazyRollingAvgsAmountFromHeight = RollingAvgs<LazyRollingAvgAmountFromHeight>;

impl<T> LazyRollingAvgsFromHeight<T>
where
    T: NumericValue + JsonSchema + Quantity,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        cumulative: &impl ReadableCloneableVec<Height, T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        let version = version + VERSION;
        Self(window_starts.map_with_suffix(|suffix, start| {
            let name = format!("{name}_{suffix}");
            LazyDeltaFromHeight::from_source(&name, version, cumulative, *start, indexes)
        }))
    }
}

impl LazyRollingAvgsAmountFromHeight {
    pub(crate) fn new(
        name: &str,
        version: Version,
        cumulative_sats: &impl ReadableCloneableVec<Height, Sats>,
        cumulative_cents: &impl ReadableCloneableVec<Height, Cents>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        let version = version + VERSION;
        Self(window_starts.map_with_suffix(|suffix, start| {
            let name = format!("{name}_{suffix}");
            let sats = LazyDeltaFromHeight::from_source(
                &format!("{name}_sats"),
                version,
                cumulative_sats,
                *start,
                indexes,
            );
            let btc = LazyPerBlock::from_resolutions::<Convert>(&name, version, &sats.resolutions);
            let (cents, usd) = priced_window(
                &format!("{name}_cents"),
                &format!("{name}_usd"),
                version,
                cumulative_cents,
                *start,
                indexes,
            );
            Value {
                btc,
                sats,
                usd: PricedAvgUsd(usd),
                cents,
            }
        }))
    }
}
