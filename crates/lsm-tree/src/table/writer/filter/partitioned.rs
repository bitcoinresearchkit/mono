// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
};

use log::trace;
use sfa::Writer;

use super::FilterWriter;
use crate::{
    CompressionType, InternalValue, RecordBytes, Result, Slice,
    config::BloomConstructionPolicy,
    table::{
        Block, BlockHandle, BlockOffset, IndexBlock, KeyedBlockHandle,
        block::{BlockType, Header as BlockHeader},
        filter::standard_bloom::Builder,
    },
};

pub struct PartitionedFilterWriter {
    final_filter_buffer: Vec<u8>,

    tli_handles: Vec<KeyedBlockHandle>,

    /// Key hashes for AMQ filter
    bloom_hash_buffer: Vec<u64>,

    partition_size: u32,

    bloom_policy: BloomConstructionPolicy,

    compression: CompressionType,
}

impl PartitionedFilterWriter {
    pub fn new(bloom_policy: BloomConstructionPolicy) -> Self {
        Self {
            final_filter_buffer: Vec::new(),

            bloom_hash_buffer: Vec::new(),

            tli_handles: Vec::new(),
            partition_size: 4_096,
            bloom_policy,

            compression: CompressionType::None,
        }
    }

    fn spill_filter_partition(&mut self, key: &[u8]) -> Result<()> {
        let filter_bytes = {
            let mut builder = self.bloom_policy.init(self.bloom_hash_buffer.len());

            for hash in self.bloom_hash_buffer.drain(..) {
                builder.set_with_hash(hash);
            }

            builder.build()
        };

        let offset = self.final_filter_buffer.len() as u64;
        let header = Block::write_into(
            &mut self.final_filter_buffer,
            &filter_bytes,
            BlockType::Filter,
            CompressionType::None,
        )?;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "data length never gets even close to 4 GiB"
        )]
        let bytes_written = (header.data_length as usize + BlockHeader::serialized_len()) as u32;

        self.tli_handles.push(KeyedBlockHandle::new(
            key.into(),
            0,
            BlockHandle::new(BlockOffset(offset), bytes_written),
        ));

        trace!(
            "Built Bloom filter partition ({}B) with end_key={key:?} at +{:#X?}",
            filter_bytes.len(),
            offset,
        );

        Ok(())
    }

    fn write_top_level_index(
        &mut self,
        file_writer: &mut Writer<BufWriter<File>>,
        index_base_offset: BlockOffset,
    ) -> Result<()> {
        file_writer.start("filter_tli")?;

        for item in &mut self.tli_handles {
            item.shift(index_base_offset);
        }

        let mut bytes = vec![];
        IndexBlock::encode_into(&mut bytes, &self.tli_handles)?;

        let header = Block::write_into(file_writer, &bytes, BlockType::Index, self.compression)?;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "blocks never even approach u32 size"
        )]
        let bytes_written = BlockHeader::serialized_len() as u32 + header.data_length;

        debug_assert!(bytes_written > 0, "Top level index should never be empty");

        trace!(
            "Written filter top level index, with {} pointers ({bytes_written} bytes) at {index_base_offset:#X?}",
            self.tli_handles.len(),
        );

        Ok(())
    }
}

impl<K: RecordBytes, V: RecordBytes> FilterWriter<K, V> for PartitionedFilterWriter {
    fn use_partition_size(mut self: Box<Self>, size: u32) -> Box<dyn FilterWriter<K, V>> {
        self.partition_size = size;
        self
    }

    fn use_tli_compression(
        mut self: Box<Self>,
        compression: CompressionType,
    ) -> Box<dyn FilterWriter<K, V>> {
        self.compression = compression;
        self
    }

    fn set_filter_policy(
        mut self: Box<Self>,
        policy: BloomConstructionPolicy,
    ) -> Box<dyn FilterWriter<K, V>> {
        self.bloom_policy = policy;
        self
    }

    fn register_block(&mut self, items: &[InternalValue<K, V>]) -> Result<()> {
        for item in items {
            let key = item.key.user_key.as_ref();
            self.bloom_hash_buffer.push(Builder::get_hash(key));

            let approx_filter_size = self
                .bloom_policy
                .estimated_filter_size(self.bloom_hash_buffer.len());

            if approx_filter_size >= self.partition_size as usize {
                self.spill_filter_partition(key)?;
            }
        }
        Ok(())
    }

    fn finish(
        mut self: Box<Self>,
        file_writer: &mut Writer<BufWriter<File>>,
        last_key: &Slice,
    ) -> Result<()> {
        if !self.bloom_hash_buffer.is_empty() {
            self.spill_filter_partition(last_key)?;
        }

        if self.final_filter_buffer.is_empty() {
            return Ok(());
        }

        let index_base_offset = BlockOffset(file_writer.get_mut().stream_position()?);

        file_writer.start("filter")?;
        file_writer.write_all(&self.final_filter_buffer)?;
        trace!("Concatted filter partitions onto blocks file");

        self.write_top_level_index(file_writer, index_base_offset)?;

        Ok(())
    }
}
