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

pub use addr::AddrStateRead;
pub use dependencies::Dependencies;
pub use has::HasAddresses;

use bitview_cohort::{AddressType, AmountRangeId};
use bitview_distribution::metrics::{CapitalViews, SupplyViews};
use bitview_distribution::{RealizedCaps, replay::LiveState};
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use addr::{AddrStateVecs, AddressVecs};
use balance::Balances;
use compute::UnpublishedState;
use state::AddrStates;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("addresses"), Version::new(50));
pub const ID: PluginId = STORAGE.id();
const SAVED_CHECKPOINTS: u16 = 10;
const CAP_COUNT: usize = AmountRangeId::ALL.len();

/// Address metrics, members first: every address type together at the root, each type under
/// `types`, each balance band under `balances`. P2PK is one type; P2A, one fixed script, is
/// not an address type (its lookups still work, under `state`).
#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState<AddrStates>>>,
    unpublished_state: M::WriteOnly<Option<UnpublishedState>>,
    caps: M::WriteOnly<RealizedCaps<CAP_COUNT>>,
    #[traversable(flatten)]
    all: Box<AddressVecs<M>>,
    /// All addresses together, the genesis output included, which Bitcoin Core leaves out of
    /// the UTXO set: their share of all unspent outputs can exceed 100% in the earliest blocks.
    supply: SupplyViews,
    /// All addresses together.
    capital: CapitalViews,
    types: Box<AddressType<AddressVecs<M>>>,
    balances: Box<Balances<M>>,
    /// Persistent per-address state: internal slots, read through [`Vecs::read_state`].
    #[traversable(hidden)]
    state: AddrStateVecs<M>,
}

impl<M: StorageMode> Vecs<M> {
    /// The address state of the latest publication, under its lock: take it before pinning the
    /// lengths, and compare [`AddrStateRead::written`] with the pin's height. `None` while a
    /// catch-up's or a rollback's write holds the lock.
    pub fn read_state(&self) -> Option<AddrStateRead<'_, M>> {
        self.state.read()
    }
}
