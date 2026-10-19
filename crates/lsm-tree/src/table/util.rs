// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    cmp::Ordering::{self, Equal, Greater},
    mem::size_of,
    path::PathBuf,
    sync::Arc,
};

use log::trace;

use super::{Block, BlockHandle, GlobalTableId};
use crate::{
    Cache, CompressionType, DescriptorTable, Error, KeyRange, Result, Table,
    table::block::BlockType, version::run::Ranged,
};

#[must_use]
pub fn aggregate_run_key_range(tables: &[Table]) -> KeyRange {
    #[expect(clippy::expect_used, reason = "runs are never empty by definition")]
    let lo = tables.first().expect("run should never be empty");
    #[expect(clippy::expect_used, reason = "runs are never empty by definition")]
    let hi = tables.last().expect("run should never be empty");
    KeyRange::new((lo.key_range().min().clone(), hi.key_range().max().clone()))
}

/// [start, end] slice indexes
#[derive(Debug)]
pub struct SliceIndexes(pub usize, pub usize);

/// Where a table's blocks come from: the block cache, else the table file through the shared
/// descriptor cache.
#[derive(Clone)]
pub struct BlockSource {
    pub table_id: GlobalTableId,
    pub path: Arc<PathBuf>,
    pub descriptor_table: Arc<DescriptorTable>,
    pub cache: Arc<Cache>,
}

impl BlockSource {
    /// Loads a block, from the block cache if cached.
    pub fn load(
        &self,
        handle: &BlockHandle,
        block_type: BlockType,
        compression: CompressionType,
    ) -> Result<Block> {
        trace!("load {block_type:?} block {handle:?}");
        self.cache
            .get_or_load(self.table_id, handle.offset(), block_type, || {
                let fd = self
                    .descriptor_table
                    .access_or_open(self.table_id, &self.path)?;
                let block = Block::from_file(&fd, *handle, compression)?;
                if block.header.block_type != block_type {
                    return Err(Error::InvalidTag((
                        "BlockType",
                        block.header.block_type.into(),
                    )));
                }
                Ok(block)
            })
    }
}

#[must_use]
pub fn longest_shared_prefix_length(s1: &[u8], s2: &[u8]) -> usize {
    const WORD_BYTES: usize = size_of::<usize>();

    let len = s1.len().min(s2.len());
    #[expect(
        clippy::indexing_slicing,
        reason = "len is bounded by both slice lengths"
    )]
    let s1 = &s1[..len];
    #[expect(
        clippy::indexing_slicing,
        reason = "len is bounded by both slice lengths"
    )]
    let s2 = &s2[..len];
    let (s1_words, s1_tail) = s1.as_chunks::<WORD_BYTES>();
    let (s2_words, s2_tail) = s2.as_chunks::<WORD_BYTES>();

    for (index, (s1_word, s2_word)) in s1_words.iter().zip(s2_words).enumerate() {
        let difference = usize::from_ne_bytes(*s1_word) ^ usize::from_ne_bytes(*s2_word);
        if difference != 0 {
            #[cfg(target_endian = "little")]
            let byte = difference.trailing_zeros() as usize / u8::BITS as usize;
            #[cfg(target_endian = "big")]
            let byte = difference.leading_zeros() as usize / u8::BITS as usize;

            return index * WORD_BYTES + byte;
        }
    }

    let word_bytes = s1_words.len() * WORD_BYTES;
    word_bytes
        + s1_tail
            .iter()
            .zip(s2_tail)
            .take_while(|(s1, s2)| s1 == s2)
            .count()
}

#[must_use]
pub fn compare_prefixed_slice(prefix: &[u8], suffix: &[u8], needle: &[u8]) -> Ordering {
    if needle.is_empty() {
        let combined_len = prefix.len() + suffix.len();
        return if combined_len > 0 { Greater } else { Equal };
    }

    let max_pfx_len = prefix.len().min(needle.len());

    {
        #[expect(unsafe_code, reason = "We checked for max_pfx_len")]
        let prefix = unsafe { prefix.get_unchecked(0..max_pfx_len) };

        #[expect(unsafe_code, reason = "We checked for max_pfx_len")]
        let needle = unsafe { needle.get_unchecked(0..max_pfx_len) };

        match prefix.cmp(needle) {
            Equal => {}
            ordering => return ordering,
        }
    }

    let rest_len = prefix.len().saturating_sub(needle.len());
    if rest_len > 0 {
        return Greater;
    }

    #[expect(
        unsafe_code,
        reason = "We know that the prefix is definitely not longer than the needle so we can safely truncate"
    )]
    let needle = unsafe { needle.get_unchecked(max_pfx_len..) };
    suffix.cmp(needle)
}
