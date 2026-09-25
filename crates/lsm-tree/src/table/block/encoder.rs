// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::io::Write;

use super::{
    super::{
        block::binary_index::Builder as BinaryIndexBuilder, util::longest_shared_prefix_length,
    },
    Trailer,
};
use crate::Result;

pub trait Encodable<Context: Default> {
    fn key(&self) -> &[u8];

    fn encode_full_into<W: Write>(
        &self,
        writer: &mut W,
        state: &mut Context,
        fixed_key_len: Option<u16>,
        fixed_value_len: Option<u32>,
    ) -> Result<()>
    where
        Self: Sized;

    fn encode_truncated_into<W: Write>(
        &self,
        writer: &mut W,
        state: &mut Context,
        shared_len: usize,
        fixed_key_len: Option<u16>,
        fixed_value_len: Option<u32>,
    ) -> Result<()>
    where
        Self: Sized;
}

/// Block encoder
///
/// The block encoder accepts an ascending slice of items, encodes them into
/// restart intervals and builds a binary seek index.
///
/// # Example
///
/// A block with `restart_interval=4`
///
/// ```text
/// [h][t][t][t][h][t][t][t][h][t][t][t][binary index][trailer]
/// ^           ^           ^
/// 0           1           2
///
/// h = restart head
/// t = truncated item
/// ```
///
/// The binary index holds pointers to all restart heads.
/// Because restart heads hold a full key, they can be used to compare to a needle key.
pub struct Encoder<'a, Context: Default, Item: Encodable<Context>> {
    pub writer: &'a mut Vec<u8>,

    pub state: Context,

    pub items: &'a [Item],

    pub restart_interval: u8,
    pub binary_index_builder: BinaryIndexBuilder,

    pub fixed_key_len: Option<u16>,
    pub fixed_value_len: Option<u32>,
}

// TODO: support no binary index -> use in meta blocks with restart interval = 1
// TODO: adjust test + fuzz tests to also test for no binary index
// TODO: https://github.com/fjall-rs/lsm-tree/issues/185

impl<'a, Context: Default, Item: Encodable<Context>> Encoder<'a, Context, Item> {
    pub fn new(
        writer: &'a mut Vec<u8>,
        items: &'a [Item],
        restart_interval: u8, // TODO: should be NonZero
    ) -> Self {
        assert!(!items.is_empty(), "chunk should not be empty");
        let binary_index_builder =
            BinaryIndexBuilder::new(items.len().div_ceil(usize::from(restart_interval)));

        Self {
            writer,

            state: Context::default(),

            items,

            restart_interval,
            binary_index_builder,

            fixed_key_len: None,
            fixed_value_len: None,
        }
    }

    #[must_use]
    pub fn use_fixed_key_len(mut self, len: Option<u16>) -> Self {
        self.fixed_key_len = len;
        self
    }

    #[must_use]
    pub fn use_fixed_value_len(mut self, len: Option<u32>) -> Self {
        self.fixed_value_len = len;
        self
    }

    pub fn finish(mut self) -> Result<()> {
        for chunk in self.items.chunks(usize::from(self.restart_interval)) {
            #[expect(clippy::expect_used, reason = "chunks never yields an empty slice")]
            let (head, tail) = chunk.split_first().expect("restart group is non-empty");
            #[expect(
                clippy::cast_possible_truncation,
                reason = "blocks never approach 4 GiB"
            )]
            self.binary_index_builder.insert(self.writer.len() as u32);
            head.encode_full_into(
                self.writer,
                &mut self.state,
                self.fixed_key_len,
                self.fixed_value_len,
            )?;
            for item in tail {
                let shared_prefix_len = longest_shared_prefix_length(head.key(), item.key());
                item.encode_truncated_into(
                    self.writer,
                    &mut self.state,
                    shared_prefix_len,
                    self.fixed_key_len,
                    self.fixed_value_len,
                )?;
            }
        }
        Trailer::write(self)
    }
}
