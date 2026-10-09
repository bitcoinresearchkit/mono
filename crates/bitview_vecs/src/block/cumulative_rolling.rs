//! PerBlockCumulativeRolling - stored cumulative + lazy block and rolling views.
//!
//! The cumulative vector is the sole stored source of truth. Per-block values
//! and window sums are derived lazily from it.

use bitview_collections::Windows;
use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{
    AnyStoredVec, AnyVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecIndex,
    VecValue, WritableVec,
};

use crate::{IndexSources, LazyPreviousDeltaVec, LazyRollingSumsFromHeight, PerBlock};

#[derive(Traversable)]
pub struct PerBlockCumulativeRolling<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    pub block: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: PerBlock<T, M>,
    pub sum: LazyRollingSumsFromHeight<T>,
    last_cumulative: M::WriteOnly<Option<(usize, T)>>,
}

impl<T> PerBlockCumulativeRolling<T>
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
        let cumulative = PerBlock::import(db, &format!("{name}_cumulative"), version, indexes)?;
        let source = cumulative.resolutions.height_source();
        let block = LazyPreviousDeltaVec::new(name, version, source);
        let sum = LazyRollingSumsFromHeight::new(
            &format!("{name}_sum"),
            version,
            source,
            window_starts,
            indexes,
        );
        let last_cumulative = cumulative
            .height
            .collect_last()
            .map(|value| (cumulative.height.len(), value));

        Ok(Self {
            block,
            cumulative,
            sum,
            last_cumulative,
        })
    }

    pub fn cumulative_source(&self) -> &(impl ReadableCloneableVec<Height, T> + use<T>) {
        self.cumulative.resolutions.height_source()
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: T)
    where
        T: Copy,
    {
        let len = self.cumulative.height.len();
        let mut cumulative = match self.last_cumulative {
            Some((cached_len, value)) if cached_len == len => value,
            _ => self.cumulative.height.collect_last().unwrap_or_default(),
        };
        cumulative += value;
        self.cumulative.height.push(cumulative);
        self.last_cumulative = Some((len + 1, cumulative));
    }

    pub fn compute_cumulative_transformed<S>(
        &mut self,
        max_from: Height,
        source: &impl ReadableVec<Height, S>,
        mut transform: impl FnMut(S) -> T,
        exit: &Exit,
    ) -> Result<()>
    where
        S: VecValue,
        T: Copy,
    {
        self.last_cumulative = None;
        let mut cumulative = None;
        Ok(self.cumulative.height.compute_transform(
            max_from,
            source,
            |(height, value, this)| {
                let cumulative = cumulative.get_or_insert_with(|| {
                    height
                        .decremented()
                        .and_then(|height| this.collect_one(height))
                        .unwrap_or_default()
                });
                *cumulative += transform(value);
                (height, *cumulative)
            },
            exit,
        )?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compute_cumulative_sum_from_indexes<A, B, S>(
        &mut self,
        max_from: Height,
        first_indexes: &impl ReadableVec<Height, A>,
        indexes_count: &impl ReadableVec<Height, B>,
        source: &impl ReadableVec<A, S>,
        transform: impl FnMut(S) -> T,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecIndex + VecValue,
        B: VecValue,
        S: VecValue,
        usize: From<B>,
        T: Copy,
    {
        self.last_cumulative = None;
        Ok(self.cumulative.height.compute_cumulative_sum_from_indexes(
            max_from,
            first_indexes,
            indexes_count,
            source,
            transform,
            exit,
        )?)
    }

    pub fn validate_computed_version_or_reset(&mut self, version: Version) -> Result<()> {
        self.last_cumulative = None;
        self.cumulative
            .height
            .validate_computed_version_or_reset(version)?;
        Ok(())
    }

    pub fn validate_and_truncate(&mut self, version: Version, height: Height) -> Result<()> {
        self.last_cumulative = None;
        Ok(self
            .cumulative
            .height
            .validate_and_truncate(version, height)?)
    }

    pub fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
        self.last_cumulative = None;
        Ok(self.cumulative.height.truncate_if_needed_at(len)?)
    }

    pub fn write(&mut self) -> Result<()> {
        self.cumulative.height.write()?;
        Ok(())
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.last_cumulative = None;
        &mut self.cumulative.height
    }
}
