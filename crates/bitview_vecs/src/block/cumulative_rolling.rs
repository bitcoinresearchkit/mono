//! The cumulative vector is the sole stored source of truth. Per-block values
//! and window sums are derived lazily from it.

use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{
    AnyStoredVec, AnyVec, ComputeVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode,
    VecIndex, VecValue, WritableVec,
};

use crate::{
    CumulativeState, IndexSources, LazyPreviousDeltaVec, LazyRollingSumsFromHeight, PerBlock,
    shapes::Windows,
};

#[derive(Traversable)]
pub struct PerBlockCumulativeRolling<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity<Sum = T>,
{
    /// Value for the represented block.
    pub block: LazyPreviousDeltaVec<Height, T>,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: PerBlock<T, M>,
    pub sum: LazyRollingSumsFromHeight<T>,
    last: M::WriteOnly<CumulativeState<T>>,
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

        Ok(Self {
            block,
            cumulative,
            sum,
            last: Default::default(),
        })
    }

    pub fn cumulative_source(&self) -> &(impl ReadableCloneableVec<Height, T> + use<T>) {
        self.cumulative.resolutions.height_source()
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: T) {
        let height = Height::from(self.cumulative.height.len());
        let cumulative = self
            .last
            .add_at(height, &self.cumulative.height, |total| *total += value);
        self.cumulative.height.push(cumulative);
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
    {
        let last = &mut self.last;
        Ok(self.cumulative.height.compute_transform(
            max_from,
            source,
            |(height, value, this)| {
                let value = transform(value);
                last.add_at(height, this, |total| *total += value)
            },
            exit,
        )?)
    }

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
        self.last = CumulativeState::default();
        Ok(self.cumulative.height.compute_cumulative_sum_from_indexes(
            max_from,
            first_indexes,
            indexes_count,
            source,
            transform,
            exit,
        )?)
    }

    /// A pure borrow: a length change reloads the cached total.
    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.cumulative.height
    }
}
