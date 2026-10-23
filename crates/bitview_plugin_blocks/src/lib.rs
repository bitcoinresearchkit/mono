mod block_rolling;
mod compute;
mod count;
mod dependencies;
mod difficulty;
mod fullness;
mod halving;
mod has;
mod import;
mod interval;

pub use dependencies::Dependencies;
pub use difficulty::Vecs as DifficultyVecs;
pub use has::HasBlocks;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{Bytes32, Count};
use bitview_traversable::Traversable;
use bitview_vecs::LazyCumulativeRolling;
use brk_types::{Version, Weight32};
use vecdb::{Database, Rw, StorageMode};

use block_rolling::BlockRolling;
use fullness::Fullness;
use halving::Vecs as HalvingVecs;
use interval::Vecs as IntervalVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("blocks"), Version::new(10));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Number of indexed blocks: the cumulative count is height plus one because
    /// genesis is included, and rolling sums count the blocks in each supported
    /// trailing window.
    count: LazyCumulativeRolling<Count>,
    interval: IntervalVecs<M>,
    /// Serialized block size in bytes, including witness data: its 80-byte header,
    /// transaction-count CompactSize, and every serialized transaction.
    pub size: BlockRolling<Bytes32, M>,
    /// BIP-141 block weight in weight units: non-witness bytes count as four
    /// weight units and witness bytes count as one.
    weight: BlockRolling<Weight32, M>,
    fullness: Fullness,
    pub difficulty: DifficultyVecs,
    halving: HalvingVecs,
}
