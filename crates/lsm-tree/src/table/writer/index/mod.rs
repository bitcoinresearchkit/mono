// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
};

use log::trace;
use sfa::Writer;

use crate::{
    Result,
    table::{
        Block, BlockOffset, IndexBlock,
        block::{BlockType, Header as BlockHeader},
        index_block::KeyedBlockHandle,
    },
};

mod full;
mod partitioned;

pub use full::FullIndexWriter;
pub use partitioned::PartitionedIndexWriter;

pub trait BlockIndexWriter {
    fn register_data_block(&mut self, block_handle: KeyedBlockHandle) -> Result<()>;

    fn finish(self: Box<Self>, file_writer: &mut Writer<BufWriter<File>>) -> Result<()>;
}

/// Writes `handles` as one index block into the file `region`.
pub fn write_top_level_index(
    file_writer: &mut Writer<BufWriter<File>>,
    region: &str,
    handles: &[KeyedBlockHandle],
) -> Result<()> {
    debug_assert!(!handles.is_empty(), "top level index should never be empty");
    file_writer.start(region)?;

    let mut bytes = vec![];
    IndexBlock::encode_into(&mut bytes, handles)?;
    let header = Block::write_into(file_writer, &bytes, BlockType::Index)?;

    trace!(
        "Written {region} with {} pointers ({}B)",
        handles.len(),
        BlockHeader::serialized_len() + header.data_length as usize,
    );
    Ok(())
}

/// Writes the concatenated `partitions` into `region`, then the top-level index over them, its
/// `handles` shifted to where the partitions landed, into `tli_region`.
pub fn write_partitioned(
    file_writer: &mut Writer<BufWriter<File>>,
    region: &str,
    partitions: &[u8],
    tli_region: &str,
    handles: &mut [KeyedBlockHandle],
) -> Result<()> {
    let base = BlockOffset(file_writer.get_mut().stream_position()?);
    file_writer.start(region)?;
    file_writer.write_all(partitions)?;
    trace!("Concatted {region} partitions onto blocks file");

    for handle in &mut *handles {
        handle.shift(base);
    }
    write_top_level_index(file_writer, tli_region, handles)
}
