// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{fs::File, io::BufWriter};

use sfa::Writer;

use crate::{
    CompressionType, InternalValue, RecordBytes, Result, Slice, config::BloomConstructionPolicy,
};

mod full;
mod partitioned;

pub use full::FullFilterWriter;
pub use partitioned::PartitionedFilterWriter;

pub trait FilterWriter<K: RecordBytes, V: RecordBytes> {
    /// Registers a data block at once to avoid dispatching for every key.
    fn register_block(&mut self, items: &[InternalValue<K, V>]) -> Result<()>;

    fn finish(
        self: Box<Self>,
        file_writer: &mut Writer<BufWriter<File>>,
        _last_key: &Slice,
    ) -> Result<()>;

    fn set_filter_policy(
        self: Box<Self>,
        policy: BloomConstructionPolicy,
    ) -> Box<dyn FilterWriter<K, V>>;

    fn use_tli_compression(
        self: Box<Self>,
        compression: CompressionType,
    ) -> Box<dyn FilterWriter<K, V>>;

    fn use_partition_size(self: Box<Self>, size: u32) -> Box<dyn FilterWriter<K, V>>;
}
