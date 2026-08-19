use bitview_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{ReadableCloneableVec, UnaryTransform};

use crate::{
    ComputedVecValue, DistributionStats, LazyPerBlock, NumericValue, RollingDistribution, Windows,
};

/// Lazy analog of `RollingDistribution<T>`: `DistributionStats<Windows<LazyPerBlock<T, S1T>>>`.
/// 8 stats × 4 windows = 32 lazy vecs, zero stored.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyRollingDistribution<T, S1T>(pub DistributionStats<Windows<LazyPerBlock<T, S1T>>>)
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
    S1T: ComputedVecValue + JsonSchema;

impl<T, S1T> LazyRollingDistribution<T, S1T>
where
    T: ComputedVecValue + JsonSchema + 'static,
    S1T: NumericValue + JsonSchema,
{
    pub fn from_rolling_distribution<F: UnaryTransform<S1T, T>>(
        name: &str,
        version: Version,
        source: &RollingDistribution<S1T>,
    ) -> Self {
        let s = &source.0;

        macro_rules! map_stat {
            ($field:ident, $suffix:expr) => {{
                let src = &s.$field;
                Windows {
                    _24h: LazyPerBlock::from_computed::<F>(
                        &format!("{name}_{}_24h", $suffix),
                        version,
                        src._24h.height.read_only_boxed_clone(),
                        &src._24h,
                    ),
                    _1w: LazyPerBlock::from_computed::<F>(
                        &format!("{name}_{}_1w", $suffix),
                        version,
                        src._1w.height.read_only_boxed_clone(),
                        &src._1w,
                    ),
                    _1m: LazyPerBlock::from_computed::<F>(
                        &format!("{name}_{}_1m", $suffix),
                        version,
                        src._1m.height.read_only_boxed_clone(),
                        &src._1m,
                    ),
                    _1y: LazyPerBlock::from_computed::<F>(
                        &format!("{name}_{}_1y", $suffix),
                        version,
                        src._1y.height.read_only_boxed_clone(),
                        &src._1y,
                    ),
                }
            }};
        }

        Self(DistributionStats {
            min: map_stat!(min, "min"),
            max: map_stat!(max, "max"),
            pct10: map_stat!(pct10, "pct10"),
            pct25: map_stat!(pct25, "pct25"),
            median: map_stat!(median, "median"),
            pct75: map_stat!(pct75, "pct75"),
            pct90: map_stat!(pct90, "pct90"),
        })
    }
}
