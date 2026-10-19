// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::cmp::Ordering;

use crate::{
    InternalValue,
    double_ended_peekable::{DoubleEndedPeekable, DoubleEndedPeekableExt},
    table::{
        block::{Decoder, ParsedItem},
        data_block::DataBlockParsedItem,
    },
};

/// The data block iterator handles double-ended scans over a data block
pub struct Iter<'a> {
    bytes: &'a [u8],
    decoder:
        DoubleEndedPeekable<DataBlockParsedItem, Decoder<'a, InternalValue, DataBlockParsedItem>>,
}

impl<'a> Iter<'a> {
    /// Creates a new iterator over a data block.
    #[must_use]
    pub fn new(bytes: &'a [u8], decoder: Decoder<'a, InternalValue, DataBlockParsedItem>) -> Self {
        let decoder = decoder.double_ended_peekable();
        Self { bytes, decoder }
    }

    /// Skips the keys below `needle`, and `needle` itself unless `INCLUSIVE`.
    pub fn seek_lower<const INCLUSIVE: bool>(&mut self, needle: &[u8]) {
        // Jump to the restart interval whose head key is the last one strictly below `needle`,
        // then scan linearly to the first key past the bound.
        if !self
            .decoder
            .inner_mut()
            .seek(|head_key, _| head_key < needle, false)
        {
            return;
        }

        while let Some(item) = self.decoder.peek() {
            match item.compare_key(needle, self.bytes) {
                Ordering::Less => {}
                Ordering::Equal if !INCLUSIVE => {}
                Ordering::Equal | Ordering::Greater => return,
            }
            self.decoder.next();
        }
    }

    /// Skips the keys above `needle`, and `needle` itself unless `INCLUSIVE`.
    pub fn seek_upper<const INCLUSIVE: bool>(&mut self, needle: &[u8]) {
        // Jump the high end to the first restart whose head key is ≤ `needle`, then walk
        // backwards to the last key within the bound.
        if !self
            .decoder
            .inner_mut()
            .seek_upper(|head_key, _| head_key <= needle, false)
        {
            return;
        }

        while let Some(item) = self.decoder.peek_back() {
            match item.compare_key(needle, self.bytes) {
                Ordering::Greater => {}
                Ordering::Equal if !INCLUSIVE => {}
                Ordering::Equal | Ordering::Less => return,
            }
            self.decoder.next_back();
        }
    }
}

impl Iterator for Iter<'_> {
    type Item = DataBlockParsedItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.next()
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.decoder.next_back()
    }
}
