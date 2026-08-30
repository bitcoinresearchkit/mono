// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use super::{Block, BlockHandle, GlobalTableId};
use crate::{
    Cache, CompressionType, KeyRange, Table, file_accessor::FileAccessor, table::block::BlockType,
    version::run::Ranged,
};
use std::{mem::size_of, path::Path};

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

/// Loads a block from disk or block cache, if cached.
///
/// Also handles file descriptor opening and caching.
#[warn(clippy::too_many_arguments)]
pub fn load_block(
    table_id: GlobalTableId,
    path: &Path,
    file_accessor: &FileAccessor,
    cache: &Cache,
    handle: &BlockHandle,
    block_type: BlockType,
    compression: CompressionType,
) -> crate::Result<Block> {
    log::trace!("load {block_type:?} block {handle:?}");

    if let Some(block) = cache.get_block(table_id, handle.offset()) {
        return Ok(block);
    }

    let fd = file_accessor.access_or_open(table_id, path)?;
    let block = Block::from_file(&fd, *handle, compression)?;

    if block.header.block_type != block_type {
        return Err(crate::Error::InvalidTag((
            "BlockType",
            block.header.block_type.into(),
        )));
    }

    cache.insert_block(table_id, handle.offset(), block.clone());

    Ok(block)
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
pub fn compare_prefixed_slice(prefix: &[u8], suffix: &[u8], needle: &[u8]) -> std::cmp::Ordering {
    use std::cmp::Ordering::{Equal, Greater};

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

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    #[expect(
        clippy::indexing_slicing,
        reason = "the index comes from the array's range"
    )]
    fn test_longest_shared_prefix_length() {
        assert_eq!(3, longest_shared_prefix_length(b"abc", b"abc"));
        assert_eq!(1, longest_shared_prefix_length(b"abc", b"a"));
        assert_eq!(1, longest_shared_prefix_length(b"a", b"abc"));
        assert_eq!(0, longest_shared_prefix_length(b"abc", b""));
        assert_eq!(0, longest_shared_prefix_length(b"", b"abc"));
        assert_eq!(0, longest_shared_prefix_length(b"", b""));
        assert_eq!(0, longest_shared_prefix_length(b"", b""));
        assert_eq!(0, longest_shared_prefix_length(b"abc", b"def"));
        assert_eq!(1, longest_shared_prefix_length(b"abc", b"acc"));

        let shared = [42; 3 * size_of::<usize>()];
        assert_eq!(shared.len(), longest_shared_prefix_length(&shared, &shared));

        for different_at in 0..shared.len() {
            let mut different = shared;
            different[different_at] = 24;
            assert_eq!(
                different_at,
                longest_shared_prefix_length(&shared, &different),
            );
        }
    }

    #[test]
    fn test_compare_prefixed_slice() {
        use std::cmp::Ordering::{Equal, Greater, Less};

        assert_eq!(Greater, compare_prefixed_slice(&[0, 161], &[], &[0]));

        assert_eq!(Equal, compare_prefixed_slice(b"abc", b"xyz", b"abcxyz"));
        assert_eq!(Equal, compare_prefixed_slice(b"abc", b"", b"abc"));
        assert_eq!(Equal, compare_prefixed_slice(b"abc", b"abc", b"abcabc"));
        assert_eq!(Equal, compare_prefixed_slice(b"", b"", b""));
        assert_eq!(Less, compare_prefixed_slice(b"a", b"", b"y"));
        assert_eq!(Less, compare_prefixed_slice(b"a", b"", b"yyy"));
        assert_eq!(Less, compare_prefixed_slice(b"a", b"", b"yyy"));
        assert_eq!(Less, compare_prefixed_slice(b"yyyy", b"a", b"yyyyb"));
        assert_eq!(Less, compare_prefixed_slice(b"yyy", b"b", b"yyyyb"));
        assert_eq!(Less, compare_prefixed_slice(b"abc", b"d", b"abce"));
        assert_eq!(Less, compare_prefixed_slice(b"ab", b"", b"ac"));
        assert_eq!(Greater, compare_prefixed_slice(b"a", b"", b""));
        assert_eq!(Greater, compare_prefixed_slice(b"", b"a", b""));
        assert_eq!(Greater, compare_prefixed_slice(b"a", b"a", b""));
        assert_eq!(Greater, compare_prefixed_slice(b"b", b"a", b"a"));
        assert_eq!(Greater, compare_prefixed_slice(b"a", b"b", b"a"));
        assert_eq!(Greater, compare_prefixed_slice(b"abc", b"xy", b"abcw"));
        assert_eq!(Greater, compare_prefixed_slice(b"ab", b"cde", b"a"));
        assert_eq!(Greater, compare_prefixed_slice(b"abcd", b"zz", b"abc"));
        assert_eq!(Greater, compare_prefixed_slice(b"abc", b"d", b"abc"));
        assert_eq!(
            Greater,
            compare_prefixed_slice(b"aaaa", b"aaab", b"aaaaaaaa")
        );
        assert_eq!(
            Greater,
            compare_prefixed_slice(b"aaaa", b"aaba", b"aaaaaaaa")
        );
        assert_eq!(Greater, compare_prefixed_slice(b"abcd", b"x", b"abc"));

        assert_eq!(Less, compare_prefixed_slice(&[0x7F], &[], &[0x80]));
        assert_eq!(Greater, compare_prefixed_slice(&[0xFF], &[], &[0x10]));
    }
}
