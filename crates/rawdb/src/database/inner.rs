use std::{path::PathBuf, sync::Weak};

use parking_lot::RwLock;

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
    pub(crate) writes: RwLock<()>,
    pub(crate) layout: RwLock<Layout>,
    pub(crate) regions: RwLock<Regions>,
    pub(crate) data: DataFile,
    pub(crate) tasks: BackgroundTasks,
}
