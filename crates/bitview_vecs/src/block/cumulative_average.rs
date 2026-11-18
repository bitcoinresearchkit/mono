use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};

use crate::{
    IndexSources, LazyPreviousDeltaVec, LazyRollingAvgsFromHeight, LazyRollingSumsFromHeight,
    PerBlockCumulative, shapes::Windows,
};

/// A stored running total with lazy exact per-block values, window sums and window averages.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(client = "Averaged")]
pub struct PerBlockCumulativeAverage<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    /// Value for the represented block.
    #[traversable(accumulation = Flow)]
    block: LazyPreviousDeltaVec<Height, T>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    running: PerBlockCumulative<T, M>,
    /// Exact total over each trailing window, from the same source as the averages.
    sum: LazyRollingSumsFromHeight<T>,
    avg: LazyRollingAvgsFromHeight<T>,
}

impl<T> PerBlockCumulativeAverage<T>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        // The running total and its windows are two versions above the per-block value: both
        // numbers are the stored ones, so no series' version moves.
        let cumulative_version = version + Version::TWO;
        let running = PerBlockCumulative::import(db, name, cumulative_version, indexes)?;
        let source = running.cumulative_source();
        let block = LazyPreviousDeltaVec::new(name, version, source);
        let sum = LazyRollingSumsFromHeight::new(
            &format!("{name}_sum"),
            cumulative_version,
            source,
            window_starts,
            indexes,
        );
        let avg = LazyRollingAvgsFromHeight::new(
            &format!("{name}_avg"),
            cumulative_version,
            source,
            window_starts,
            indexes,
        );

        Ok(Self {
            block,
            running,
            sum,
            avg,
        })
    }
}
