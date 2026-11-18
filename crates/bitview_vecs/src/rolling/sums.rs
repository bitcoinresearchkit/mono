use bitview_compute::NumericValue;
use bitview_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Convert, DeltaOp, DeltaSub, ReadableCloneableVec};

use crate::{
    Fiat, FiatType, IndexSources, LazyDeltaFromHeight, LazyPerBlock, PricedRollingSumFromHeight,
    PricedSumUsd, Value, priced_window, shapes::Windows,
};

/// One sum per trailing window, in the units `A` holds.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct RollingSums<A>(
    /// Total of the per-block values over the trailing window ending at the
    /// represented block. At time-period indexes, the value is taken at the
    /// period's final block.
    pub Windows<A>,
);

/// A window's sum `cum[h] - cum[window_start[h] - 1]` over a cumulative source, using zero when
/// the window starts at genesis, and its resolution views.
pub type LazyRollingSumFromHeight<T> = LazyDeltaFromHeight<T, T, DeltaSub>;
/// An amount's sum over a window, in BTC and satoshis and in USD and cents.
pub type LazyRollingSumAmountFromHeight = Value<
    LazyRollingSumFromHeight<Sats>,
    PricedRollingSumFromHeight<Cents>,
    LazyPerBlock<Bitcoin, Sats>,
    PricedSumUsd<LazyPerBlock<Dollars, Cents>>,
>;
/// A dollar value's sum over a window, in USD and cents.
pub type LazyRollingSumFiatFromHeight<C> =
    Fiat<PricedRollingSumFromHeight<C>, PricedSumUsd<LazyPerBlock<Dollars, C>>>;

/// Window sums in a value's plain units; `Op` the sum's (a dollar one's: [`crate::PricedSum`]).
pub type LazyRollingSumsFromHeight<T, Op = DeltaSub> = RollingSums<LazyDeltaFromHeight<T, T, Op>>;
pub type LazyRollingSumsAmountFromHeight = RollingSums<LazyRollingSumAmountFromHeight>;
pub type LazyRollingSumsFiatFromHeight<C> = RollingSums<LazyRollingSumFiatFromHeight<C>>;

impl<T, Op> LazyRollingSumsFromHeight<T, Op>
where
    T: NumericValue + JsonSchema,
    Op: DeltaOp<T, T>,
{
    pub fn new(
        name: &str,
        version: Version,
        cumulative: &impl ReadableCloneableVec<Height, T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self(window_starts.map_with_suffix(|suffix, start| {
            let name = format!("{name}_{suffix}");
            LazyDeltaFromHeight::from_source(&name, version, cumulative, *start, indexes)
        }))
    }
}

impl LazyRollingSumsAmountFromHeight {
    pub fn new(
        name: &str,
        version: Version,
        cumulative_sats: &impl ReadableCloneableVec<Height, Sats>,
        cumulative_cents: &impl ReadableCloneableVec<Height, Cents>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
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
                usd: PricedSumUsd(usd),
                cents,
            }
        }))
    }
}

impl<C: FiatType> LazyRollingSumsFiatFromHeight<C> {
    pub(crate) fn new(
        name: &str,
        version: Version,
        cumulative_cents: &impl ReadableCloneableVec<Height, C>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self(window_starts.map_with_suffix(|suffix, start| {
            let name = format!("{name}_{suffix}");
            let (cents, usd) = priced_window(
                &format!("{name}_cents"),
                &name,
                version,
                cumulative_cents,
                *start,
                indexes,
            );
            Fiat {
                usd: PricedSumUsd(usd),
                cents,
            }
        }))
    }
}
