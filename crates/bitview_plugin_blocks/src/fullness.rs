use bitview_compute::DistributionStats;
use bitview_primitives::{PartsPerMillion32, WeightFract};
use bitview_traversable::Traversable;
use bitview_vecs::{
    IndexSources, LazyPercentPerBlock, LazyPercentVec, LazyRollingAvgsFromHeight, RollingWindows,
    shapes::Windows,
};
use brk_types::{Height, Version, Weight, Weight32};
use vecdb::{LazyVec, ReadableCloneableVec, VecValue};

fn block_fullness(_: Height, weight: Weight32) -> PartsPerMillion32 {
    weight_fullness(Height::ZERO, Weight::from(weight))
}

fn weight_fullness(_: Height, weight: Weight) -> PartsPerMillion32 {
    PartsPerMillion32::from(f64::from(weight.fullness()))
}

fn mean_fullness(_: Height, weight: WeightFract) -> PartsPerMillion32 {
    PartsPerMillion32::from(f64::from(weight) / f64::from(Weight::MAX_BLOCK))
}

/// `source`'s values as shares of the block weight limit, at every index.
fn share_of<S: VecValue>(
    name: &str,
    version: Version,
    source: &impl ReadableCloneableVec<Height, S>,
    compute: fn(Height, S) -> PartsPerMillion32,
    indexes: &IndexSources,
) -> LazyPercentPerBlock<PartsPerMillion32> {
    let fixed = LazyVec::new(
        &format!("{name}_ppm_source"),
        version,
        source.read_only_boxed_clone(),
        compute,
    );
    LazyPercentPerBlock::from_height_source(name, version, &fixed, indexes)
}

/// Block weight as a share of the 4,000,000-weight-unit consensus limit, and the weight's rolling
/// statistics as such shares.
#[derive(Clone, Traversable)]
pub struct Fullness {
    /// Block weight as a share of the 4,000,000-weight-unit consensus limit.
    block: LazyPercentVec<PartsPerMillion32, Weight32>,
    /// Mean fullness (block weight as a share of the 4,000,000-weight-unit consensus limit) of
    /// the blocks in a trailing window, each block weighted equally. At time-period indexes,
    /// the value is taken at the period's final block.
    avg: Windows<LazyPercentPerBlock<PartsPerMillion32>>,
    /// Fullness (block weight as a share of the 4,000,000-weight-unit consensus limit) of the
    /// blocks in a trailing window. At time-period indexes, the value is taken at the period's
    /// final block.
    #[traversable(flatten)]
    distribution: DistributionStats<Windows<LazyPercentPerBlock<PartsPerMillion32>>>,
}

impl Fullness {
    /// Fullness from the block weight's per-block values and rolling statistics.
    pub(crate) fn new(
        version: Version,
        weight: &impl ReadableCloneableVec<Height, Weight32>,
        avg: &LazyRollingAvgsFromHeight<Weight>,
        distribution: &DistributionStats<RollingWindows<Weight>>,
        indexes: &IndexSources,
    ) -> Self {
        Self {
            block: LazyPercentVec::from_indexed_source(
                "block_fullness",
                version,
                weight,
                block_fullness,
            ),
            avg: avg.0.map_with_suffix(|suffix, window| {
                share_of(
                    &format!("block_fullness_avg_{suffix}"),
                    version,
                    &window.height,
                    mean_fullness,
                    indexes,
                )
            }),
            distribution: distribution.map_with_suffix(|stat, windows| {
                windows.0.map_with_suffix(|suffix, window| {
                    share_of(
                        &format!("block_fullness_{stat}_{suffix}"),
                        version,
                        &window.height,
                        weight_fullness,
                        indexes,
                    )
                })
            }),
        }
    }
}
