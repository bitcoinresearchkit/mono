use bitview_collections::Windows;
use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::ReadableCloneableVec;

use crate::{IndexSources, LazyPreviousDeltaVec, LazyRollingAvgsFromHeight};

/// Lazy exact per-block values and rolling averages backed by one cumulative source.
#[derive(Traversable)]
pub struct LazyPerBlockCumulativeAverage<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    block: LazyPreviousDeltaVec<Height, T>,
    avg: LazyRollingAvgsFromHeight<T>,
}

impl<T> Clone for LazyPerBlockCumulativeAverage<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    fn clone(&self) -> Self {
        Self {
            block: self.block.clone(),
            avg: self.avg.clone(),
        }
    }
}

impl<T> LazyPerBlockCumulativeAverage<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    pub fn new(
        name: &str,
        version: Version,
        cumulative: &impl ReadableCloneableVec<Height, T>,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Self {
        Self {
            block: LazyPreviousDeltaVec::new(name, version, cumulative),
            avg: LazyRollingAvgsFromHeight::new(
                &format!("{name}_avg"),
                version + Version::TWO,
                cumulative,
                window_starts,
                indexes,
            ),
        }
    }
}
