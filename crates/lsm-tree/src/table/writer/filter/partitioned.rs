// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{fs::File, io::BufWriter};

use log::trace;
use sfa::Writer;

use super::FilterWriter;
use crate::{
    InternalValue, RecordBytes, Result, Slice,
    config::BloomConstructionPolicy,
    table::{
        Block, BlockHandle, BlockOffset, KeyedBlockHandle,
        block::{BlockType, Header as BlockHeader},
        filter::standard_bloom::Builder,
        writer::{META_PARTITION_SIZE, index::write_partitioned},
    },
};

pub struct PartitionedFilterWriter {
    final_filter_buffer: Vec<u8>,

    tli_handles: Vec<KeyedBlockHandle>,

    /// Key hashes for AMQ filter
    bloom_hash_buffer: Vec<u64>,

    bloom_policy: BloomConstructionPolicy,

    /// Buffered key count at which a partition reaches `META_PARTITION_SIZE`.
    partition_key_limit: usize,
}

impl PartitionedFilterWriter {
    pub fn new(bloom_policy: BloomConstructionPolicy) -> Self {
        Self {
            final_filter_buffer: Vec::new(),

            bloom_hash_buffer: Vec::new(),

            tli_handles: Vec::new(),
            bloom_policy,
            partition_key_limit: bloom_policy.partition_key_limit(META_PARTITION_SIZE),
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
}

impl<K: RecordBytes, V: RecordBytes> FilterWriter<K, V> for PartitionedFilterWriter {
    fn register_block(&mut self, items: &[InternalValue<K, V>]) -> Result<()> {
        for item in items {
            let key = item.key.user_key.as_ref();
            self.bloom_hash_buffer.push(Builder::get_hash(key));

            if self.bloom_hash_buffer.len() >= self.partition_key_limit {
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

        write_partitioned(
            file_writer,
            "filter",
            &self.final_filter_buffer,
            "filter_tli",
            &mut self.tli_handles,
        )
    }
}
