use bitview_collections::Windows;
use bitview_compute::FixedRatio;
use bitview_transforms::Convert;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{ReadableCloneableVec, VecValue};

use crate::{
    FiatType, IndexSources, LazyDeltaFiatFromHeight, LazyDeltaFromHeight,
    LazyDeltaPercentFromHeight, LazyPerBlock,
};

#[derive(Clone, Traversable)]
pub struct LazyRollingDeltasFiatFromHeight<S, C, B>
where
    S: VecValue,
    C: FiatType,
    B: FixedRatio,
{
    /// Absolute change from the start of a trailing window through the
    /// represented block.
    pub absolute: Windows<LazyDeltaFiatFromHeight<S, C>>,
    /// Relative change from the start of a trailing window through the
    /// represented block, divided by the starting value. Returns zero when the
    /// starting value is zero.
    rate: Windows<LazyDeltaPercentFromHeight<S, B>>,
}

impl<S, C, B> LazyRollingDeltasFiatFromHeight<S, C, B>
where
    S: VecValue + Into<f64>,
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
