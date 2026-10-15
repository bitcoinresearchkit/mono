// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{fs::File, io::BufWriter};

use log::trace;
use sfa::Writer;

use crate::{
    Result,
    table::{
        index_block::KeyedBlockHandle,
        writer::index::{BlockIndexWriter, write_top_level_index},
    },
};

pub struct FullIndexWriter {
    block_handles: Vec<KeyedBlockHandle>,
}

impl FullIndexWriter {
    pub fn new() -> Self {
        Self {
            block_handles: Vec::new(),
        }
    }
}

impl BlockIndexWriter for FullIndexWriter {
    fn register_data_block(&mut self, block_handle: KeyedBlockHandle) -> Result<()> {
        trace!(
            "Registering block at {:?} with size {} [end_key={:?}]",
            block_handle.offset(),
            block_handle.size(),
            block_handle.end_key(),
        );

        self.block_handles.push(block_handle);
        Ok(())
    }

    fn finish(self: Box<Self>, file_writer: &mut Writer<BufWriter<File>>) -> Result<()> {
        write_top_level_index(file_writer, "tli", &self.block_handles)
    }
}
