use brk_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Ident, ReadableCloneableVec, UnaryTransform};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, LazyPreviousDeltaVec, LazyRollingAvgsFromHeight, NumericValue,
        Windows,
    },
};

/// Lazy exact per-block values and rolling averages backed by one cumulative source.
#[derive(Traversable)]
pub struct LazyPerBlockCumulativeAverage<T, C = T, F = Ident>
where
    T: NumericValue + JsonSchema,
    C: NumericValue + JsonSchema,
    F: UnaryTransform<C, T>,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyPreviousDeltaVec<Height, C, T, F>,
    #[traversable(flatten)]
    pub average: LazyRollingAvgsFromHeight<C>,
}

impl<T, C, F> Clone for LazyPerBlockCumulativeAverage<T, C, F>
where
    T: NumericValue + JsonSchema,
    C: NumericValue + JsonSchema,
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
    C: NumericValue + JsonSchema,
    F: UnaryTransform<C, T>,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        cumulative: &(impl ReadableCloneableVec<Height, C> + 'static),
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        Self {
            block: LazyPreviousDeltaVec::transformed(
                name,
                version,
                cumulative.read_only_boxed_clone(),
            ),
            average: LazyRollingAvgsFromHeight::new(
                &format!("{name}_average"),
                version + Version::TWO,
                cumulative,
                cached_starts,
                indexes,
            ),
        }
    }
}
