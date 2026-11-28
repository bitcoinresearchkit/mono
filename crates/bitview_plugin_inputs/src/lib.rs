mod by_type;
mod compute;
mod counts;
mod dependencies;
mod has;
mod import;
mod spends;

pub use by_type::{InputTypeVecs, Vecs as TypesVecs};
pub use dependencies::Dependencies;
pub use has::HasInputs;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_plugin_mappings::HeightOfVec;
use bitview_primitives::{Count16, TxInIndex, TxOutIndex};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerSecondWindows, PerBlockFullFromCumulative};
use brk_types::Version;
use statedb::Spends;
use vecdb::{Database, Rw, StorageMode};

use counts::Counts;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("inputs"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Each block's spends grouped by the height of the output they spend, for compute
    /// consumers.
    pub spends: M::WriteOnly<Spends>,

    /// Number of transaction inputs, excluding coinbase inputs.
    #[traversable(lead = "Inputs: how many coins were spent.")]
    pub count: PerBlockFullFromCumulative<Count16, M>,
    /// Transaction-input rate, excluding coinbase inputs.
    #[traversable(lead = "Inputs per second.")]
    per_second: LazyPerSecondWindows,
    /// Transaction inputs by the locking-script type of the output they spend,
    /// excluding coinbase inputs and coinbase transactions.
    types: TypesVecs<M>,
    #[traversable(flatten)]
    counts: Counts<M>,
    /// Height of the block holding the output the input spends: where its coin was born; null for
    /// a coinbase input.
    #[traversable(lead = "The block each input's coin was born in.")]
    spent_txout_height: HeightOfVec<TxInIndex, TxOutIndex>,
}
