// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
    mem::size_of,
};

use log::trace;
use sfa::Writer;

use crate::{
    CompressionType, Result,
    table::{
        Block, BlockHandle, BlockOffset, IndexBlock,
        block::{BlockType, Header as BlockHeader},
        index_block::KeyedBlockHandle,
        writer::index::BlockIndexWriter,
    },
};

pub struct PartitionedIndexWriter {
    compression: CompressionType,

    tli_handles: Vec<KeyedBlockHandle>,
    data_block_handles: Vec<KeyedBlockHandle>,

    buffer_size: u32,
    partition_size: u32,

    block_buffer: Vec<u8>,

    final_write_buffer: Vec<u8>,
}

impl PartitionedIndexWriter {
    pub fn new() -> Self {
        Self {
            buffer_size: 0,

            partition_size: 4_096,
            compression: CompressionType::None,

            tli_handles: Vec::new(),
            data_block_handles: Vec::new(),
            block_buffer: Vec::with_capacity(4_096),

            final_write_buffer: Vec::new(),
        }
    }

    fn cut_index_block(&mut self) -> Result<()> {
        self.block_buffer.clear();
        IndexBlock::encode_into(&mut self.block_buffer, &self.data_block_handles)?;

        let offset = self.final_write_buffer.len() as u64;
        let header = Block::write_into(
            &mut self.final_write_buffer,
            &self.block_buffer,
            BlockType::Index,
            self.compression,
        )?;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "blocks never even approach size of 4 GiB"
        )]
        let bytes_written = BlockHeader::serialized_len() as u32 + header.data_length;

        // Also, we are allowed to remove the last item
        // to get ownership of it, because the chunk is cleared after
        // this anyway
        #[expect(clippy::expect_used, reason = "chunk is not empty")]
        let last = self
            .data_block_handles
            .pop()
            .expect("Chunk should not be empty");

        let index_block_handle = KeyedBlockHandle::new(
            last.end_key().clone(),
            last.seqno(),
            BlockHandle::new(BlockOffset(offset), bytes_written),
        );

        trace!(
            "Built index partition ({bytes_written}B) with end_key={:?} at +{:#X?}",
            last.end_key(),
            offset,
        );

        self.tli_handles.push(index_block_handle);

        // IMPORTANT: Clear buffer after everything else
        self.data_block_handles.clear();
        self.buffer_size = 0;

        Ok(())
    }

    fn write_top_level_index(
        &mut self,
        file_writer: &mut Writer<BufWriter<File>>,
        index_base_offset: BlockOffset,
    ) -> Result<()> {
        file_writer.start("tli")?;

        for item in &mut self.tli_handles {
            item.shift(index_base_offset);
        }

        let mut bytes = vec![];
        IndexBlock::encode_into(&mut bytes, &self.tli_handles)?;

        let header = Block::write_into(file_writer, &bytes, BlockType::Index, self.compression)?;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "blocks never even approach 4 GiB in size"
        )]
        let bytes_written = BlockHeader::serialized_len() as u32 + header.data_length;

        debug_assert!(bytes_written > 0, "Top level index should never be empty");

        trace!(
            "Written top level index, with {} pointers ({bytes_written} bytes)",
            self.tli_handles.len(),
        );

        Ok(())
    }
}

impl BlockIndexWriter for PartitionedIndexWriter {
    fn use_partition_size(mut self: Box<Self>, size: u32) -> Box<dyn BlockIndexWriter> {
        self.partition_size = size;
        self
    }

    fn use_compression(
        mut self: Box<Self>,
        compression: CompressionType,
    ) -> Box<dyn BlockIndexWriter> {
        self.compression = compression;
        self
    }

    fn register_data_block(&mut self, block_handle: KeyedBlockHandle) -> Result<()> {
        trace!(
            "Registering block at {:?} with size {} [end_key={:?}]",
            block_handle.offset(),
            block_handle.size(),
            block_handle.end_key(),
        );

        #[expect(
            clippy::cast_possible_truncation,
            reason = "key is u16 max, so we can not exceed u32::MAX"
        )]
        let block_handle_size =
            (block_handle.end_key().len() + size_of::<KeyedBlockHandle>()) as u32;

        self.buffer_size += block_handle_size;

        self.data_block_handles.push(block_handle);

        if self.buffer_size >= self.partition_size {
            self.cut_index_block()?;
        }

        Ok(())
    }

    fn finish(mut self: Box<Self>, file_writer: &mut Writer<BufWriter<File>>) -> Result<()> {
        if self.buffer_size > 0 {
            self.cut_index_block()?;
        }

        let index_base_offset = BlockOffset(file_writer.get_mut().stream_position()?);

        file_writer.start("index")?;
        file_writer.write_all(&self.final_write_buffer)?;
        trace!("Concatted index partitions onto blocks file");

        self.write_top_level_index(file_writer, index_base_offset)?;

        Ok(())
    }
}
