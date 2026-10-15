mod by_type;
mod compute;
mod count;
mod dependencies;
mod has;
mod import;
mod spent;
mod value;

pub use by_type::{OutputTypeVecs, Vecs as TypesVecs};
pub use dependencies::Dependencies;
pub use has::HasOutputs;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyPerSecondWindows};
use brk_types::Version;
use statedb::Creations;
use vecdb::{Database, Rw, StorageMode};

use count::Vecs as CountVecs;
use spent::Vecs as SpentVecs;
use value::Vecs as ValueVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("outputs"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    pub creations: M::WriteOnly<Creations>,
    #[traversable(flatten)]
    pub spent: SpentVecs<M>,
    pub count: CountVecs<M>,
    /// Transaction-output rate, including coinbase outputs.
    per_second: LazyPerSecondWindows,
    /// Number of transaction outputs excluding `OP_RETURN` outputs, which are
    /// provably unspendable.
    spendable_count: LazyPerBlockCumulativeRolling<Count>,
    /// Transaction outputs by locking-script type, including coinbase outputs
    /// and coinbase transactions.
    types: TypesVecs<M>,
    #[traversable(flatten)]
    pub value: ValueVecs<M>,
}
