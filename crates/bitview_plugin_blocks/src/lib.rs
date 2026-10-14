mod block_rolling;
mod compute;
mod count;
mod dependencies;
mod difficulty;
mod halving;
mod has;
mod import;
mod interval;

pub use dependencies::Dependencies;
pub use difficulty::Vecs as DifficultyVecs;
pub use has::HasBlocks;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{Bytes32, Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyPercentVec};
use brk_types::{Version, Weight32};
use vecdb::{Database, Rw, StorageMode};

use block_rolling::BlockRolling;
use halving::Vecs as HalvingVecs;
use interval::Vecs as IntervalVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("blocks"), Version::new(10));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Number of indexed blocks. The per-block value is one, the cumulative
    /// count is height plus one because genesis is included, and rolling sums
    /// count the blocks in each supported trailing window.
    count: LazyPerBlockCumulativeRolling<Count>,
    interval: IntervalVecs<M>,
    /// Serialized block size in bytes, including witness data: its 80-byte header,
    /// transaction-count CompactSize, and every serialized transaction.
    pub size: BlockRolling<Bytes32, M>,
    /// BIP-141 block weight in weight units: non-witness bytes count as four
    /// weight units and witness bytes count as one.
    weight: BlockRolling<Weight32, M>,
    /// Block weight as a share of the 4,000,000-weight-unit consensus limit.
    /// 100% means the block reached the limit; lower values indicate unused
    /// weight capacity.
    fullness: LazyPercentVec<PartsPerMillion32, Weight32>,
    pub difficulty: DifficultyVecs,
    halving: HalvingVecs,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
