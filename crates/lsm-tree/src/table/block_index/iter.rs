// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::{
    Slice,
    table::{IndexBlock, KeyedBlockHandle, block::ParsedItem, index_block::Iter as IndexBlockIter},
};
use self_cell::self_cell;

self_cell!(
    pub struct OwnedIndexBlockIter {
        owner: IndexBlock,

        #[covariant]
        dependent: IndexBlockIter,
    }
);

impl OwnedIndexBlockIter {
    /// Iterates over `block` within the bounds; `None` if no handle overlaps them.
    pub fn bounded(
        block: IndexBlock,
        lo: Option<&(Slice, u64)>,
        hi: Option<&(Slice, u64)>,
    ) -> Option<Self> {
        let mut iter = Self::new(block, IndexBlock::iter);
        (lo.is_none_or(|(key, seqno)| iter.seek_lower(key, *seqno))
            && hi.is_none_or(|(key, seqno)| iter.seek_upper(key, *seqno)))
        .then_some(iter)
    }

    pub fn seek_lower(&mut self, needle: &[u8], seqno: u64) -> bool {
        self.with_dependent_mut(|_, m| m.seek(needle, seqno))
    }

    pub fn seek_upper(&mut self, needle: &[u8], seqno: u64) -> bool {
        self.with_dependent_mut(|_, m| m.seek_upper(needle, seqno))
    }
}

impl Iterator for OwnedIndexBlockIter {
    type Item = KeyedBlockHandle;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|block, iter| {
            iter.next().map(|item| item.materialize(&block.inner.data))
        })
    }
}

impl DoubleEndedIterator for OwnedIndexBlockIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.with_dependent_mut(|block, iter| {
            iter.next_back()
                .map(|item| item.materialize(&block.inner.data))
        })
    }
}
