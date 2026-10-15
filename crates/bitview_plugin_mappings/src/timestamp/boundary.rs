use std::marker::PhantomData;

use brk_types::{Height, Timestamp, Version};
use vecdb::{AnyVec, ReadOnlyClone, ReadableBoxedVec, ReadableVec, TypedVec, VecIndex};

/// Timestamp of the first block in each fixed-size block period.
#[derive(Clone)]
pub struct BoundaryTimestampVec<I: VecIndex> {
    timestamps: ReadableBoxedVec<Height, Timestamp>,
    blocks_per_period: usize,
    index: PhantomData<I>,
}

impl<I: VecIndex> BoundaryTimestampVec<I> {
    pub fn new(timestamps: ReadableBoxedVec<Height, Timestamp>, blocks_per_period: usize) -> Self {
        debug_assert_ne!(blocks_per_period, 0);
        Self {
            timestamps,
            blocks_per_period,
            index: PhantomData,
        }
    }

    fn source_index(&self, period: usize) -> usize {
        period * self.blocks_per_period
    }

    fn source_indices(&self, periods: impl Iterator<Item = usize>) -> Vec<usize> {
        let len = self.len();
        periods
            .take_while(|period| *period < len)
            .map(|period| self.source_index(period))
            .collect()
    }

    fn values(&self, from: usize, to: usize) -> Vec<Timestamp> {
        let indices = self.source_indices(from..to);
        self.timestamps.read_sorted_at(&indices)
    }
}

impl<I: VecIndex> AnyVec for BoundaryTimestampVec<I> {
    fn version(&self) -> Version {
        self.timestamps.version()
    }

    fn name(&self) -> &str {
        "timestamp"
    }

    fn len(&self) -> usize {
        self.timestamps.len().div_ceil(self.blocks_per_period)
    }

    fn is_mutable(&self) -> bool {
        self.timestamps.is_mutable()
    }

    vecdb::any_vec_type_methods!(I, Timestamp);
}

impl<I: VecIndex> TypedVec for BoundaryTimestampVec<I> {
    type I = I;
    type T = Timestamp;
}

impl<I: VecIndex> ReadableVec<I, Timestamp> for BoundaryTimestampVec<I> {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Timestamp>) {
        let indices = self.source_indices(from..to);
        self.timestamps.read_sorted_into_at(&indices, buf);
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Timestamp)) {
        self.values(from, to).into_iter().for_each(each);
    }

    fn fold_range_at<B, F: FnMut(B, Timestamp) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> B {
        self.values(from, to).into_iter().fold(init, fold)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Timestamp) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.values(from, to).into_iter().try_fold(init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<Timestamp> {
        (index < self.len())
            .then(|| self.source_index(index))
            .and_then(|index| self.timestamps.collect_one_at(index))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Timestamp>) {
        let indices = self.source_indices(indices.iter().copied());
        self.timestamps.read_sorted_into_at(&indices, out);
    }
}

impl<I: VecIndex> ReadOnlyClone for BoundaryTimestampVec<I> {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

bitview_traversable::leaf!([I: VecIndex] BoundaryTimestampVec<I>);
