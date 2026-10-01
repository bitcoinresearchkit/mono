mod checkpoint;
mod compute;
mod import;

use bitview_plugin::{Plugin, PluginStorage};
use bitview_plugin_distribution_common::RealizedCaps;
use bitview_traversable::Traversable;
use brk_error::Result;
use vecdb::{Database, Rw, StorageMode};

use crate::{CAP_COUNT, STORAGE, live::LiveState, metrics::CohortMetrics};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    caps: M::WriteOnly<RealizedCaps<CAP_COUNT>>,
    pub cohorts: CohortMetrics<M>,
}
impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
impl Vecs {
    pub(crate) fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}
