use bitview_collections::Windows;
use bitview_compute::{ComputedVecValue, NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{ReadableCloneableVec, UnaryTransform};

use crate::{IndexSources, LazyPerBlock, LazyRollingComplete, PerBlockRolling};

/// Lazy analog of `PerBlockRolling<T>`: lazy cumulative + lazy rolling complete.
/// Derived by transforming a stored `PerBlockRolling<S1T>` (its cumulative and its distribution).
/// Zero stored vecs.
#[derive(Clone, Traversable)]
pub struct LazyPerBlockRolling<T, S1T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
    S1T: ComputedVecValue + JsonSchema + Quantity,
{
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    cumulative: LazyPerBlock<T, S1T::Sum>,
    #[traversable(flatten)]
    rolling: LazyRollingComplete<T, S1T>,
}

impl<T, S1T> LazyPerBlockRolling<T, S1T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T> + 'static,
    S1T: NumericValue + JsonSchema + Quantity,
{
    pub fn from_rolling<F: UnaryTransform<S1T, T> + UnaryTransform<S1T::Sum, T>>(
        name: &str,
        version: Version,
        source: &PerBlockRolling<S1T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        let cumulative = LazyPerBlock::from_resolutions::<F>(
            &format!("{name}_cumulative"),
            version,
            &source.cumulative,
        );

        let rolling = LazyRollingComplete::from_rolling_complete::<F>(
            name,
            version,
            &cumulative.height,
            &source.rolling,
            window_starts,
            indexes,
        );

        Self {
            cumulative,
            rolling,
        }
    }
}
