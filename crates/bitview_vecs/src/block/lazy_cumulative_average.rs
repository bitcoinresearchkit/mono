use bitview_collections::Windows;
use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Ident, ReadableCloneableVec, UnaryTransform};

use crate::{IndexSources, LazyPreviousDeltaVec, LazyRollingAvgsFromHeight};

/// Lazy exact per-block values and rolling averages backed by one cumulative source.
#[derive(Traversable)]
pub struct LazyPerBlockCumulativeAverage<T, C = T, F = Ident>
where
    T: NumericValue + JsonSchema,
    C: NumericValue + JsonSchema + Quantity,
    F: UnaryTransform<C, T>,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    block: LazyPreviousDeltaVec<Height, C, T, F>,
    #[traversable(flatten)]
    average: LazyRollingAvgsFromHeight<C>,
}

impl<T, C, F> Clone for LazyPerBlockCumulativeAverage<T, C, F>
where
    T: NumericValue + JsonSchema,
    C: NumericValue + JsonSchema + Quantity,
    F: UnaryTransform<C, T>,
{
    fn clone(&self) -> Self {
        Self {
            block: self.block.clone(),
            average: self.average.clone(),
        }
    }
}

impl<T, C, F> LazyPerBlockCumulativeAverage<T, C, F>
where
    T: NumericValue + JsonSchema,
    C: NumericValue + JsonSchema + Quantity,
    F: UnaryTransform<C, T>,
{
    pub fn new(
        name: &str,
        version: Version,
        cumulative: &impl ReadableCloneableVec<Height, C>,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Self {
        Self {
            block: LazyPreviousDeltaVec::transformed(name, version, cumulative),
            average: LazyRollingAvgsFromHeight::new(
                &format!("{name}_average"),
                version + Version::TWO,
                cumulative,
                window_starts,
                indexes,
            ),
        }
    }
}
