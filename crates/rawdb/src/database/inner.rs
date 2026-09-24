use std::{path::PathBuf, sync::Weak};

use parking_lot::RwLock;

use super::{
    background_tasks::BackgroundTasks, data_file::DataFile, layout::Layout, owner::DatabaseOwner,
    regions::Regions,
};

/// Shared storage state. Writers take region access → mutation barrier →
/// layout → regions → metadata. Remapping takes the mapping lock, then tries the
/// exclusive barrier. It releases the mapping before waiting for that barrier,
/// allowing batch callbacks to read another region while a remap is pending.
pub(crate) struct DatabaseInner {
    pub(crate) path: PathBuf,
    pub(crate) foreground: Weak<DatabaseOwner>,
    pub(crate) writes: RwLock<()>,
    pub(crate) layout: RwLock<Layout>,
    pub(crate) regions: RwLock<Regions>,
    pub(crate) data: DataFile,
    pub(crate) tasks: BackgroundTasks,
}
