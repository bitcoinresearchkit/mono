use crate::{Keyspace, db_config::Config, locked_file::LockedFileGuard, worker_pool::WorkerPool};
use std::{collections::HashMap, sync::Mutex};

/// Shared database state.
pub struct Inner {
    /// Background compaction workers.
    pub worker_pool: WorkerPool,
    /// Keyspaces opened in this process.
    pub keyspaces: Mutex<HashMap<String, Keyspace>>,
    /// Shared storage configuration.
    pub config: Config,
    /// Root database lock.
    pub lock: LockedFileGuard,
}
