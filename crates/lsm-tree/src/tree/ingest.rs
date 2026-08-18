use crate::{
    Table, Tree, UserKey, UserValue, config::FilterPolicyEntry, table::multi_writer::MultiWriter,
};

const INGESTION_LEVEL: usize = 1;

/// A strictly sorted direct-to-table ingestion.
pub struct Ingestion<'a> {
    tree: &'a Tree,
    writer: MultiWriter,
    #[cfg(debug_assertions)]
    last_key: Option<UserKey>,
}

impl<'a> Ingestion<'a> {
    /// Starts an ingestion for `tree`.
    ///
    /// # Errors
    ///
    /// Returns an error when the table writer cannot be created.
    pub fn new(tree: &'a Tree) -> crate::Result<Self> {
        let config = &tree.config;
        let mut writer = MultiWriter::new(
            config.path.join(crate::file::TABLES_FOLDER),
            tree.table_id_counter.clone(),
            64 * 1_024 * 1_024,
        )?
        .use_bloom_policy(if config.expect_point_read_hits {
            crate::config::BloomConstructionPolicy::BitsPerKey(0.0)
        } else if let FilterPolicyEntry::Bloom(policy) = config.filter_policy.get(INGESTION_LEVEL) {
            policy
        } else {
            crate::config::BloomConstructionPolicy::BitsPerKey(0.0)
        })
        .use_data_block_size(config.data_block_size_policy.get(INGESTION_LEVEL))
        .use_data_block_hash_ratio(config.data_block_hash_ratio_policy.get(INGESTION_LEVEL))
        .use_data_block_compression(config.data_block_compression_policy.get(INGESTION_LEVEL))
        .use_index_block_compression(config.index_block_compression_policy.get(INGESTION_LEVEL))
        .use_data_block_restart_interval(
            config
                .data_block_restart_interval_policy
                .get(INGESTION_LEVEL),
        )
        .use_index_block_restart_interval(
            config
                .index_block_restart_interval_policy
                .get(INGESTION_LEVEL),
        );

        if config.index_block_partitioning_policy.get(INGESTION_LEVEL) {
            writer = writer.use_partitioned_index();
        }
        if config.filter_block_partitioning_policy.get(INGESTION_LEVEL) {
            writer = writer.use_partitioned_filter();
        }

        Ok(Self {
            tree,
            writer,
            #[cfg(debug_assertions)]
            last_key: None,
        })
    }

    /// Appends a key-value pair. Keys must be strictly increasing.
    ///
    /// # Errors
    ///
    /// Returns an error when the table cannot accept the item.
    pub fn write<K: Into<UserKey>, V: Into<UserValue>>(
        &mut self,
        key: K,
        value: V,
    ) -> crate::Result<()> {
        let key = key.into();
        self.validate_key(&key);
        self.writer.write(crate::InternalValue::from_components(
            key,
            value,
            0,
            crate::ValueType::Value,
        ))
    }

    /// Appends a weak tombstone. Keys must be strictly increasing.
    ///
    /// # Errors
    ///
    /// Returns an error when the table cannot accept the item.
    pub fn write_weak_tombstone<K: Into<UserKey>>(&mut self, key: K) -> crate::Result<()> {
        let key = key.into();
        self.validate_key(&key);
        self.writer.write(crate::InternalValue::from_components(
            key,
            crate::UserValue::empty(),
            0,
            crate::ValueType::WeakTombstone,
        ))
    }

    /// Persists and atomically publishes the ingested tables.
    ///
    /// # Errors
    ///
    /// Returns an error when tables cannot be finalized, recovered, or published.
    pub fn finish(self) -> crate::Result<()> {
        if self.writer.is_empty() {
            return Ok(());
        }

        let (folder, outputs) = self.writer.finish()?;
        let global_seqno = self.tree.seqno.next();
        let tables = outputs
            .into_iter()
            .map(|(table_id, checksum)| {
                Table::recover(
                    folder.join(table_id.to_string()),
                    checksum,
                    global_seqno,
                    self.tree.id,
                    self.tree.config.cache.clone(),
                    self.tree.config.descriptor_table.clone(),
                    false,
                    false,
                )
            })
            .collect::<crate::Result<Vec<_>>>()?;

        self.tree
            .versions
            .publish(&self.tree.config.path, |current| {
                Ok(current.with_new_l0_run(&tables))
            })
    }

    #[cfg(debug_assertions)]
    fn validate_key(&mut self, key: &UserKey) {
        if let Some(previous) = &self.last_key {
            debug_assert!(key > previous, "ingestion keys must be strictly increasing");
        }
        self.last_key = Some(key.clone());
    }

    #[cfg(not(debug_assertions))]
    fn validate_key(&mut self, _key: &UserKey) {}
}
