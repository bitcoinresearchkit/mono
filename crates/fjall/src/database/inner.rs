use crate::{Keyspace, db_config::Config, locked_file::LockedFileGuard, worker_pool::WorkerPool};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Shared database state.
pub struct Inner {
    /// Background compaction workers.
    pub worker_pool: WorkerPool,
    /// Keyspaces opened or currently opening in this process.
    pub keyspaces: Mutex<HashMap<String, Arc<Mutex<Option<Keyspace>>>>>,
    /// Shared storage configuration.
    pub config: Config,
    /// Root database lock.
    pub lock: LockedFileGuard,
}
