mod by_type;
mod compute;
mod counts;
mod dependencies;
mod has;
mod import;
mod spent;
mod value;

pub use by_type::{OutputTypeVecs, Vecs as TypesVecs};
pub use dependencies::Dependencies;
pub use has::HasOutputs;
pub use spent::published_spend;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{Count, Count32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyPerBlockCumulativeRolling, LazyPerSecondWindows,
    PerBlockFullFromCumulative, ValuePerBlockCumulativeRolling,
};
use brk_types::{Sats, Version};
use statedb::Creations;
use vecdb::{Database, Rw, StorageMode};

use counts::{Coinbase, Pubkeys};
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
    /// spent: the genesis coinbase output, the two earlier coinbase outputs the
    /// BIP30 duplicate coinbases overwrote, and any locking script over 10,000
    /// bytes (none on chain), which Bitcoin Core leaves out of its UTXO set.
    #[traversable(lead = "Outputs other than OP_RETURN's.")]
    non_op_return_count: LazyPerBlockCumulativeRolling<Count>,
    /// The value sent to provably unspendable outputs (`OP_RETURN`, or a locking
    /// script over 10,000 bytes) in each block: the burned value's share of outputs.
    #[traversable(hidden)]
    pub unspendable_value: CachedSeries<Sats, M>,
    /// Transaction outputs by locking-script type, including coinbase outputs
    /// and coinbase transactions.
    types: TypesVecs<M>,
    /// Total bitcoin value assigned to provably unspendable `OP_RETURN`
    /// outputs.
    #[traversable(lead = "Bitcoin burned in OP_RETURN outputs.")]
    pub op_return_value: ValuePerBlockCumulativeRolling<M>,
    /// Outputs of coinbase transactions, in all and by type.
    coinbase: Coinbase<M>,
    pubkeys: Pubkeys<M>,
}
