use std::path::PathBuf;

use parking_lot::RwLock;

use super::{
    background_tasks::BackgroundTasks, data_file::DataFile, layout::Layout, regions::Regions,
};

/// Shared storage state. Region access precedes the mutation barrier, then
/// layout → regions → mapping → metadata. Never acquire region access while
/// holding layout or regions, or wait for remapping while holding the barrier.
pub(crate) struct DatabaseInner {
    pub(crate) path: PathBuf,
    pub(crate) writes: RwLock<()>,
    pub(crate) layout: RwLock<Layout>,
    pub(crate) regions: RwLock<Regions>,
    pub(crate) data: DataFile,
    pub(crate) tasks: BackgroundTasks,
}
