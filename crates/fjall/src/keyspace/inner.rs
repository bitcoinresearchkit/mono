use flume::Sender;
use lsm_tree::{Result as TreeResult, Tree};

use crate::{locked_file::LockedFileGuard, worker_pool::WorkerMessage};

/// Shared keyspace state.
pub(super) struct Inner {
    /// Stable keyspace name.
    pub(super) name: String,
    /// Immutable-table LSM tree.
    pub(super) tree: Tree,
    /// Record-specialized compaction, selected once when the keyspace opens.
    pub(super) compaction: fn(&Tree) -> TreeResult<()>,
    /// Background worker sender.
    pub(super) worker: Sender<WorkerMessage>,
    /// Keeps the database lock alive while handles exist.
    pub(super) _lock: LockedFileGuard,
}
