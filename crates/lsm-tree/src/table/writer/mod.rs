// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    fs::{self, File},
    io::BufWriter,
    path::{self, PathBuf},
};

use index::BlockIndexWriter;
use log::{debug, trace};
use sfa::Writer as SfaWriter;

use super::{
    Block, DataBlock, KeyedBlockHandle,
    block::{BlockType, Header as BlockHeader},
    filter::BloomConstructionPolicy,
};
use crate::{
    CompressionType, InternalValue, RecordBytes, Result, Slice, ValueType,
    coding::Encode,
    table::{
        BlockHandle,
        block::BlockType as BlockBlockType,
        writer::{
            filter::{FilterWriter, FullFilterWriter},
            index::FullIndexWriter,
        },
    },
};

mod filter;
mod index;
mod meta;

const FILE_BUFFER_CAPACITY: usize = 256 * 1_024;

#[derive(Default)]
struct FixedLen {
    len: Option<usize>,
    mixed: bool,
}

impl FixedLen {
    fn observe(&mut self, len: usize) {
        if self.mixed {
            return;
        }

        match self.len {
            None => self.len = Some(len),
            Some(previous) if previous != len => self.mixed = true,
            Some(_) => {}
        }
    }

    fn get(&self) -> Option<usize> {
        (!self.mixed).then_some(self.len).flatten()
    }
}

/// Serializes and compresses values into blocks and writes them to disk as a table
pub struct Writer<K = Slice, V = Slice> {
    /// Table file path
    pub path: PathBuf,

    table_id: u32,

    data_block_restart_interval: u8,

    meta_partition_size: u32,

    data_block_size: u32,

    /// Compression to use for data blocks
    data_block_compression: CompressionType,

    /// Compression to use for data blocks
    index_block_compression: CompressionType,

    /// Buffer to serialize blocks into
    block_buffer: Vec<u8>,

    /// File writer
    #[expect(clippy::struct_field_names)]
    file_writer: SfaWriter<BufWriter<File>>,

    /// Writer of index blocks
    #[expect(clippy::struct_field_names)]
    index_writer: Box<dyn BlockIndexWriter>,

    /// Writer of filter
    #[expect(clippy::struct_field_names)]
    filter_writer: Box<dyn FilterWriter<K, V>>,

    /// Buffer of KVs
    chunk: Vec<InternalValue<K, V>>,
    chunk_size: usize,
    chunk_key_len: FixedLen,
    chunk_value_len: FixedLen,

    pub meta: meta::Metadata,

    bloom_policy: BloomConstructionPolicy,
}

impl<K: RecordBytes, V: RecordBytes> Writer<K, V> {
    pub fn new(path: PathBuf, table_id: u32) -> Result<Self> {
        let writer = BufWriter::with_capacity(FILE_BUFFER_CAPACITY, File::create_new(&path)?);
        let mut writer = SfaWriter::from_writer(writer);
        writer.start("data")?;

        Ok(Self {
            meta: meta::Metadata::default(),

            table_id,

            data_block_restart_interval: 16,

            meta_partition_size: 4_096,

            data_block_size: 4_096,

            data_block_compression: CompressionType::None,
            index_block_compression: CompressionType::None,

            path: path::absolute(path)?,

            index_writer: Box::new(FullIndexWriter::new()),
            filter_writer: Box::new(FullFilterWriter::new(BloomConstructionPolicy::default())),

            block_buffer: Vec::new(),
            file_writer: writer,
            chunk: Vec::new(),

            chunk_size: 0,
            chunk_key_len: FixedLen::default(),
            chunk_value_len: FixedLen::default(),

            bloom_policy: BloomConstructionPolicy::default(),
        })
    }

    #[must_use]
    pub fn use_partitioned_filter(mut self) -> Self {
        self.filter_writer = Box::new(filter::PartitionedFilterWriter::new(self.bloom_policy))
            .use_partition_size(self.meta_partition_size)
            .use_tli_compression(self.index_block_compression);
        self
    }

    #[must_use]
    pub fn use_partitioned_index(mut self) -> Self {
        self.index_writer = Box::new(index::PartitionedIndexWriter::new())
            .use_partition_size(self.meta_partition_size)
            .use_compression(self.index_block_compression);
        self
    }

    #[must_use]
    pub fn use_data_block_restart_interval(mut self, interval: u8) -> Self {
        self.data_block_restart_interval = interval;
        self
    }

    #[must_use]
    pub fn use_data_block_size(mut self, size: u32) -> Self {
        assert!(
            size <= 4 * 1_024 * 1_024,
            "data block size must be <= 4 MiB",
        );
        self.data_block_size = size;
        self
    }

    #[must_use]
    #[cfg(test)]
    pub fn use_meta_partition_size(mut self, size: u32) -> Self {
        assert!(
            size <= 4 * 1_024 * 1_024,
            "data block size must be <= 4 MiB",
        );
        self.meta_partition_size = size;
        self.index_writer = self.index_writer.use_partition_size(size);
        self.filter_writer = self.filter_writer.use_partition_size(size);
        self
    }

    #[must_use]
    pub fn use_data_block_compression(mut self, compression: CompressionType) -> Self {
        self.data_block_compression = compression;
        self
    }

    #[must_use]
    pub fn use_index_block_compression(mut self, compression: CompressionType) -> Self {
        self.index_block_compression = compression;
        self.index_writer = self.index_writer.use_compression(compression);
        self.filter_writer = self.filter_writer.use_tli_compression(compression);
        self
    }

    #[must_use]
    pub fn use_bloom_policy(mut self, bloom_policy: BloomConstructionPolicy) -> Self {
        self.bloom_policy = bloom_policy;
        self.filter_writer = self.filter_writer.set_filter_policy(bloom_policy);
        self
    }

    /// Writes an item.
    ///
    /// # Note
    ///
    /// Items must have strictly increasing user keys.
    pub fn write(&mut self, item: InternalValue<K, V>) -> Result<()> {
        let value_type = item.key.value_type;
        let seqno = item.key.seqno;
        let user_key = item.key.user_key.as_ref();
        let value_len = if value_type.is_tombstone() {
            0
        } else {
            item.value.as_ref().len()
        };

        if self.meta.first_key.is_none() {
            self.meta.first_key = Some(Slice::new(user_key));
        }

        self.chunk_size += user_key.len() + value_len;
        if K::FIXED_LEN.is_none() {
            self.chunk_key_len.observe(user_key.len());
        }
        if V::FIXED_LEN.is_none() && !value_type.is_tombstone() {
            self.chunk_value_len.observe(value_len);
        }
        self.chunk.push(item);
        if self.chunk_size >= self.data_block_size as usize {
            self.spill_block()?;
        }

        self.meta.highest_seqno = self.meta.highest_seqno.max(seqno);

        Ok(())
    }

    /// Writes a compressed block to disk.
    ///
    /// This is triggered when a `Writer::write` causes the buffer to grow to the configured `block_size`.
    ///
    /// Should only be called when the block has items in it.
    pub fn spill_block(&mut self) -> Result<()> {
        let Some(last) = self.chunk.last() else {
            return Ok(());
        };

        if self.bloom_policy.is_active() {
            self.filter_writer.register_block(&self.chunk)?;
        }

        self.block_buffer.clear();

        #[expect(clippy::cast_possible_truncation, reason = "values are u32 length max")]
        let fixed_value_len = V::FIXED_LEN
            .or_else(|| self.chunk_value_len.get())
            .map(|len| len as u32);
        // With compressed empty-value blocks, keeping the repeated key-length byte gives LZ4
        // a useful alignment/pattern and is smaller than omitting both lengths.
        #[expect(clippy::cast_possible_truncation, reason = "keys are u16 length max")]
        let fixed_key_len = K::FIXED_LEN
            .or_else(|| self.chunk_key_len.get())
            .filter(|_| {
                self.data_block_compression == CompressionType::None || fixed_value_len != Some(0)
            })
            .map(|len| len as u16);

        DataBlock::encode_into_with_fixed_lengths(
            &mut self.block_buffer,
            &self.chunk,
            self.data_block_restart_interval,
            fixed_key_len,
            fixed_value_len,
        )?;

        let header = Block::write_into(
            &mut self.file_writer,
            &self.block_buffer,
            BlockType::Data,
            self.data_block_compression,
        )?;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "block header is a couple of bytes only, so cast is fine"
        )]
        let bytes_written = BlockHeader::serialized_len() as u32 + header.data_length;

        let last_key = Slice::new(last.key.user_key.as_ref());
        self.index_writer
            .register_data_block(KeyedBlockHandle::new(
                last_key.clone(),
                last.key.seqno,
                BlockHandle::new(self.meta.file_pos, bytes_written),
            ))?;

        // Adjust metadata
        self.meta.file_pos += u64::from(bytes_written);
        self.meta.item_count += self.chunk.len();
        self.meta.data_block_count += 1;

        // Set last key
        self.meta.last_key = Some(last_key);

        // IMPORTANT: Clear chunk after everything else
        self.chunk.clear();
        self.chunk_size = 0;
        self.chunk_key_len = FixedLen::default();
        self.chunk_value_len = FixedLen::default();

        Ok(())
    }

    // TODO: split meta writing into new function
    /// Finishes the table and flushes all userspace buffers.
    pub fn finish(mut self) -> Result<Option<u32>> {
        self.spill_block()?;

        // No items written! Just delete table file and return nothing
        if self.meta.item_count == 0 {
            fs::remove_file(&self.path)?;
            return Ok(None);
        }

        // Write index
        trace!("Finishing index writer");
        self.index_writer.finish(&mut self.file_writer)?;

        // Write filter
        trace!("Finishing filter writer");
        self.filter_writer.finish(
            &mut self.file_writer,
            #[expect(clippy::expect_used, reason = "the non-empty table has been flushed")]
            self.meta.last_key.as_ref().expect("non-empty table"),
        )?;

        // Write metadata
        self.file_writer.start("meta")?;

        {
            fn meta(key: &str, value: &[u8]) -> InternalValue {
                InternalValue::from_components(key, value, 0, ValueType::Value)
            }

            let meta_items = [
                meta(
                    "block_count#data",
                    &(self.meta.data_block_count as u64).to_le_bytes(),
                ),
                meta(
                    "compression#data",
                    &self.data_block_compression.encode_into_vec(),
                ),
                meta(
                    "compression#index",
                    &self.index_block_compression.encode_into_vec(),
                ),
                meta("file_size", &self.meta.file_pos.to_le_bytes()),
                meta("item_count", &(self.meta.item_count as u64).to_le_bytes()),
                meta(
                    "key#max",
                    // NOTE: At the beginning we check that we have written at least 1 item, so last_key must exist
                    #[expect(clippy::expect_used)]
                    self.meta.last_key.as_ref().expect("should exist"),
                ),
                meta(
                    "key#min",
                    // NOTE: At the beginning we check that we have written at least 1 item, so first_key must exist
                    #[expect(clippy::expect_used)]
                    self.meta.first_key.as_ref().expect("should exist"),
                ),
                meta("seqno#max", &self.meta.highest_seqno.to_le_bytes()),
                meta("table_id", &self.table_id.to_le_bytes()),
            ];

            // NOTE: Just to make sure the items are definitely sorted
            #[cfg(debug_assertions)]
            {
                let is_sorted = meta_items.iter().is_sorted_by_key(|kv| &kv.key);
                assert!(is_sorted, "meta items not sorted correctly");
            }

            self.block_buffer.clear();

            // TODO: disable binary index: https://github.com/fjall-rs/lsm-tree/issues/185
            DataBlock::encode_into(&mut self.block_buffer, &meta_items, 1)?;

            Block::write_into(
                &mut self.file_writer,
                &self.block_buffer,
                BlockBlockType::Meta,
                CompressionType::None,
            )?;
        };

        // Write the fixed-size trailer and flush buffered table bytes. BRK does
        // not require storage-device barriers for crash or power-loss safety.
        drop(self.file_writer.into_inner()?);

        debug!(
            "Written {} items in {} blocks into new table file #{}, written {} MiB",
            self.meta.item_count,
            self.meta.data_block_count,
            self.table_id,
            *self.meta.file_pos / 1_024 / 1_024,
        );

        Ok(Some(self.table_id))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use test_log::test;

    use super::*;

    #[test]
    fn table_writer_count() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("1");
        let mut writer = Writer::new(path, 1)?;
        assert_eq!(0, writer.chunk_size);

        writer.write(InternalValue::from_components(
            b"a",
            b"a",
            0,
            ValueType::Value,
        ))?;
        assert_eq!(2, writer.chunk_size);

        writer.write(InternalValue::from_components(
            b"b",
            b"b",
            0,
            ValueType::Value,
        ))?;
        assert_eq!(4, writer.chunk_size);

        writer.write(InternalValue::from_components(
            b"c",
            b"c",
            0,
            ValueType::Value,
        ))?;
        assert_eq!(6, writer.chunk_size);

        writer.spill_block()?;
        assert_eq!(0, writer.chunk_size);

        Ok(())
    }
}
