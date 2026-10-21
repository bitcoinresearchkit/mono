use bitview_compute::{
    DistributionStats, Retained, SortedWindow, compute_rolling_distribution_from_starts,
};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_oracle::FIRST_PRICE_HEIGHT;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, ReadableVec, Rw, StorageMode};

use crate::{IndexSources, ValuePerBlock, WindowStarts, shapes::Windows};

/// Stored rolling distribution statistics with statistic-before-window ids, such as
/// `{name}_max_24h`; in dollars, over the window's blocks from the first exchange price (block
/// 74,641) on, none before it.
#[derive(Deref, DerefMut, Traversable)]
pub struct RollingDistributionValuePerBlock<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    stats: DistributionStats<Windows<ValuePerBlock<M>>>,
    /// The kept sats and cents windows of each rolling window, in `Windows` order.
    retained: M::WriteOnly<[[Retained<SortedWindow>; 2]; 4]>,
}

impl RollingDistributionValuePerBlock {
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self {
            stats: DistributionStats::try_from_fn(|stat_suffix| {
                Windows::try_from_fn(|window_suffix| {
                    ValuePerBlock::import(
                        db,
                        &format!("{name}_{stat_suffix}_{window_suffix}"),
                        version,
                        indexes,
                    )
                })
            })?,
            retained: Default::default(),
        })
    }

    pub(crate) fn compute(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        sats_source: &impl ReadableVec<Height, Sats>,
        cents_source: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        let [day_kept, week_kept, month_kept, year_kept] = &mut self.retained;

        macro_rules! compute_window {
            ($w:ident, $starts:expr, $kept:expr) => {{
                let [sats_kept, cents_kept] = $kept;
                macro_rules! compute_unit {
                    ($unit:ident, $first:expr, $source:expr, $retained:expr) => {
                        compute_rolling_distribution_from_starts(
                            max_from,
                            $first,
                            $starts,
                            $source,
                            DistributionStats {
                                min: &mut self.stats.min.$w.$unit.height,
                                max: &mut self.stats.max.$w.$unit.height,
                                p10: &mut self.stats.p10.$w.$unit.height,
                                p25: &mut self.stats.p25.$w.$unit.height,
                                median: &mut self.stats.median.$w.$unit.height,
                                p75: &mut self.stats.p75.$w.$unit.height,
                                p90: &mut self.stats.p90.$w.$unit.height,
                            },
                            $retained,
                            exit,
                        )?
                    };
                }
                compute_unit!(sats, 0, sats_source, sats_kept);
                // (In dollars, over the window's priced blocks: none before the first price.)
                compute_unit!(cents, FIRST_PRICE_HEIGHT, cents_source, cents_kept);
            }};
        }
        compute_window!(_24h, windows._24h, day_kept);
        compute_window!(_1w, windows._1w, week_kept);
        compute_window!(_1m, windows._1m, month_kept);
        compute_window!(_1y, windows._1y, year_kept);

        Ok(())
    }
}
