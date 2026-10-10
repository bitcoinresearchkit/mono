mod by_type;
mod compute;
mod count;
mod dependencies;
mod has;
mod import;
mod origins;
mod value;

pub use by_type::{InputTypeVecs, Vecs as TypesVecs};
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

    /// Value of the output the input spends, in satoshis; `Sats::MAX` for a coinbase
    /// input, which spends none.
    pub value: M::Stored<PcoVec<TxInIndex, Sats>>,
    pub count: CountVecs<M>,
    /// Transaction-input rate, excluding coinbase inputs.
    per_second: LazyPerSecondWindows,
    /// Transaction inputs by the locking-script type of the output they spend,
    /// excluding coinbase inputs and coinbase transactions.
    types: TypesVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
