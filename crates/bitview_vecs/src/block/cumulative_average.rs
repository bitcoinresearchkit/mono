use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{ComputeVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecValue};

use crate::{
    shapes::Windows, CachedSeries, CumulativeState, IndexSources, LazyPreviousDeltaVec,
    LazyRollingAvgsFromHeight, LazyRollingSumsFromHeight, PerBlock,
};

/// Cumulative source of truth with lazy exact per-block values and rolling averages.
#[derive(Traversable)]
#[traversable(client = "Averaged")]
pub struct PerBlockCumulativeAverage<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    /// Value for the represented block.
    block: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block, sampled at each period's final block.
    cumulative: PerBlock<T, M>,
    /// Exact total over each trailing window, from the same source as the averages.
    sum: LazyRollingSumsFromHeight<T>,
    avg: LazyRollingAvgsFromHeight<T>,
    last: M::WriteOnly<CumulativeState<T>>,
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
        let cumulative_version = version + Version::TWO;
        let cumulative = PerBlock::import(
            db,
            &format!("{name}_cumulative"),
            cumulative_version,
            indexes,
        )?;
        let source = cumulative.resolutions.height_source();
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
            cumulative,
            sum,
            avg,
            last: Default::default(),
        })
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: T) {
        self.last.push(&mut self.cumulative.height, value);
    }

    pub fn compute_from<S>(
        &mut self,
        max_from: Height,
        source: &impl ReadableVec<Height, S>,
        mut transform: impl FnMut(Height, S) -> T,
        exit: &Exit,
    ) -> Result<()>
    where
        S: VecValue,
    {
        let last = &mut self.last;
        Ok(self.cumulative.height.compute_transform(
            max_from,
            source,
            |(height, value, this)| {
                let value = transform(height, value);
                last.add_at(height, this, value)
            },
            exit,
        )?)
    }

    pub fn cumulative_source(&self) -> &CachedSeries<Height, T> {
        &self.cumulative.height
    }
}
