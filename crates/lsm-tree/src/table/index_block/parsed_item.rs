// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::cmp::Ordering;

use super::{BlockHandle, KeyedBlockHandle};
use crate::{
    Slice,
    table::{
        block::{BlockOffset, ParsedItem},
        util::{SliceIndexes, compare_prefixed_slice},
    },
};

#[derive(Debug)]
pub struct IndexBlockParsedItem {
    pub offset: BlockOffset,
    pub size: u32,
    pub prefix: Option<SliceIndexes>,
    pub end_key: SliceIndexes,
    pub seqno: u64,
}

impl ParsedItem<KeyedBlockHandle> for IndexBlockParsedItem {
    fn compare_key(&self, needle: &[u8], bytes: &[u8]) -> Ordering {
        if let Some(prefix) = &self.prefix {
            let prefix = unsafe { bytes.get_unchecked(prefix.0..prefix.1) };
            let rest_key = unsafe { bytes.get_unchecked(self.end_key.0..self.end_key.1) };
            compare_prefixed_slice(prefix, rest_key, needle)
        } else {
            let key = unsafe { bytes.get_unchecked(self.end_key.0..self.end_key.1) };
            key.cmp(needle)
        }
    }

    fn key_offset(&self) -> usize {
        self.end_key.0
    }

    fn materialize(&self, bytes: &Slice) -> KeyedBlockHandle {
        // NOTE: We consider the prefix and key slice indexes to be trustworthy
        #[expect(clippy::indexing_slicing)]
        let key = if let Some(prefix) = &self.prefix {
            let prefix_key = &bytes[prefix.0..prefix.1];
            let rest_key = &bytes[self.end_key.0..self.end_key.1];
            Slice::fused(prefix_key, rest_key)
        } else {
            bytes.slice(self.end_key.0..self.end_key.1)
        };

        KeyedBlockHandle::new(key, self.seqno, BlockHandle::new(self.offset, self.size))
    }
}
