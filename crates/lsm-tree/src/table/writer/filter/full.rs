// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{fs::File, io::BufWriter, time::Instant};

use log::trace;
use sfa::Writer;

use super::FilterWriter;
use crate::{
    CompressionType, InternalValue, RecordBytes, Result, Slice,
    config::BloomConstructionPolicy,
    table::{Block, block::BlockType, filter::standard_bloom::Builder},
};

pub struct FullFilterWriter {
    /// Key hashes for AMQ filter
    bloom_hash_buffer: Vec<u64>,

    bloom_policy: BloomConstructionPolicy,
}

impl FullFilterWriter {
    pub fn new(bloom_policy: BloomConstructionPolicy) -> Self {
        Self {
            bloom_hash_buffer: Vec::new(),
            bloom_policy,
        }
    }
}

impl<K: RecordBytes, V: RecordBytes> FilterWriter<K, V> for FullFilterWriter {
    fn use_partition_size(self: Box<Self>, _: u32) -> Box<dyn FilterWriter<K, V>> {
        self
    }

    fn use_tli_compression(self: Box<Self>, _: CompressionType) -> Box<dyn FilterWriter<K, V>> {
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
        self.bloom_hash_buffer.extend(
            items
                .iter()
                .map(|item| Builder::get_hash(item.key.user_key.as_ref())),
        );
        Ok(())
    }

    fn finish(
        self: Box<Self>,
        file_writer: &mut Writer<BufWriter<File>>,
        _last_key: &Slice,
    ) -> Result<()> {
        if self.bloom_hash_buffer.is_empty() {
            trace!("Filter writer has no buffered hashes - not building filter");
        } else {
            file_writer.start("filter")?;

            let n = self.bloom_hash_buffer.len();

            trace!(
                "Constructing Bloom filter with {n} entries: {:?}",
                self.bloom_policy,
            );

            let start = Instant::now();

            let filter_bytes = {
                let mut builder = self.bloom_policy.init(n);

                for hash in self.bloom_hash_buffer {
                    builder.set_with_hash(hash);
                }

                builder.build()
            };

            trace!(
                "Built Bloom filter ({}B) in {:?}",
                filter_bytes.len(),
                start.elapsed(),
            );

            Block::write_into(
                file_writer,
                &filter_bytes,
                BlockType::Filter,
                CompressionType::None,
            )?;
        }

        Ok(())
    }
}
