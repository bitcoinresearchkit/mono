mod compute;
mod count;
mod dependencies;
mod difficulty;
mod halving;
mod has;
mod import;
mod interval;
mod lookback;
mod size;
mod weight;

pub use dependencies::Dependencies;
pub use difficulty::Vecs as DifficultyVecs;
pub use has::HasBlocks;
pub use lookback::Vecs as LookbackVecs;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlockCumulativeRolling;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use halving::Vecs as HalvingVecs;
use interval::Vecs as IntervalVecs;
use size::Vecs as SizeVecs;
use weight::Vecs as WeightVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("blocks"), Version::new(10));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Number of indexed blocks. The per-block value is one, the cumulative
    /// count is height plus one because genesis is included, and rolling sums
    /// count the blocks in each supported trailing window.
    pub count: LazyPerBlockCumulativeRolling<Count>,
    // First block height inside each trailing duration (from the running maximum of
    // block-header timestamps): window starts for other plugins, not a public series.
    #[traversable(hidden)]
    pub lookback: LookbackVecs,
    interval: IntervalVecs<M>,
    #[traversable(flatten)]
    pub size: SizeVecs<M>,
    #[traversable(flatten)]
    weight: WeightVecs<M>,
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
