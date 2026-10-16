use bitview_compute::{FixedRatio, NumericValue};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{Convert, ReadableCloneableVec, VecValue};

use crate::{
    AmountType, IndexSources, LazyDeltaAmountFromHeight, LazyDeltaFromHeight,
    LazyDeltaPercentFromHeight, LazyPerBlock, shapes::Windows,
};

#[derive(Clone, Traversable)]
pub struct LazyRollingDeltasAmountFromHeight<S, C, B>
where
    S: VecValue,
    C: AmountType,
    B: FixedRatio,
{
    /// Absolute change through the represented block from the value just
    /// before a trailing window (zero for a window from genesis).
    absolute: Windows<LazyDeltaAmountFromHeight<S, C>>,
    /// Relative change through the represented block from the value just
    /// before a trailing window, divided by that value. Undefined (null) when
    /// it is zero, as for a window from genesis.
    rate: Windows<LazyDeltaPercentFromHeight<S, B>>,
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
        let version = version + super::VERSION;
        let (absolute, rate) = window_starts
            .map_with_suffix(|suffix, window_start| {
                let name = format!("{name}_{suffix}");
                let sats_name = format!("{name}_sats");
                let sats = LazyDeltaFromHeight::from_source(
                    &sats_name,
                    version,
                    source,
                    *window_start,
                    indexes,
                );
                let btc =
                    LazyPerBlock::from_resolutions::<Convert>(&name, version, &sats.resolutions);
                let absolute = LazyDeltaAmountFromHeight { btc, sats };

                let rate = LazyDeltaPercentFromHeight::from_source(
                    &name,
                    version,
                    source,
                    *window_start,
                    indexes,
                );

                (absolute, rate)
            })
            .unzip();

        Self { absolute, rate }
    }
}
