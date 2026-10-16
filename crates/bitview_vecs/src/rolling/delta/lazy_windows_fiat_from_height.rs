use bitview_compute::{FixedRatio, NumericValue};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{Convert, ReadableCloneableVec, VecValue};

use crate::{
    FiatType, IndexSources, LazyDeltaFiatFromHeight, LazyDeltaFromHeight,
    LazyDeltaPercentFromHeight, LazyPerBlock, shapes::Windows,
};

#[derive(Clone, Traversable)]
pub struct LazyRollingDeltasFiatFromHeight<S, C, B>
where
    S: VecValue,
    C: FiatType,
    B: FixedRatio,
{
    /// Absolute change through the represented block from the value just
    /// before a trailing window (zero for a window from genesis).
    absolute: Windows<LazyDeltaFiatFromHeight<S, C>>,
    /// Relative change through the represented block from the value just
    /// before a trailing window, divided by that value. Undefined (null) when
    /// it is zero, as for a window from genesis.
    rate: Windows<LazyDeltaPercentFromHeight<S, B>>,
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
        let version = version + super::VERSION;
        let (absolute, rate) = window_starts
            .map_with_suffix(|suffix, window_start| {
                let name = format!("{name}_{suffix}");
                let cents_name = format!("{name}_cents");
                let cents = LazyDeltaFromHeight::from_source(
                    &cents_name,
                    version,
                    source,
                    *window_start,
                    indexes,
                );
                let usd =
                    LazyPerBlock::from_resolutions::<Convert>(&name, version, &cents.resolutions);
                let absolute = LazyDeltaFiatFromHeight { usd, cents };

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
