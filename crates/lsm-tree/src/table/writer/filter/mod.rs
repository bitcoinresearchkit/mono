// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

mod full;
mod partitioned;

pub use full::FullFilterWriter;
pub use partitioned::PartitionedFilterWriter;

use crate::{CompressionType, Result, Slice, config::BloomConstructionPolicy};
use std::{fs::File, io::BufWriter};

pub trait FilterWriter {
    fn register_key(&mut self, key: &Slice) -> Result<()>;

    fn finish(self: Box<Self>, file_writer: &mut sfa::Writer<BufWriter<File>>) -> Result<usize>;

    fn set_filter_policy(self: Box<Self>, policy: BloomConstructionPolicy)
    -> Box<dyn FilterWriter>;

    fn use_tli_compression(self: Box<Self>, compression: CompressionType) -> Box<dyn FilterWriter>;

    fn use_partition_size(self: Box<Self>, size: u32) -> Box<dyn FilterWriter>;
}
