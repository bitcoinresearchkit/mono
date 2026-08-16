pub mod count;
pub mod difficulty;
pub mod halving;
pub mod interval;
pub mod lookback;
pub mod size;
pub mod weight;

mod compute;
mod import;

use brk_plugin::{Plugin, PluginGate};
use brk_traversable::Traversable;
use vecdb::{Database, Rw, StorageMode};

pub use count::Vecs as CountVecs;
pub use difficulty::Vecs as DifficultyVecs;
pub use halving::Vecs as HalvingVecs;
pub use interval::Vecs as IntervalVecs;
pub use lookback::Vecs as LookbackVecs;
pub use size::Vecs as SizeVecs;
pub use weight::Vecs as WeightVecs;

pub const DB_NAME: &str = "blocks";

pub(crate) const TARGET_BLOCKS_PER_DAY_F64: f64 = 144.0;
pub(crate) const TARGET_BLOCKS_PER_DAY_F32: f32 = 144.0;
pub(crate) const TARGET_BLOCKS_PER_DAY: u64 = 144;
pub(crate) const TARGET_BLOCKS_PER_WEEK: u64 = 7 * TARGET_BLOCKS_PER_DAY;
pub(crate) const TARGET_BLOCKS_PER_MONTH: u64 = 30 * TARGET_BLOCKS_PER_DAY;
pub(crate) const TARGET_BLOCKS_PER_YEAR: u64 = 365 * TARGET_BLOCKS_PER_DAY;
pub(crate) const ONE_TERA_HASH: f64 = 1_000_000_000_000.0;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub db: Database,

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
    Self: Send + Sync,
{
    fn id(&self) -> &'static str {
        DB_NAME
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

#[cfg(test)]
mod tests {
    use super::{
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
