// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

mod full;
mod partitioned;

pub use full::FullIndexWriter;
pub use partitioned::PartitionedIndexWriter;

use crate::{CompressionType, checksum::ChecksummedWriter, table::index_block::KeyedBlockHandle};
use std::{fs::File, io::BufWriter};

pub trait BlockIndexWriter<W: std::io::Write> {
    fn register_data_block(&mut self, block_handle: KeyedBlockHandle) -> crate::Result<()>;

    fn finish(
        self: Box<Self>,
        file_writer: &mut sfa::Writer<ChecksummedWriter<BufWriter<File>>>,
    ) -> crate::Result<usize>;

    fn use_compression(
        self: Box<Self>,
        compression: CompressionType,
    ) -> Box<dyn BlockIndexWriter<W>>;

    fn use_partition_size(self: Box<Self>, size: u32) -> Box<dyn BlockIndexWriter<W>>;
}
