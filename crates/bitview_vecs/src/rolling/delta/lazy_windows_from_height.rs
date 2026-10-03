use bitview_collections::Windows;
use bitview_compute::{FixedRatio, NumericValue};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{DeltaChange, ReadableCloneableVec, VecValue};

use crate::{IndexSources, LazyDeltaFromHeight, LazyDeltaPercentFromHeight};

#[derive(Clone, Traversable)]
pub struct LazyRollingDeltasFromHeight<S, C, B>
where
    S: VecValue,
    C: NumericValue + JsonSchema,
    B: FixedRatio,
{
    /// Absolute change from the start of a trailing window through the
    /// represented block.
    absolute: Windows<LazyDeltaFromHeight<S, C, DeltaChange>>,
    /// Relative change from the start of a trailing window through the
    /// represented block, divided by the starting value. Returns zero when the
    /// starting value is zero.
    rate: Windows<LazyDeltaPercentFromHeight<S, B>>,
}

impl<S, C, B> LazyRollingDeltasFromHeight<S, C, B>
where
    S: VecValue + Into<f64>,
    C: NumericValue + JsonSchema + From<f64>,
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
                let absolute = LazyDeltaFromHeight::from_source(
                    &name,
                    version,
                    source,
                    *window_start,
                    indexes,
                );

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
