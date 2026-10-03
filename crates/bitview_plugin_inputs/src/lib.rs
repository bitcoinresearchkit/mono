mod by_type;
mod compute;
mod count;
mod dependencies;
mod has;
mod import;
mod origins;
mod value;

pub use by_type::Vecs as ByTypeVecs;
pub use count::Vecs as CountVecs;
pub use dependencies::Dependencies;
pub use has::HasInputs;
pub use origins::OriginSpends;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::TxInIndex;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerSecondWindows;
use brk_types::{Sats, Version};
use vecdb::{Database, PcoVec, Rw, StorageMode};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("inputs"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Actual spends grouped by origin, retained for compute consumers.
    #[traversable(skip)]
    pub origins: M::WriteOnly<OriginSpends>,

    /// Value in satoshis of the indexed transaction output. At `txout_index`,
    /// this is the output's value; at `txin_index`, it is the value of the
    /// previous output spent by the input. Coinbase inputs use `Sats::MAX`
    /// because they have no previous output.
    pub value: M::Stored<PcoVec<TxInIndex, Sats>>,
    pub count: CountVecs<M>,
    /// Transaction-input rate, including one coinbase input per block.
    per_sec: LazyPerSecondWindows,
    pub by_type: ByTypeVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
