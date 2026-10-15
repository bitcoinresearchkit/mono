// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use super::KeyedBlockHandle;
use crate::{
    CompressionType, Result, Slice,
    table::{
        BlockHandle, IndexBlock,
        block::BlockType,
        block_index::{BlockIndexIter, iter::OwnedIndexBlockIter},
        util::BlockSource,
    },
};

/// Index that translates item keys to data block handles
///
/// The index is loaded on demand.
#[derive(Clone)]
pub struct VolatileBlockIndex {
    pub source: BlockSource,
    pub handle: BlockHandle,
    pub compression: CompressionType,
}

impl VolatileBlockIndex {
    pub fn load(&self) -> Result<IndexBlock> {
        self.source
            .load(&self.handle, BlockType::Index, self.compression)
            .map(IndexBlock::new)
    }

    pub fn iter(&self) -> Iter {
        Iter {
            index: self.clone(),
            inner: None,
            lo: None,
            hi: None,
        }
    }
}

pub struct Iter {
    index: VolatileBlockIndex,
    inner: Option<OwnedIndexBlockIter>,
    lo: Option<(Slice, u64)>,
    hi: Option<(Slice, u64)>,
}

impl Iter {
    /// The bounded index block iterator, loaded on first use; `None` if no handle is in bounds.
    fn inner(&mut self) -> Option<Result<&mut OwnedIndexBlockIter>> {
        if self.inner.is_none() {
            let block = match self.index.load() {
                Ok(block) => block,
                Err(e) => return Some(Err(e)),
            };
            self.inner = Some(OwnedIndexBlockIter::bounded(
                block,
                self.lo.as_ref(),
                self.hi.as_ref(),
            )?);
        }
        self.inner.as_mut().map(Ok)
    }
}

impl BlockIndexIter for Iter {
    fn seek_lower(&mut self, key: &[u8], seqno: u64) -> bool {
        self.lo = Some((key.into(), seqno));
        true
    }

    fn seek_upper(&mut self, key: &[u8], seqno: u64) -> bool {
        self.hi = Some((key.into(), seqno));
        true
    }
}

impl Iterator for Iter {
    type Item = Result<KeyedBlockHandle>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner()?.map(Iterator::next).transpose()
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner()?
            .map(DoubleEndedIterator::next_back)
            .transpose()
    }
}
