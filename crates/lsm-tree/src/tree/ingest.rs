use crate::{
    InternalValue, RecordBytes, Result, Slice, Table, Tree, ValueType,
    file::TABLES_FOLDER,
    key::InternalKey,
    table::{multi_writer::MultiWriter, writer::WriterOptions},
};

// Use L1 encoding policies, then publish completed tables into L0.
const ENCODING_LEVEL: usize = 1;

/// A strictly sorted direct-to-table ingestion.
///
/// A tree takes one ingestion at a time: concurrent ones could publish out of
/// sequence-number order, and reads would disagree on which value is newest.
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
        let writer = MultiWriter::new(
            config.path.join(TABLES_FOLDER),
            tree.table_id_counter.clone(),
            64 * 1_024 * 1_024,
            WriterOptions::new(
                config,
                ENCODING_LEVEL,
                config.filter_policy.bloom_at(ENCODING_LEVEL),
            ),
        )?;

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
        // Finishing an empty writer removes its file.
        let (folder, outputs) = self.writer.finish()?;
        if outputs.is_empty() {
            return Ok(());
        }

        let global_seqno = self.tree.seqno.next();
        let tables = outputs
            .into_iter()
            .map(|table_id| {
                Table::recover(
                    folder.join(table_id.to_string()),
                    global_seqno,
                    self.tree.id,
                    0,
                    &self.tree.config,
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
