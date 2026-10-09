use bitview_collections::Windows;
use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{
    AnyStoredVec, AnyVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecValue,
    WritableVec,
};

use crate::{
    CachedSeries, IndexSources, LazyPreviousDeltaVec, LazyRollingAvgsFromHeight, import_cached,
};

/// Cumulative source of truth with lazy exact per-block values and rolling averages.
#[derive(Traversable)]
pub struct PerBlockCumulativeAverage<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    block: LazyPreviousDeltaVec<Height, T>,
    #[traversable(hidden)]
    cumulative: CachedSeries<Height, T, M>,
    avg: LazyRollingAvgsFromHeight<T>,
    last_cumulative: M::WriteOnly<Option<(usize, T)>>,
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
        let last_cumulative = cumulative
            .collect_last()
            .map(|value| (cumulative.len(), value));
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
            last_cumulative,
        })
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: T)
    where
        T: Copy,
    {
        let len = self.cumulative.len();
        let mut cumulative = match self.last_cumulative {
            Some((cached_len, value)) if cached_len == len => value,
            _ => self.cumulative.collect_last().unwrap_or_default(),
        };
        cumulative += value;
        self.cumulative.push(cumulative);
        self.last_cumulative = Some((len + 1, cumulative));
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
        T: Copy,
    {
        let mut cumulative = None;
        self.cumulative.compute_transform(
            max_from,
            source,
            |(height, value, this)| {
                let cumulative = cumulative.get_or_insert_with(|| {
                    height
                        .decremented()
                        .and_then(|height| this.collect_one(height))
                        .unwrap_or_default()
                });
                *cumulative += transform(height, value);
                (height, *cumulative)
            },
            exit,
        )?;
        self.last_cumulative = None;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.last_cumulative = None;
        self.cumulative.reset()?;
        Ok(())
    }

    /// The stored running total the views derive from.
    pub fn cumulative_source(&self) -> &CachedSeries<Height, T> {
        &self.cumulative
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.last_cumulative = None;
        &mut self.cumulative
    }
}
