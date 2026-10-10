#![doc = include_str!("../README.md")]

mod addr;
mod balance;
mod block;
mod checkpoint;
mod compute;
mod dependencies;
mod has;
mod import;
mod state;

pub use dependencies::Dependencies;
pub use has::HasAddresses;

use bitview_cohort::{AddressType, AmountRange, AmountRangeId};
use bitview_distribution::{RealizedCaps, replay::LiveState};
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use addr::{AddrStateVecs, AddressVecs};
use balance::BalanceVecs;
use state::AddrStates;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("addresses"), Version::new(47));
pub const ID: PluginId = STORAGE.id();
const SAVED_CHECKPOINTS: u16 = 10;
const CAP_COUNT: usize = AmountRangeId::ALL.len();

/// Address metrics, members first: every address type together at the root, each type under
/// `types`, each balance band under `balances`. P2PK is one type; P2A, one fixed script, is
/// not an address type (its lookups still work, under `state`).
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState<AddrStates>>>,
    caps: M::WriteOnly<RealizedCaps<CAP_COUNT>>,
    #[traversable(flatten)]
    all: Box<AddressVecs<M>>,
    types: Box<AddressType<AddressVecs<M>>>,
    balances: Box<AmountRange<BalanceVecs<M>>>,
    pub state: AddrStateVecs<M>,
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
