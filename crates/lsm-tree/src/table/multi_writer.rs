// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{mem, path::PathBuf};

use log::debug;

use super::{
    id::next_table_id,
    writer::{Writer, WriterOptions},
};
use crate::{RecordBytes, Result, SequenceNumberCounter, Slice, value::InternalValue};

/// Like `Writer` but will rotate to a new table, once a table grows larger than `target_size`
///
/// This results in a sorted "run" of tables
pub struct MultiWriter<K = Slice, V = Slice> {
    base_path: PathBuf,

    options: WriterOptions,

    /// Target size of tables in bytes
    ///
    /// If a table reaches the target size, a new one is started,
    /// resulting in a sorted "run" of tables
    target_size: u64,

    results: Vec<u32>,

    table_id_generator: SequenceNumberCounter,

    writer: Writer<K, V>,
}

impl<K: RecordBytes, V: RecordBytes> MultiWriter<K, V> {
    /// Sets up a new `MultiWriter` at the given tables folder
    pub fn new(
        base_path: PathBuf,
        table_id_generator: SequenceNumberCounter,
        target_size: u64,
        options: WriterOptions,
    ) -> Result<Self> {
        let current_table_id = next_table_id(&table_id_generator);

        let path = base_path.join(current_table_id.to_string());
        let writer = Writer::new(path, current_table_id, options)?;

        Ok(Self {
            base_path,
            options,
            target_size,
            results: Vec::new(),
            table_id_generator,
            writer,
        })
    }

    /// Flushes the current writer, stores its metadata, and sets up a new writer for the next table
    #[cold]
    #[inline(never)]
    fn rotate(&mut self) -> Result<()> {
        debug!("Rotating table writer");

        let new_table_id = next_table_id(&self.table_id_generator);
        let path = self.base_path.join(new_table_id.to_string());
        let new_writer = Writer::new(path, new_table_id, self.options)?;

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
