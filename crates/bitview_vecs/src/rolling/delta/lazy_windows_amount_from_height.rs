use bitview_collections::Windows;
use bitview_compute::FixedRatio;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use vecdb::{ReadableCloneableVec, VecValue};

use crate::{
    AmountType, IndexSources, LazyDeltaAmountFromHeight, LazyDeltaFromHeight,
    LazyDeltaPercentFromHeight, LazyPerBlock,
};

#[derive(Clone, Traversable)]
pub struct LazyRollingDeltasAmountFromHeight<S, C, B>
where
    S: VecValue,
    C: AmountType,
    B: FixedRatio,
{
    /// Absolute change from the start of a trailing window through the
    /// represented block.
    absolute: Windows<LazyDeltaAmountFromHeight<S, C>>,
    /// Relative change from the start of a trailing window through the
    /// represented block, divided by the starting value. Returns zero when the
    /// starting value is zero.
    rate: Windows<LazyDeltaPercentFromHeight<S, B>>,
}

impl<S, C, B> LazyRollingDeltasAmountFromHeight<S, C, B>
where
    S: VecValue + Into<f64>,
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
                let btc = LazyPerBlock::from_resolutions::<C::ToBitcoin>(
                    &name,
                    version,
                    &sats.resolutions,
                );
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
