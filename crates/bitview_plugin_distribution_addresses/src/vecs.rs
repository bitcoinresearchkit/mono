mod checkpoint;
mod compute;
mod import;

use bitview_plugin::{Plugin, PluginStorage};
use bitview_plugin_distribution_common::RealizedCaps;
use bitview_traversable::Traversable;
use brk_error::Result;
use vecdb::{Database, Rw, StorageMode};

use crate::{
    CAP_COUNT, STORAGE,
    addr::{AddrStateVecs, AddrVecs},
    live::LiveState,
    metrics::BalanceMetrics,
};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    caps: M::WriteOnly<RealizedCaps<CAP_COUNT>>,
    #[traversable(wrap = "addrs", rename = "state")]
    pub addr_state: AddrStateVecs<M>,
    #[traversable(wrap = "addrs", rename = "by_balance")]
    pub(crate) balances: Box<BalanceMetrics<M>>,
    pub(crate) addrs: AddrVecs<M>,
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
