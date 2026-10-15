mod by_type;
mod compute;
mod dependencies;
mod has;
mod import;
mod spent;
mod value;

pub use by_type::{OutputTypeVecs, Vecs as TypesVecs};
pub use dependencies::Dependencies;
pub use has::HasOutputs;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{Count, Count32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPerBlockCumulativeRolling, LazyPerSecondWindows, PerBlockFullFromCumulative,
};
use brk_types::Version;
use statedb::Creations;
use vecdb::{Database, Rw, StorageMode};

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
    /// Number of transaction outputs, including coinbase outputs.
    pub count: PerBlockFullFromCumulative<Count32, M>,
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
