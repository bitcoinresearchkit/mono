mod checkpoint;
mod compute;
mod import;

use bitview_plugin::{Plugin, PluginStorage};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::CentsSats;
use vecdb::{BytesVec, Database, MutableVec, Rw, StorageMode};

use crate::{
    STORAGE,
    addr::{AddrStateVecs, AddrVecs},
    live::LiveState,
    metrics::CohortMetrics,
};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    caps: M::WriteOnly<MutableVec<BytesVec<usize, CentsSats>>>,
    #[traversable(wrap = "addrs", rename = "state")]
    pub addr_state: AddrStateVecs<M>,
    pub cohorts: CohortMetrics<M>,
    pub addrs: AddrVecs<M>,
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
