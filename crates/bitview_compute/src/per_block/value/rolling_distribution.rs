use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Exit, ReadableVec, Rw, StorageMode};

use crate::{
    DistributionStats, ValuePerBlock, WindowStarts, Windows,
    algo::compute_rolling_distribution_from_starts,
};

/// Rolling distribution across 4 windows, stat-first naming.
///
/// Tree: `average._24h.sats.height`, `max._24h.sats.height`, etc.
/// Series: `{name}_average_24h`, `{name}_max_24h`, etc.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct RollingDistributionValuePerBlock<M: StorageMode = Rw>(
    pub DistributionStats<Windows<ValuePerBlock<M>>>,
);

impl RollingDistributionValuePerBlock {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        Ok(Self(DistributionStats::try_from_fn(|stat_suffix| {
            Windows::try_from_fn(|window_suffix| {
                ValuePerBlock::forced_import(
                    db,
                    &format!("{name}_{stat_suffix}_{window_suffix}"),
                    version,
                    indexes,
                )
            })
        })?))
    }

    pub fn compute(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        sats_source: &impl ReadableVec<Height, Sats>,
        cents_source: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        let mut sats_cache = None;
        let mut cents_cache = None;

        macro_rules! compute_window {
            ($w:ident, $starts:expr) => {{
                macro_rules! compute_unit {
                    ($unit:ident, $source:expr, $cache:expr) => {
                        compute_rolling_distribution_from_starts(
                            max_from,
                            $starts,
                            $source,
                            &mut self.0.min.$w.$unit.height,
                            &mut self.0.max.$w.$unit.height,
                            &mut self.0.pct10.$w.$unit.height,
                            &mut self.0.pct25.$w.$unit.height,
                            &mut self.0.median.$w.$unit.height,
                            &mut self.0.pct75.$w.$unit.height,
                            &mut self.0.pct90.$w.$unit.height,
                            exit,
                            $cache,
                        )?
                    };
                }
                compute_unit!(sats, sats_source, &mut sats_cache);
                compute_unit!(cents, cents_source, &mut cents_cache);
            }};
        }
        // Largest window first: its cache covers all smaller windows.
        compute_window!(_1y, windows._1y);
        compute_window!(_1m, windows._1m);
        compute_window!(_1w, windows._1w);
        compute_window!(_24h, windows._24h);

        Ok(())
    }
}
