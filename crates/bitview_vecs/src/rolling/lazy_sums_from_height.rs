use bitview_compute::NumericValue;
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{DeltaOp, DeltaSub, ReadableCloneableVec};

use crate::{IndexSources, LazyDeltaFromHeight, shapes::Windows};

/// Window sums `cum[h] - cum[window_start[h] - 1]` over a cumulative source, using zero when the
/// window starts at genesis; `Op` the sum's (a dollar one's: [`crate::PricedSum`]).
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyRollingSumsFromHeight<T, Op = DeltaSub>(
    /// Total of the per-block values over the trailing window ending at the
    /// represented block. At time-period indexes, the value is taken at the
    /// period's final block.
    pub Windows<LazyDeltaFromHeight<T, T, Op>>,
)
where
    T: NumericValue + JsonSchema,
    Op: 'static;

impl<T, Op> LazyRollingSumsFromHeight<T, Op>
where
    T: NumericValue + JsonSchema,
    Op: DeltaOp<T, T>,
{
    pub fn new(
        name: &str,
        version: Version,
        cumulative: &impl ReadableCloneableVec<Height, T>,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self {
        Self(window_starts.map_with_suffix(|suffix, window_start| {
            LazyDeltaFromHeight::from_source(
                &format!("{name}_{suffix}"),
                version,
                cumulative,
                *window_start,
                indexes,
            )
        }))
    }
}
