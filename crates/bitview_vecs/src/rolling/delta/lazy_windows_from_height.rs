use bitview_compute::{FixedRatio, NumericValue};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Convert, DeltaChange, ReadableCloneableVec, VecValue};

use crate::{
    AmountType, FiatType, IndexSources, LazyDeltaAmountFromHeight, LazyDeltaFiatFromHeight,
    LazyDeltaFromHeight, LazyDeltaPercentFromHeight, LazyPerBlock, PRICED_VERSION,
    PricedDeltaFromHeight, PricedDeltaUsd, shapes::Windows,
};

/// Trailing-window deltas of a value in its plain units.
pub type LazyRollingDeltasFromHeight<S, C, B> =
    LazyRollingDeltas<LazyDeltaFromHeight<S, C, DeltaChange>, S, B>;
/// Trailing-window deltas in BTC and satoshis.
pub type LazyRollingDeltasAmountFromHeight<S, C, B> =
    LazyRollingDeltas<LazyDeltaAmountFromHeight<S, C>, S, B>;
/// Trailing-window deltas in USD and cents.
pub type LazyRollingDeltasFiatFromHeight<S, C, B> =
    LazyRollingDeltas<LazyDeltaFiatFromHeight<S, C>, S, B>;

#[derive(Clone, Traversable)]
#[traversable(client = "Delta")]
pub struct LazyRollingDeltas<A, S, B>
where
    S: VecValue,
    B: FixedRatio,
{
    /// Absolute change through the represented block from the value just
    /// before a trailing window (zero for a window from genesis).
    absolute: Windows<A>,
    /// Relative change through the represented block from the value just
    /// before a trailing window, divided by that value. Undefined (null) when
    /// it is zero, as for a window from genesis.
    percent: Windows<LazyDeltaPercentFromHeight<S, B>>,
}

impl<A, S, B> LazyRollingDeltas<A, S, B>
where
    S: NumericValue,
    B: FixedRatio + From<f64>,
{
    fn build<W: ReadableCloneableVec<Height, Height>>(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        window_starts: &Windows<&W>,
        indexes: &IndexSources,
        mut absolute: impl FnMut(&str, Version, &W) -> A,
    ) -> Self {
        let version = version + super::VERSION;
        let (absolute, percent) = window_starts
            .map_with_suffix(|suffix, window_start| {
                let name = format!("{name}_{suffix}");
                let percent = LazyDeltaPercentFromHeight::from_source(
                    &name,
                    version,
                    source,
                    *window_start,
                    indexes,
                );
                (absolute(&name, version, *window_start), percent)
            })
            .unzip();
        Self { absolute, percent }
    }
}

impl<S, C, B> LazyRollingDeltasFromHeight<S, C, B>
where
    S: NumericValue,
    C: NumericValue + JsonSchema + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self::build(
            name,
            version,
            source,
            window_starts,
            indexes,
            |name, version, start| {
                LazyDeltaFromHeight::from_source(name, version, source, start, indexes)
            },
        )
    }
}

impl<S, C, B> LazyRollingDeltasAmountFromHeight<S, C, B>
where
    S: NumericValue,
    C: AmountType + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self::build(
            name,
            version,
            source,
            window_starts,
            indexes,
            |name, version, start| {
                let sats_name = format!("{name}_sats");
                let sats =
                    LazyDeltaFromHeight::from_source(&sats_name, version, source, start, indexes);
                let btc =
                    LazyPerBlock::from_resolutions::<Convert>(name, version, &sats.resolutions);
                LazyDeltaAmountFromHeight { btc, sats }
            },
        )
    }
}

impl<S, C, B> LazyRollingDeltasFiatFromHeight<S, C, B>
where
    S: NumericValue,
    C: FiatType + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self::build(
            name,
            version,
            source,
            window_starts,
            indexes,
            |name, version, start| {
                let (cents_name, version) = (format!("{name}_cents"), version + PRICED_VERSION);
                let cents = PricedDeltaFromHeight::from_source(
                    &cents_name,
                    version,
                    source,
                    start,
                    indexes,
                );
                let usd =
                    LazyPerBlock::from_resolutions::<Convert>(name, version, &cents.resolutions);
                LazyDeltaFiatFromHeight {
                    usd: PricedDeltaUsd(usd),
                    cents,
                }
            },
        )
    }
}
