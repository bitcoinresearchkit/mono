#![doc = include_str!("../README.md")]

mod addr;
mod block;
mod checkpoint;
mod compute;
mod dependencies;
mod has;
mod import;
mod metrics;
mod state;

pub use dependencies::Dependencies;
pub use has::HasAddresses;

use bitview_cohort::AmountRangeId;
use bitview_distribution::{RealizedCaps, replay::LiveState};
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use addr::{AddrStateVecs, AddrVecs};
use metrics::BalanceMetrics;
use state::AddrStates;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("addresses"), Version::new(46));
pub const ID: PluginId = STORAGE.id();
const SAVED_CHECKPOINTS: u16 = 10;
const CAP_COUNT: usize = AmountRangeId::ALL.len();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState<AddrStates>>>,
    caps: M::WriteOnly<RealizedCaps<CAP_COUNT>>,
    #[traversable(rename = "state")]
    pub addr_state: AddrStateVecs<M>,
    #[traversable(flatten)]
    balances: Box<BalanceMetrics<M>>,
    #[traversable(flatten)]
    addrs: AddrVecs<M>,
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
    fn flush(&self) -> Result<()> {
        self.db.flush();
        Ok(())
    }
}
