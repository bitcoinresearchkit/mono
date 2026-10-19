// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::cmp::Ordering;

use crate::{
    double_ended_peekable::{DoubleEndedPeekable, DoubleEndedPeekableExt},
    table::{KeyedBlockHandle, block::Decoder, index_block::IndexBlockParsedItem},
};

pub struct Iter<'a> {
    decoder: DoubleEndedPeekable<
        IndexBlockParsedItem,
        Decoder<'a, KeyedBlockHandle, IndexBlockParsedItem>,
    >,
}

impl<'a> Iter<'a> {
    #[must_use]
    pub fn new(decoder: Decoder<'a, KeyedBlockHandle, IndexBlockParsedItem>) -> Self {
        let decoder = decoder.double_ended_peekable();
        Self { decoder }
    }

    pub fn seek(&mut self, needle: &[u8], seqno: u64) -> bool {
        self.decoder.inner_mut().seek(
            |end_key, s| match end_key.cmp(needle) {
                Ordering::Greater => false,
                Ordering::Less => true,
                Ordering::Equal => s >= seqno,
            },
            true,
        )
    }

    pub fn seek_upper(&mut self, needle: &[u8], _seqno: u64) -> bool {
        self.decoder
            .inner_mut()
            .seek_upper(|end_key, _s| end_key <= needle, true)
    }
}

impl Iterator for Iter<'_> {
    type Item = IndexBlockParsedItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.next()
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.decoder.next_back()
    }
}
