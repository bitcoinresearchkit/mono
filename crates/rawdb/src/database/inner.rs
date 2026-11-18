use std::{
    path::PathBuf,
    sync::{Weak, atomic::AtomicBool},
};

use brk_exit::Exit;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::{
    background_tasks::BackgroundTasks, data_file::DataFile, layout::Layout, owner::DatabaseOwner,
    regions::Regions,
};

/// Shared storage state. Writers take region access (exclusive, or shared plus the region's
/// writer lock for appends) → mutation barrier → layout → regions → metadata → the data file's
/// growth lock (a leaf). The data file's mapping never moves while it is open.
pub(crate) struct DatabaseInner {
    pub(crate) path: PathBuf,
    pub(crate) foreground: Weak<DatabaseOwner>,
    pub(super) writes: RwLock<()>,
    pub(crate) layout: RwLock<Layout>,
    pub(crate) regions: RwLock<Regions>,
    pub(crate) data: DataFile,
    pub(crate) tasks: BackgroundTasks,
    /// Whether space may await punching: set by every change that frees space (under the
    /// mutation barrier), cleared by compaction (which holds it exclusively).
    pub(crate) reclaim_pending: AtomicBool,
}

impl DatabaseInner {
    /// The mutation barrier, shared: every change to the data, the layout or the metadata holds
    /// it, so this is where debug builds check that a shutdown signal waits for the change.
    pub(crate) fn mutation(&self) -> RwLockReadGuard<'_, ()> {
        debug_assert!(Exit::is_locked(), "rawdb write outside the exit lock");
        self.writes.read()
    }

    /// The mutation barrier, exclusive: removing unused regions and punching holes.
    pub(crate) fn exclusive_mutation(&self) -> RwLockWriteGuard<'_, ()> {
        debug_assert!(Exit::is_locked(), "rawdb compaction outside the exit lock");
        self.writes.write()
    }
}
