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
    fn seek_lower_inclusive(&mut self, needle: &[u8], _seqno: u64) -> bool {
        self.with_dependent_mut(|_, iter| iter.seek(needle))
    }

    fn seek_upper_inclusive(&mut self, needle: &[u8], _seqno: u64) -> bool {
        self.with_dependent_mut(|_, iter| iter.seek_upper(needle))
    }

    fn seek_lower_exclusive(&mut self, needle: &[u8], _seqno: u64) -> bool {
        self.with_dependent_mut(|_, iter| iter.seek_exclusive(needle))
    }

    fn seek_upper_exclusive(&mut self, needle: &[u8], _seqno: u64) -> bool {
        self.with_dependent_mut(|_, iter| iter.seek_upper_exclusive(needle))
    }

    pub fn seek_lower_bound(&mut self, bound: &Bound, seqno: u64) -> bool {
        match bound {
            Bound::Included(key) => self.seek_lower_inclusive(key, seqno),
            Bound::Excluded(key) => self.seek_lower_exclusive(key, seqno),
        }
    }

    pub fn seek_upper_bound(&mut self, bound: &Bound, seqno: u64) -> bool {
        match bound {
            Bound::Included(key) => self.seek_upper_inclusive(key, seqno),
            Bound::Excluded(key) => self.seek_upper_exclusive(key, seqno),
        }
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
