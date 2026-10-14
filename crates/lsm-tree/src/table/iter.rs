// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{marker::PhantomData, path::PathBuf, sync::Arc};

use super::{
    DataBlock, GlobalTableId, Table, bound::Bound, data_block::DataBlock as DataBlockDataBlock,
    owned_data_block_iter::OwnedDataBlockIter,
};
use crate::{
    Cache, CompressionType, InternalValue, RecordBytes, Result, Slice,
    file_accessor::FileAccessor,
    table::{
        BlockHandle, KeyedBlockHandle,
        block::BlockType,
        block_index::{BlockIndexIter, BlockIndexIterImpl},
        util::load_block,
    },
};

fn create_data_block_reader(block: DataBlock) -> OwnedDataBlockIter {
    OwnedDataBlockIter::new(block, DataBlockDataBlock::iter)
}

pub struct Iter<K = Slice, V = Slice> {
    record: PhantomData<(K, V)>,
    _table: Table,
    table_id: GlobalTableId,
    path: Arc<PathBuf>,

    global_seqno: u64,

    #[expect(clippy::struct_field_names)]
    index_iter: BlockIndexIterImpl,

    file_accessor: FileAccessor,
    cache: Arc<Cache>,
    compression: CompressionType,

    index_initialized: bool,
    lo_data_block: Option<OwnedDataBlockIter>,
    hi_data_block: Option<OwnedDataBlockIter>,

    range: (Option<Bound>, Option<Bound>),
}

impl<K: RecordBytes, V: RecordBytes> Iter<K, V> {
    pub fn new(table: Table, index_iter: BlockIndexIterImpl) -> Self {
        let table_id = table.global_id();
        let global_seqno = table.global_seqno();
        let path = table.path.clone();
        let file_accessor = table.file_accessor.clone();
        let cache = table.cache.clone();
        let compression = table.metadata.data_block_compression;

        Self {
            record: PhantomData,
            _table: table,
            table_id,
            path,

            global_seqno,

            index_iter,
            file_accessor,
            cache,
            compression,

            index_initialized: false,
            lo_data_block: None,
            hi_data_block: None,

            range: (None, None),
        }
    }

    pub fn set_lower_bound(&mut self, bound: Bound) {
        self.range.0 = Some(bound);
    }

    pub fn set_upper_bound(&mut self, bound: Bound) {
        self.range.1 = Some(bound);
    }
}

impl<K: RecordBytes, V: RecordBytes> Iter<K, V> {
    /// Narrows the index to the blocks overlapping the bounds, once, before the first read.
    ///
    /// Returns `false` if no block overlaps. Exact bounds are applied inside the boundary
    /// data blocks.
    fn init_index(&mut self) -> bool {
        self.index_initialized = true;
        let ok = self
            .range
            .0
            .as_ref()
            .is_none_or(|bound| self.index_iter.seek_lower(bound.key(), u64::MAX))
            && self
                .range
                .1
                .as_ref()
                .is_none_or(|bound| self.index_iter.seek_upper(bound.key(), u64::MAX));
        if !ok {
            self.lo_data_block = None;
            self.hi_data_block = None;
        }
        ok
    }

    fn read_block(&self, handle: &KeyedBlockHandle) -> Result<OwnedDataBlockIter> {
        let block = load_block(
            self.table_id,
            &self.path,
            &self.file_accessor,
            &self.cache,
            &BlockHandle::new(handle.offset(), handle.size()),
            BlockType::Data,
            self.compression,
        )?;
        Ok(create_data_block_reader(DataBlock::new(block)))
    }

    /// Returns the upper bound if it can cut into the block ending at `handle`'s key.
    fn upper_bound_within(&self, handle: &KeyedBlockHandle) -> Option<&Bound> {
        self.range
            .1
            .as_ref()
            .filter(|bound| handle.end_key() >= bound.key())
    }

    fn rebase(&self, item: Result<InternalValue<K, V>>) -> Result<InternalValue<K, V>> {
        item.map(|mut value| {
            value.key.seqno += self.global_seqno;
            value
        })
    }
}

impl<K: RecordBytes, V: RecordBytes> Iterator for Iter<K, V> {
    type Item = Result<InternalValue<K, V>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Keep iterating inside the already-materialized low data block first.
        if let Some(item) = self
            .lo_data_block
            .as_mut()
            .and_then(OwnedDataBlockIter::next_as)
        {
            return Some(self.rebase(item));
        }

        if !self.index_initialized && !self.init_index() {
            return None;
        }

        loop {
            let Some(handle) = self.index_iter.next() else {
                // Flush any items buffered on the high side by reverse iteration.
                if let Some(item) = self
                    .hi_data_block
                    .as_mut()
                    .and_then(OwnedDataBlockIter::next_as)
                {
                    return Some(self.rebase(item));
                }

                self.lo_data_block = None;
                self.hi_data_block = None;
                return None;
            };
            let handle = fail_iter!(handle);
            let mut reader = fail_iter!(self.read_block(&handle));

            // Index seeks are exact, so only the first block can hold keys below the lower
            // bound, and only a block ending at or past the upper bound can exceed it.
            // Seek the low side first, then clamp the high side.
            if self.lo_data_block.is_none()
                && let Some(bound) = &self.range.0
            {
                reader.seek_lower_bound(bound, u64::MAX);
            }
            if let Some(bound) = self.upper_bound_within(&handle) {
                reader.seek_upper_bound(bound, u64::MAX);
            }

            let item = reader.next_as::<K, V>();

            self.lo_data_block = Some(reader);

            if let Some(item) = item {
                return Some(self.rebase(item));
            }
        }
    }
}

impl<K: RecordBytes, V: RecordBytes> DoubleEndedIterator for Iter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // Mirror the forward iterator: prefer the already-materialized high data block.
        if let Some(item) = self
            .hi_data_block
            .as_mut()
            .and_then(OwnedDataBlockIter::next_back_as)
        {
            return Some(self.rebase(item));
        }

        if !self.index_initialized && !self.init_index() {
            return None;
        }

        loop {
            let Some(handle) = self.index_iter.next_back() else {
                // Flush any items buffered on the low side by forward iteration.
                if let Some(item) = self
                    .lo_data_block
                    .as_mut()
                    .and_then(OwnedDataBlockIter::next_back_as)
                {
                    return Some(self.rebase(item));
                }

                self.lo_data_block = None;
                self.hi_data_block = None;
                return None;
            };
            let handle = fail_iter!(handle);
            let mut reader = fail_iter!(self.read_block(&handle));

            // Clamp the high side first, then the low side. Once forward iteration has read
            // the first block, every block reached from the back lies above the lower bound.
            if let Some(bound) = self.upper_bound_within(&handle) {
                reader.seek_upper_bound(bound, u64::MAX);
            }
            if self.lo_data_block.is_none()
                && let Some(bound) = &self.range.0
            {
                reader.seek_lower_bound(bound, u64::MAX);
            }

            let item = reader.next_back_as::<K, V>();

            self.hi_data_block = Some(reader);

            if let Some(item) = item {
                return Some(self.rebase(item));
            }
        }
    }
}
