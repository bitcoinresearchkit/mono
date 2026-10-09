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

pub use count::Vecs as CountVecs;
pub use dependencies::Dependencies;
pub use difficulty::Vecs as DifficultyVecs;
pub use has::HasBlocks;
pub use lookback::Vecs as LookbackVecs;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
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

    pub count: CountVecs,
    /// First block height inside this series' trailing duration, found from the
    /// running maximum of block-header timestamps. A height exactly at the
    /// cutoff is excluded; returns genesis height zero when less history
    /// exists. Duration suffixes are fixed: `h` is 3,600 seconds, `d` is 24
    /// hours, `w` is 7 days, `m` is 30 days, and `y` is 365 days.
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
