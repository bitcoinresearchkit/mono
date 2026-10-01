use bitview_collections::Windows;
use bitview_compute::{ComputedVecValue, NumericValue};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{ReadableCloneableVec, UnaryTransform};

use crate::{IndexSources, LazyPerBlock, LazyRollingComplete, PerBlock, RollingComplete};

/// Lazy analog of `PerBlockRolling<T>`: lazy cumulative + lazy rolling complete.
/// Derived by transforming another metric's cumulative and rolling parts.
/// Zero stored vecs.
#[derive(Clone, Traversable)]
pub struct LazyPerBlockRolling<T, S1T>
where
    T: NumericValue + JsonSchema,
    S1T: ComputedVecValue + JsonSchema,
{
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    cumulative: LazyPerBlock<T, S1T>,
    #[traversable(flatten)]
    rolling: LazyRollingComplete<T, S1T>,
}

impl<T, S1T> LazyPerBlockRolling<T, S1T>
where
    T: NumericValue + JsonSchema + 'static,
    S1T: NumericValue + JsonSchema,
{
    pub fn from_full_parts<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        source_cumulative: &PerBlock<S1T>,
        source_rolling: &RollingComplete<S1T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        let cumulative = LazyPerBlock::from_resolutions::<F>(
            &format!("{name}_cumulative"),
            version,
            source_cumulative,
        );

        let rolling = LazyRollingComplete::from_rolling_complete::<F>(
            name,
            version,
            &cumulative.height,
            source_rolling,
            window_starts,
            indexes,
        );

        Self {
            cumulative,
            rolling,
        }
    }
}
