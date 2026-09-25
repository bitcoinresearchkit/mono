// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{mem, path::PathBuf};

use log::debug;

use super::{filter::BloomConstructionPolicy, id::next_table_id, writer::Writer};
use crate::{
    CompressionType, RecordBytes, Result, SequenceNumberCounter, Slice, value::InternalValue,
};

/// Like `Writer` but will rotate to a new table, once a table grows larger than `target_size`
///
/// This results in a sorted "run" of tables
pub struct MultiWriter<K = Slice, V = Slice> {
    pub base_path: PathBuf,

    data_block_size: u32,

    data_block_restart_interval: u8,

    use_partitioned_index: bool,
    use_partitioned_filter: bool,

    /// Target size of tables in bytes
    ///
    /// If a table reaches the target size, a new one is started,
    /// resulting in a sorted "run" of tables
    pub target_size: u64,

    results: Vec<u32>,

    table_id_generator: SequenceNumberCounter,

    pub writer: Writer<K, V>,

    pub data_block_compression: CompressionType,
    pub index_block_compression: CompressionType,

    bloom_policy: BloomConstructionPolicy,
}

impl<K: RecordBytes, V: RecordBytes> MultiWriter<K, V> {
    /// Sets up a new `MultiWriter` at the given tables folder
    pub fn new(
        base_path: PathBuf,
        table_id_generator: SequenceNumberCounter,
        target_size: u64,
    ) -> Result<Self> {
        let current_table_id = next_table_id(&table_id_generator);

        let path = base_path.join(current_table_id.to_string());
        let writer = Writer::new(path, current_table_id)?;

        Ok(Self {
            base_path,

            data_block_size: 4_096,

            data_block_restart_interval: 16,

            target_size,
            results: Vec::new(),
            table_id_generator,
            writer,

            data_block_compression: CompressionType::None,
            index_block_compression: CompressionType::None,

            use_partitioned_index: false,
            use_partitioned_filter: false,

            bloom_policy: BloomConstructionPolicy::default(),
        })
    }

    #[must_use]
    pub fn use_partitioned_index(mut self) -> Self {
        self.use_partitioned_index = true;
        self.writer = self.writer.use_partitioned_index();
        self
    }

    #[must_use]
    pub fn use_partitioned_filter(mut self) -> Self {
        self.use_partitioned_filter = true;
        self.writer = self.writer.use_partitioned_filter();
        self
    }

    #[must_use]
    pub fn use_data_block_restart_interval(mut self, interval: u8) -> Self {
        self.data_block_restart_interval = interval;
        self.writer = self.writer.use_data_block_restart_interval(interval);
        self
    }

    #[must_use]
    pub fn use_data_block_size(mut self, size: u32) -> Self {
        assert!(
            size <= 4 * 1_024 * 1_024,
            "data block size must be <= 4 MiB",
        );
        self.data_block_size = size;
        self.writer = self.writer.use_data_block_size(size);
        self
    }

    #[must_use]
    pub fn use_data_block_compression(mut self, compression: CompressionType) -> Self {
        self.data_block_compression = compression;
        self.writer = self.writer.use_data_block_compression(compression);
        self
    }

    #[must_use]
    pub fn use_index_block_compression(mut self, compression: CompressionType) -> Self {
        self.index_block_compression = compression;
        self.writer = self.writer.use_index_block_compression(compression);
        self
    }

    #[must_use]
    pub fn use_bloom_policy(mut self, bloom_policy: BloomConstructionPolicy) -> Self {
        self.bloom_policy = bloom_policy;
        self.writer = self.writer.use_bloom_policy(bloom_policy);
        self
    }

    /// Flushes the current writer, stores its metadata, and sets up a new writer for the next table
    #[cold]
    #[inline(never)]
    fn rotate(&mut self) -> Result<()> {
        debug!("Rotating table writer");

        let new_table_id = next_table_id(&self.table_id_generator);
        let path = self.base_path.join(new_table_id.to_string());

        let mut new_writer = Writer::new(path, new_table_id)?
            .use_data_block_compression(self.data_block_compression)
            .use_index_block_compression(self.index_block_compression)
            .use_data_block_size(self.data_block_size)
            .use_data_block_restart_interval(self.data_block_restart_interval)
            .use_bloom_policy(self.bloom_policy);

        if self.use_partitioned_index {
            new_writer = new_writer.use_partitioned_index();
        }
        if self.use_partitioned_filter {
            new_writer = new_writer.use_partitioned_filter();
        }

        let old_writer = mem::replace(&mut self.writer, new_writer);

        if let Some(table_id) = old_writer.finish()? {
            self.results.push(table_id);
        }

        Ok(())
    }

    /// Writes an item with a user key greater than the previous item.
    #[inline]
    pub fn write(&mut self, item: InternalValue<K, V>) -> Result<()> {
        if *self.writer.meta.file_pos >= self.target_size {
            self.rotate()?;
        }

        self.writer.write(item)?;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.results.is_empty() && self.writer.meta.first_key.is_none()
    }

    /// Finishes the last table, making sure all data is written durably
    ///
    /// Returns the metadata of created tables
    pub fn finish(mut self) -> Result<(PathBuf, Vec<u32>)> {
        if let Some(table_id) = self.writer.finish()? {
            self.results.push(table_id);
        }

        Ok((self.base_path, self.results))
    }
}
