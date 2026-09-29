mod by_type;
mod correction;
mod count;
mod dependencies;
mod has;
mod spent;
mod value;
pub use correction::overwritten_output;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use statedb::Creations;
use vecdb::{Database, Rw, StorageMode};

use bitview_vecs::LazyPerSecondWindows;

pub use by_type::Vecs as ByTypeVecs;
use count::Vecs as CountVecs;
pub use dependencies::Dependencies;
pub use has::HasOutputs;
use spent::Vecs as SpentVecs;
use value::Vecs as ValueVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("outputs"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    #[traversable(skip)]
    pub creations: M::WriteOnly<Creations>,
    pub spent: SpentVecs<M>,
    pub count: CountVecs<M>,
    /// Transaction-output rate, including coinbase outputs.
    pub per_sec: LazyPerSecondWindows,
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
}
