mod by_type;
mod count;
mod dependencies;
mod has;

mod compute;
mod import;
mod value;

use bitview_plugin::{Plugin, PluginGate, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use brk_types::{Sats, TxInIndex};
use vecdb::{Database, PcoVec, Rw, StorageMode};

use bitview_compute::LazyPerSecondWindows;

pub use by_type::Vecs as ByTypeVecs;
pub use count::Vecs as CountVecs;
pub use dependencies::Dependencies;
pub use has::HasInputs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("inputs"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    /// Value in satoshis of the indexed transaction output. At `txout_index`,
    /// this is the output's value; at `txin_index`, it is the value of the
    /// previous output spent by the input. Coinbase inputs use `Sats::MAX`
    /// because they have no previous output.
    pub value: M::Stored<PcoVec<TxInIndex, Sats>>,
    pub count: CountVecs<M>,
    /// Transaction-input rate, including one coinbase input per block.
    pub per_sec: LazyPerSecondWindows,
    pub by_type: ByTypeVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
