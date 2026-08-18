use crate::{
    SequenceNumberCounter, TableId,
    compaction::state::CompactionState,
    config::Config,
    table::next_table_id,
    version::{Set, Version},
};
use std::sync::{Arc, Mutex, atomic::AtomicU32};

/// Process-local tree identifier used by shared caches.
pub type TreeId = u32;

/// Runtime state of a table-only LSM tree.
pub struct Inner {
    /// Process-local tree identifier used by shared caches.
    pub id: TreeId,
    /// Monotonic table identifier source.
    pub table_id_counter: SequenceNumberCounter,
    /// Monotonic ingestion sequence number source.
    pub seqno: SequenceNumberCounter,
    /// Atomically published table layout.
    pub versions: Arc<Set>,
    /// Tables currently owned by background compactions.
    pub compaction_state: Arc<Mutex<CompactionState>>,
    /// Immutable tree configuration.
    pub config: Arc<Config>,
}

impl Inner {
    /// Creates the initial empty tree state.
    pub fn create_new(config: Config) -> crate::Result<Self> {
        let version = Version::new(0);
        version.persist(&config.path)?;

        Ok(Self {
            id: Self::next_tree_id(),
            table_id_counter: SequenceNumberCounter::default(),
            seqno: SequenceNumberCounter::default(),
            versions: Arc::new(Set::new(version)),
            compaction_state: Arc::new(Mutex::new(CompactionState::default())),
            config: Arc::new(config),
        })
    }

    /// Restores runtime counters from a recovered table version.
    #[must_use]
    pub fn recover(config: Config, version: Version, tree_id: TreeId) -> Self {
        let next_table_id = version
            .iter_tables()
            .map(crate::Table::id)
            .max()
            .map_or(0, |id| u64::from(id) + 1);
        let next_seqno = version
            .iter_tables()
            .map(crate::Table::get_highest_seqno)
            .max()
            .map_or(0, |seqno| seqno + 1);

        Self {
            id: tree_id,
            table_id_counter: SequenceNumberCounter::new(next_table_id),
            seqno: SequenceNumberCounter::new(next_seqno),
            versions: Arc::new(Set::new(version)),
            compaction_state: Arc::new(Mutex::new(CompactionState::default())),
            config: Arc::new(config),
        }
    }

    /// Allocates the next table identifier.
    #[must_use]
    pub fn next_table_id(&self) -> TableId {
        next_table_id(&self.table_id_counter)
    }

    pub fn next_tree_id() -> TreeId {
        static TREE_ID: AtomicU32 = AtomicU32::new(0);

        let id = TREE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        assert_ne!(id, TreeId::MAX, "ran out of tree IDs");
        id
    }
}
