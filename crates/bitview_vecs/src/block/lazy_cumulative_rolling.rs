use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{Ident, ReadableCloneableVec};

use crate::{
    IndexSources, LazyPerBlock, LazyPreviousDeltaVec, LazyRollingSumsFromHeight, shapes::Windows,
};

#[derive(Clone, Traversable)]
#[traversable(client = "Flow")]
pub struct LazyPerBlockCumulativeRolling<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    /// Value for the represented block.
    pub block: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyPerBlock<T>,
    pub sum: LazyRollingSumsFromHeight<T>,
}

/// A flow's cumulative value and trailing-window sums without its per-block value, for a count
/// whose block value says nothing: one block per block, a pool's zero or one.
#[derive(Clone, Traversable)]
#[traversable(client = "CumulativeFlow")]
pub struct LazyCumulativeRolling<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: LazyPerBlock<T>,
    pub sum: LazyRollingSumsFromHeight<T>,
}

impl<T> LazyCumulativeRolling<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    pub fn from_cumulative_source<V>(
        name: &str,
        version: Version,
        source: &V,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self
    where
        V: ReadableCloneableVec<Height, T> + ?Sized,
    {
        let cumulative = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_cumulative"),
            version,
            source,
            indexes,
        );
        let sum = LazyRollingSumsFromHeight::new(
            &format!("{name}_sum"),
            version,
            &cumulative.height,
            window_starts,
            indexes,
        );
        Self { cumulative, sum }
    }
}

impl<T> LazyPerBlockCumulativeRolling<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    pub fn from_cumulative_source<V>(
        name: &str,
        version: Version,
        source: &V,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        indexes: &IndexSources,
    ) -> Self
    where
        V: ReadableCloneableVec<Height, T> + ?Sized,
    {
        let LazyCumulativeRolling { cumulative, sum } =
            LazyCumulativeRolling::from_cumulative_source(
                name,
                version,
                source,
                window_starts,
                indexes,
            );
        let block = LazyPreviousDeltaVec::new(name, version, &cumulative.height);
        Self {
            block,
            cumulative,
            sum,
        }
    }
}
