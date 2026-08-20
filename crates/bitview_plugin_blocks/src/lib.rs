mod count;
mod dependencies;
mod difficulty;
mod halving;
mod has;
mod interval;
mod lookback;
mod size;
mod weight;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

pub use count::Vecs as CountVecs;
pub use dependencies::Dependencies;
pub use difficulty::Vecs as DifficultyVecs;
use halving::Vecs as HalvingVecs;
pub use has::HasBlocks;
use interval::Vecs as IntervalVecs;
pub use lookback::Vecs as LookbackVecs;
use size::Vecs as SizeVecs;
use weight::Vecs as WeightVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("blocks"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

pub const ONE_TERA_HASH: f64 = 1_000_000_000_000.0;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    pub count: CountVecs,
    /// First block height inside this series' trailing duration, found from the
    /// running maximum of block-header timestamps. A height exactly at the
    /// cutoff is excluded; returns genesis height zero when less history
    /// exists. Duration suffixes are fixed: `h` is 3,600 seconds, `d` is 24
    /// hours, `w` is 7 days, `m` is 30 days, and `y` is 365 days.
    pub lookback: LookbackVecs,
    pub interval: IntervalVecs<M>,
    #[traversable(flatten)]
    pub size: SizeVecs<M>,
    #[traversable(flatten)]
    pub weight: WeightVecs,
    pub difficulty: DifficultyVecs,
    pub halving: HalvingVecs,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

#[cfg(test)]
mod tests {
    use bitview_compute::{
        TARGET_BLOCKS_PER_DAY, TARGET_BLOCKS_PER_MONTH, TARGET_BLOCKS_PER_WEEK,
        TARGET_BLOCKS_PER_YEAR,
    };

    #[test]
    fn target_block_counts_match_rolling_window_days() {
        assert_eq!(TARGET_BLOCKS_PER_DAY, 144);
        assert_eq!(TARGET_BLOCKS_PER_WEEK, 7 * 144);
        assert_eq!(TARGET_BLOCKS_PER_MONTH, 30 * 144);
        assert_eq!(TARGET_BLOCKS_PER_YEAR, 365 * 144);
    }
}
