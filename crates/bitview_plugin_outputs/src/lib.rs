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
    ValuePerBlockCumulativeRolling,
};
use brk_types::Version;
use statedb::Creations;
use vecdb::{Database, Rw, StorageMode};

use spent::Vecs as SpentVecs;

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
    #[traversable(lead = "Outputs: how many were created.")]
    pub count: PerBlockFullFromCumulative<Count32, M>,
    /// Transaction-output rate, including coinbase outputs.
    #[traversable(lead = "Outputs per second.")]
    per_second: LazyPerSecondWindows,
    /// Number of transaction outputs excluding `OP_RETURN` outputs, which are
    /// provably unspendable. It still counts the other outputs that can never be
    /// spent: the genesis coinbase output and the two earlier coinbase outputs the
    /// BIP30 duplicate coinbases overwrote.
    #[traversable(lead = "Outputs other than OP_RETURN's.")]
    spendable_count: LazyPerBlockCumulativeRolling<Count>,
    /// Transaction outputs by locking-script type, including coinbase outputs
    /// and coinbase transactions.
    types: TypesVecs<M>,
    /// Total bitcoin value assigned to provably unspendable `OP_RETURN`
    /// outputs.
    #[traversable(lead = "Bitcoin burned in OP_RETURN outputs.")]
    pub op_return_value: ValuePerBlockCumulativeRolling<M>,
}
