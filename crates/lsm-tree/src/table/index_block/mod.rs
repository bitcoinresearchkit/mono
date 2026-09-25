use super::{
    Block,
    block::{BlockOffset, Encoder, Trailer},
};
use crate::{Result, Slice, table::block::Decoder};

// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

mod block_handle;
mod iter;
mod parsed_item;

pub use block_handle::{BlockHandle, KeyedBlockHandle};
pub use iter::Iter;
pub use parsed_item::IndexBlockParsedItem;

/// Block that contains block handles (file offset + size)
#[derive(Clone)]
pub struct IndexBlock {
    pub inner: Block,
}

impl IndexBlock {
    #[must_use]
    pub fn new(inner: Block) -> Self {
        Self { inner }
    }

    /// Accesses the inner raw bytes
    #[must_use]
    pub fn as_slice(&self) -> &Slice {
        &self.inner.data
    }

    /// Returns the number of items in the block.
    #[must_use]
    pub fn len(&self) -> usize {
        Trailer::new(&self.inner).item_count()
    }

    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(Decoder::<KeyedBlockHandle, IndexBlockParsedItem>::new(
            &self.inner,
        ))
    }

    pub fn encode_into_vec(items: &[KeyedBlockHandle]) -> Result<Vec<u8>> {
        let mut buf = vec![];

        Self::encode_into(&mut buf, items)?;

        Ok(buf)
    }

    /// Builds an index block.
    ///
    /// # Panics
    ///
    /// Panics if the given item array if empty.
    pub fn encode_into(writer: &mut Vec<u8>, items: &[KeyedBlockHandle]) -> Result<()> {
        Encoder::<'_, BlockOffset, KeyedBlockHandle>::new(writer, items, 1).finish()
    }
}
