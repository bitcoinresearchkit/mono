//! A stored running total and lazy window sums, with a block view that usually
//! reads the difference between cumulative values.

use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, DeltaOp, DeltaSub, ReadableCloneableVec, Rw, StorageMode};

use crate::{
    IndexSources, LazyPreviousDeltaVec, LazyRollingSumsFromHeight, PerBlockCumulative, PricedSum,
    shapes::Windows,
};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(client = "Flow")]
pub struct PerBlockCumulativeRolling<
    T,
    M: StorageMode = Rw,
    Op = DeltaSub,
    B = LazyPreviousDeltaVec<Height, T>,
> where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
    Op: Clone + 'static,
    B: Clone + Traversable,
{
    /// Value for the represented block.
    #[traversable(accumulation = Flow)]
    pub block: B,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    running: PerBlockCumulative<T, M>,
    pub sum: LazyRollingSumsFromHeight<T, Op>,
}

/// A flow valued in dollars: its window sums none until wholly priced (see [`crate::priced`]).
pub type PricedPerBlockCumulativeRolling<T, M = Rw> = PerBlockCumulativeRolling<T, M, PricedSum>;

impl<T, Op> PerBlockCumulativeRolling<T, Rw, Op>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
    Op: DeltaOp<T, T> + Clone,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        Self::import_with_block(db, name, version, indexes, window_starts, |running| {
            LazyPreviousDeltaVec::new(name, version, running.cumulative_source())
        })
    }
}

impl<T, Op, B> PerBlockCumulativeRolling<T, Rw, Op, B>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
    Op: DeltaOp<T, T> + Clone,
    B: Clone + Traversable,
{
    pub fn import_with_block(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        block: impl FnOnce(&PerBlockCumulative<T>) -> B,
    ) -> Result<Self> {
        let running = PerBlockCumulative::import(db, name, version, indexes)?;
        let block = block(&running);
        let sum = LazyRollingSumsFromHeight::new(
            &format!("{name}_sum"),
            version,
            running.cumulative_source(),
            window_starts,
            indexes,
        );

        Ok(Self {
            block,
            running,
            sum,
        })
    }
}

/// A stored running total and its trailing-window sums, without a per-block value: for a count
/// whose block value says nothing, zero or one a block (as a pool's blocks, see
/// [`crate::LazyCumulativeRolling`]).
#[derive(Deref, DerefMut, Traversable)]
#[traversable(client = "CumulativeFlow")]
pub struct CumulativeRolling<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    running: PerBlockCumulative<T, M>,
    pub sum: LazyRollingSumsFromHeight<T>,
}

impl<T> CumulativeRolling<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
    DeltaSub: DeltaOp<T, T>,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        let running = PerBlockCumulative::import(db, name, version, indexes)?;
        let sum = LazyRollingSumsFromHeight::new(
            &format!("{name}_sum"),
            version,
            running.cumulative.resolutions.height_source(),
            window_starts,
            indexes,
        );
        Ok(Self { running, sum })
    }
}
