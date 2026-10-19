use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{
    AnyVec, ComputeVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecValue,
    WritableVec,
};

use crate::{
    CachedSeries, CumulativeState, IndexSources, LazyPreviousDeltaVec, LazyRollingAvgsFromHeight,
    import_cached, shapes::Windows,
};

/// Cumulative source of truth with lazy exact per-block values and rolling averages.
#[derive(Traversable)]
pub struct PerBlockCumulativeAverage<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    /// Value for the represented block.
    block: LazyPreviousDeltaVec<Height, T>,
    #[traversable(hidden)]
    cumulative: CachedSeries<Height, T, M>,
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
        let cumulative = import_cached(db, &format!("{name}_cumulative"), cumulative_version)?;
        let block = LazyPreviousDeltaVec::new(name, version, &cumulative);
        let avg = LazyRollingAvgsFromHeight::new(
            &format!("{name}_avg"),
            cumulative_version,
            &cumulative,
            window_starts,
            indexes,
        );

        Ok(Self {
            block,
            cumulative,
            avg,
            last: Default::default(),
        })
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: T) {
        let height = Height::from(self.cumulative.len());
        let cumulative = self
            .last
            .add_at(height, &self.cumulative, |total| *total += value);
        self.cumulative.push(cumulative);
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
        Ok(self.cumulative.compute_transform(
            max_from,
            source,
            |(height, value, this)| {
                let value = transform(height, value);
                last.add_at(height, this, |total| *total += value)
            },
            exit,
        )?)
    }

    pub fn cumulative_source(&self) -> &CachedSeries<Height, T> {
        &self.cumulative
    }
}
