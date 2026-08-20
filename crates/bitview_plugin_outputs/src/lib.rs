mod by_type;
mod count;
mod dependencies;
mod has;
mod spent;
mod unspent;
mod value;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use bitview_compute::LazyPerSecondWindows;

pub use by_type::Vecs as ByTypeVecs;
use count::Vecs as CountVecs;
pub use dependencies::Dependencies;
pub use has::HasOutputs;
use spent::Vecs as SpentVecs;
use unspent::Vecs as UnspentVecs;
use value::Vecs as ValueVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("outputs"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    pub spent: SpentVecs<M>,
    pub count: CountVecs<M>,
    /// Transaction-output rate, including coinbase outputs.
    pub per_sec: LazyPerSecondWindows,
    pub unspent: UnspentVecs<M>,
    pub by_type: ByTypeVecs<M>,
    pub value: ValueVecs<M>,
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
