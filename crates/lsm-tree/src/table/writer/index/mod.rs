// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{fs::File, io::BufWriter};

use sfa::Writer;

use crate::{CompressionType, Result, table::index_block::KeyedBlockHandle};

mod full;
mod partitioned;

pub use full::FullIndexWriter;
pub use partitioned::PartitionedIndexWriter;

pub trait BlockIndexWriter {
    fn register_data_block(&mut self, block_handle: KeyedBlockHandle) -> Result<()>;

    fn finish(self: Box<Self>, file_writer: &mut Writer<BufWriter<File>>) -> Result<()>;

    fn use_compression(self: Box<Self>, compression: CompressionType) -> Box<dyn BlockIndexWriter>;

    fn use_partition_size(self: Box<Self>, size: u32) -> Box<dyn BlockIndexWriter>;
}
