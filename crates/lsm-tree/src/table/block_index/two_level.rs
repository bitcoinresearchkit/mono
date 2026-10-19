// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::{
    CompressionType, Result, Slice,
    table::{
        BlockHandle, IndexBlock, KeyedBlockHandle,
        block::BlockType,
        block_index::{BlockIndexIter, iter::OwnedIndexBlockIter},
        util::BlockSource,
    },
};

/// Index that translates item keys to data block handles
///
/// Only the top-level index is loaded into memory.
#[derive(Clone)]
pub struct TwoLevelBlockIndex {
    pub top_level_index: IndexBlock,
    pub source: BlockSource,
    pub compression: CompressionType,
}

impl TwoLevelBlockIndex {
    /// Loads the lower-level index block at `handle`.
    pub fn load(&self, handle: &BlockHandle) -> Result<IndexBlock> {
        self.source
            .load(handle, BlockType::Index, self.compression)
            .map(IndexBlock::new)
    }

    pub fn iter(&self) -> Iter {
        Iter {
            index: self.clone(),
            tli: None,
            lo_consumer: None,
            hi_consumer: None,
            lo: None,
            hi: None,
        }
    }
}

pub struct Iter {
    index: TwoLevelBlockIndex,
    tli: Option<OwnedIndexBlockIter>,

    lo_consumer: Option<OwnedIndexBlockIter>,
    hi_consumer: Option<OwnedIndexBlockIter>,

    lo: Option<(Slice, u64)>,
    hi: Option<(Slice, u64)>,
}

impl Iter {
    /// The bounded top-level iterator, created on first use; `None` if no handle is in bounds.
    fn tli(&mut self) -> Option<&mut OwnedIndexBlockIter> {
        if self.tli.is_none() {
            self.tli = Some(OwnedIndexBlockIter::bounded(
                self.index.top_level_index.clone(),
                self.lo.as_ref(),
                self.hi.as_ref(),
            )?);
        }
        self.tli.as_mut()
    }

    /// The bounded iterator over the lower-level block at `handle`; `None` if out of bounds.
    fn lower(&self, handle: &KeyedBlockHandle) -> Option<Result<OwnedIndexBlockIter>> {
        match self.index.load(handle.as_ref()) {
            Ok(block) => {
                OwnedIndexBlockIter::bounded(block, self.lo.as_ref(), self.hi.as_ref()).map(Ok)
            }
            Err(e) => Some(Err(e)),
        }
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
        if let Some(item) = self.lo_consumer.as_mut().and_then(Iterator::next) {
            return Some(Ok(item));
        }

        if let Some(handle) = self.tli()?.next() {
            let mut block = match self.lower(&handle)? {
                Ok(block) => block,
                Err(e) => return Some(Err(e)),
            };
            let item = block.next();
            self.lo_consumer = Some(block);
            if let Some(item) = item {
                return Some(Ok(item));
            }
        }

        // Nothing more found, consume from hi consumer
        self.hi_consumer.as_mut()?.next().map(Ok)
    }
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(item) = self
            .hi_consumer
            .as_mut()
            .and_then(DoubleEndedIterator::next_back)
        {
            return Some(Ok(item));
        }

        if let Some(handle) = self.tli()?.next_back() {
            let mut block = match self.lower(&handle)? {
                Ok(block) => block,
                Err(e) => return Some(Err(e)),
            };
            let item = block.next_back();
            self.hi_consumer = Some(block);
            if let Some(item) = item {
                return Some(Ok(item));
            }
        }

        // Nothing more found, consume from lo consumer
        self.lo_consumer.as_mut()?.next_back().map(Ok)
    }
}
