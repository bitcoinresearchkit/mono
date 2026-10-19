use super::{DataBlock, bound::Bound, data_block::Iter as DataBlockIter};
use crate::{InternalValue, RecordBytes, Result};
use self_cell::self_cell;

self_cell!(
    pub struct OwnedDataBlockIter {
        owner: DataBlock,

        #[covariant]
        dependent: DataBlockIter,
    }
);

impl OwnedDataBlockIter {
    /// Skips the keys below `bound`.
    pub fn seek_lower_bound(&mut self, bound: &Bound) {
        self.with_dependent_mut(|_, iter| match bound {
            Bound::Included(key) => iter.seek_lower::<true>(key),
            Bound::Excluded(key) => iter.seek_lower::<false>(key),
        });
    }

    /// Skips the keys above `bound`.
    pub fn seek_upper_bound(&mut self, bound: &Bound) {
        self.with_dependent_mut(|_, iter| match bound {
            Bound::Included(key) => iter.seek_upper::<true>(key),
            Bound::Excluded(key) => iter.seek_upper::<false>(key),
        });
    }

    pub fn next_as<K: RecordBytes, V: RecordBytes>(
        &mut self,
    ) -> Option<Result<InternalValue<K, V>>> {
        self.with_dependent_mut(|block, iter| {
            iter.next()
                .map(|item| item.materialize_as(&block.inner.data))
        })
    }

    pub fn next_back_as<K: RecordBytes, V: RecordBytes>(
        &mut self,
    ) -> Option<Result<InternalValue<K, V>>> {
        self.with_dependent_mut(|block, iter| {
            iter.next_back()
                .map(|item| item.materialize_as(&block.inner.data))
        })
    }
}
