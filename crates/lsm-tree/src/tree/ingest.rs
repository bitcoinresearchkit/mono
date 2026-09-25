use crate::{
    InternalValue, RecordBytes, Result, Slice, Table, Tree, ValueType,
    config::{BloomConstructionPolicy, FilterPolicyEntry},
    file::TABLES_FOLDER,
    key::InternalKey,
    table::multi_writer::MultiWriter,
};

// Use L1 encoding policies, then publish completed tables into L0.
const ENCODING_LEVEL: usize = 1;

/// A strictly sorted direct-to-table ingestion.
pub struct Ingestion<'a, K = Slice, V = Slice> {
    tree: &'a Tree,
    writer: MultiWriter<K, V>,
    #[cfg(debug_assertions)]
    last_key: Option<K>,
}

impl<'a, K: RecordBytes, V: RecordBytes> Ingestion<'a, K, V> {
    /// Starts an ingestion for `tree`.
    ///
    /// # Errors
    ///
    /// Returns an error when the table writer cannot be created.
    pub fn new(tree: &'a Tree) -> Result<Self> {
        let config = &tree.config;
        let mut writer = MultiWriter::new(
            config.path.join(TABLES_FOLDER),
            tree.table_id_counter.clone(),
            64 * 1_024 * 1_024,
        )?
        .use_bloom_policy(
            if let FilterPolicyEntry::Bloom(policy) = config.filter_policy.at_level(ENCODING_LEVEL)
            {
                policy
            } else {
                BloomConstructionPolicy::BitsPerKey(0.0)
            },
        )
        .use_data_block_size(config.data_block_size_policy.at_level(ENCODING_LEVEL))
        .use_data_block_compression(
            config
                .data_block_compression_policy
                .at_level(ENCODING_LEVEL),
        )
        .use_index_block_compression(
            config
                .index_block_compression_policy
                .at_level(ENCODING_LEVEL),
        )
        .use_data_block_restart_interval(
            config
                .data_block_restart_interval_policy
                .at_level(ENCODING_LEVEL),
        );

        if config.index_block_partitioning_policy.get(ENCODING_LEVEL) {
            writer = writer.use_partitioned_index();
        }
        if config.filter_block_partitioning_policy.get(ENCODING_LEVEL) {
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
    ///
    /// # Panics
    /// Panics if the key is empty or exceeds 65,535 bytes, or the value exceeds 4 GiB.
    pub fn write<IK: Into<K>, IV: Into<V>>(&mut self, key: IK, value: IV) -> Result<()> {
        self.write_item(key.into(), value.into(), ValueType::Value)
    }

    /// Appends a weak tombstone. Keys must be strictly increasing.
    ///
    /// # Errors
    /// Returns an error when the table cannot accept the item.
    ///
    /// # Panics
    /// Panics if the key is empty or exceeds 65,535 bytes.
    pub fn write_weak_tombstone<IK: Into<K>>(&mut self, key: IK) -> Result<()> {
        self.write_item(key.into(), V::empty(), ValueType::WeakTombstone)
    }

    fn write_item(&mut self, key: K, value: V, value_type: ValueType) -> Result<()> {
        assert!(!key.as_ref().is_empty(), "key may not be empty");
        assert!(
            u16::try_from(key.as_ref().len()).is_ok(),
            "key exceeds 65,535 bytes"
        );
        assert!(
            u32::try_from(value.as_ref().len()).is_ok(),
            "value exceeds 4 GiB"
        );
        self.validate_key(&key);
        self.writer.write(InternalValue {
            key: InternalKey {
                user_key: key,
                seqno: 0,
                value_type,
            },
            value,
        })
    }

    /// Persists and atomically publishes the ingested tables.
    ///
    /// # Errors
    ///
    /// Returns an error when tables cannot be finalized, recovered, or published.
    pub fn finish(self) -> Result<()> {
        if self.writer.is_empty() {
            return Ok(());
        }

        let (folder, outputs) = self.writer.finish()?;
        let global_seqno = self.tree.seqno.next();
        let tables = outputs
            .into_iter()
            .map(|table_id| {
                Table::recover(
                    folder.join(table_id.to_string()),
                    global_seqno,
                    self.tree.id,
                    self.tree.config.cache.clone(),
                    self.tree.config.descriptor_table.clone(),
                    self.tree.config.filter_block_pinning_policy.at_level(0),
                    self.tree.config.index_block_pinning_policy.at_level(0),
                )
            })
            .collect::<Result<Vec<_>>>()?;

        self.tree
            .versions
            .publish(&self.tree.config.path, |current| {
                Ok(current.with_new_l0_run(&tables))
            })
    }

    #[cfg(debug_assertions)]
    fn validate_key(&mut self, key: &K) {
        if let Some(previous) = &self.last_key {
            debug_assert!(key > previous, "ingestion keys must be strictly increasing");
        }
        self.last_key = Some(key.clone());
    }

    #[cfg(not(debug_assertions))]
    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    fn validate_key(&mut self, _key: &K) {}
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{Config, Result, config::PinningPolicy, version::Level};

    #[test]
    fn ingestion_honors_policies_before_and_after_recovery() -> Result<()> {
        for expect_point_read_hits in [false, true] {
            for (pin_filter, pin_index) in
                [(false, false), (true, false), (false, true), (true, true)]
            {
                let directory = tempdir()?;
                let config = || {
                    Config::new(directory.path())
                        .expect_point_read_hits(expect_point_read_hits)
                        .filter_block_pinning_policy(PinningPolicy::all(pin_filter))
                        .index_block_pinning_policy(PinningPolicy::all(pin_index))
                };
                let mut tree = config().open()?;
                let mut ingestion = tree.ingestion_as::<[u8; 8], [u8; 4]>()?;
                ingestion.write([1; 8], [2; 4])?;
                ingestion.finish()?;

                for recovered in [false, true] {
                    if recovered {
                        drop(tree);
                        tree = config().open()?;
                    }
                    let version = tree.versions.guard();
                    for table in version
                        .iter_levels()
                        .flat_map(Level::iter)
                        .flat_map(|run| run.iter())
                    {
                        assert!(table.filter_size() > 0);
                        assert_eq!(table.pinned_filter_size() > 0, pin_filter);
                        assert_eq!(table.pinned_block_index_size() > 0, pin_index);
                    }
                    assert_eq!(tree.get_as::<[u8; 4]>(&[1; 8])?, Some([2; 4]));
                }
            }
        }
        Ok(())
    }
}
