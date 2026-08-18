pub mod by_type;
pub mod count;

mod compute;
mod import;
mod value;

use bitview_plugin::{Plugin, PluginGate, PluginId};
use brk_traversable::Traversable;
use brk_types::{Sats, TxInIndex};
use vecdb::{Database, PcoVec, Rw, StorageMode};

use crate::internal::LazyPerSecondWindows;

pub use by_type::Vecs as ByTypeVecs;
pub use count::Vecs as CountVecs;

pub const DB_NAME: &str = "inputs";

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub(crate) db: Database,

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
    Self: Send + Sync,
{
    fn id(&self) -> PluginId {
        PluginId::new(DB_NAME)
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
