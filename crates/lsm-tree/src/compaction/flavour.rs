// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::{
    InternalValue, Table,
    compaction::{Input as CompactionPayload, worker::Worker},
    table::multi_writer::MultiWriter,
};
use std::time::Instant;

pub struct StandardCompaction {
    start: Instant,
    table_writer: MultiWriter,
    tables_to_rewrite: Vec<Table>,
}

impl StandardCompaction {
    pub fn new(table_writer: MultiWriter, tables_to_rewrite: Vec<Table>) -> Self {
        Self {
            start: Instant::now(),
            table_writer,
            tables_to_rewrite,
        }
    }

    pub fn write(&mut self, item: InternalValue) -> crate::Result<()> {
        self.table_writer.write(item)
    }

    fn consume_writer(self, worker: &Worker, dst_lvl: usize) -> crate::Result<Vec<Table>> {
        let pin_filter = worker.config.filter_block_pinning_policy.get(dst_lvl);
        let pin_index = worker.config.index_block_pinning_policy.get(dst_lvl);
        let (table_base_folder, results) = self.table_writer.finish()?;

        results
            .into_iter()
            .map(|(table_id, checksum)| {
                Table::recover(
                    table_base_folder.join(table_id.to_string()),
                    checksum,
                    0,
                    worker.tree_id,
                    worker.config.cache.clone(),
                    worker.config.descriptor_table.clone(),
                    pin_filter,
                    pin_index,
                )
            })
            .collect()
    }

    pub fn finish(
        mut self,
        worker: &Worker,
        payload: &CompactionPayload,
        dst_lvl: usize,
    ) -> crate::Result<()> {
        log::debug!("Compaction done in {:?}", self.start.elapsed());

        let tables_to_delete = std::mem::take(&mut self.tables_to_rewrite);
        let created_tables = self.consume_writer(worker, dst_lvl)?;

        let table_ids = payload.table_ids.iter().copied().collect::<Vec<_>>();
        worker.versions.publish(&worker.config.path, |current| {
            Ok(current.with_merge(&table_ids, &created_tables, usize::from(payload.dest_level)))
        })?;

        for table in tables_to_delete {
            table.mark_as_deleted();
        }

        Ok(())
    }
}
