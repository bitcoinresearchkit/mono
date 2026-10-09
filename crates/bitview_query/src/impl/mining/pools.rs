use std::{borrow::Cow, cmp::Reverse};

use bitview_primitives::{Count, Day1, Hashrate, Pool, PoolSlug, pools};
use bitview_types::{
    PoolBlockCounts, PoolBlockShares, PoolDetail, PoolDetailInfo, PoolHashrateEntry, PoolInfo,
    PoolStats, PoolsSummary, TimePeriod,
};
use brk_types::Height;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use super::start_height;
use crate::{Error, OptionData, Query, Result};

/// 7-day lookback for share computation.
const LOOKBACK_DAYS: usize = 7;
/// Weekly sample interval (~604800s).
const SAMPLE_WEEKLY: usize = 7;

/// Pre-read shared data for hashrate computation.
struct HashrateSharedData {
    start_day: usize,
    end_day: usize,
    daily_hashrate: Vec<Option<Hashrate>>,
    first_heights: Vec<Height>,
}

impl Query {
    /// Mining-pool leaderboard for `time_period`. For each pool, computes
    /// block count over the window via `cumulative(end) - cumulative(start - 1)`
    /// (tip-cumulative minus pre-window-cumulative), sorts pools by count
    /// descending, assigns ranks, and emits the per-pool share. Also bundles
    /// current / 3d / 1w network hashrate snapshots. Returns zeros early
    /// when no blocks have been indexed. The window start uses the
    /// timestamp-based lookback vecs (`_24h`, `_3d`, ...) rather than
    /// block-count math; `TimePeriod::All` walks from genesis.
    pub fn mining_pools(&self, time_period: TimePeriod) -> Result<PoolsSummary> {
        let _guard = self.read_publication()?;
        let plugins = self.plugins();
        let current_height = self.height();

        if plugins.pools.pool.len() == 0 {
            return Ok(PoolsSummary {
                pools: vec![],
                block_count: 0,
                last_estimated_hashrate: 0,
                last_estimated_hashrate3d: 0,
                last_estimated_hashrate1w: 0,
            });
        }

        let start = start_height(self, time_period)?.to_usize();
        let lookback = &plugins.blocks.lookback;

        let pools = pools();
        let mut pool_data: Vec<(&'static Pool, u64)> = Vec::new();

        // Range count = cumulative(end) - cumulative(start - 1).
        for (pool_id, cumulative) in plugins
            .pools
            .by_slug
            .iter()
            .map(|(id, v)| (id, &v.blocks_mined.cumulative.height))
        {
            let count_at_end: u64 = *cumulative.collect_one(current_height).data()?;

            let count_at_start: u64 = if start == 0 {
                0
            } else {
                *cumulative.collect_one(Height::from(start - 1)).data()?
            };

            let block_count = count_at_end.saturating_sub(count_at_start);

            if block_count > 0 {
                pool_data.push((pools.get(*pool_id), block_count));
            }
        }

        let last_estimated_hashrate = self.hashrate_at(current_height)?;
        let last_estimated_hashrate3d =
            self.hashrate_at(lookback._3d.collect_one(current_height).data()?)?;
        let last_estimated_hashrate1w =
            self.hashrate_at(lookback._1w.collect_one(current_height).data()?)?;
        drop(_guard);

        pool_data.sort_by_key(|p| Reverse(p.1));

        let total_blocks: u64 = pool_data.iter().map(|(_, count)| count).sum();

        let pool_stats: Vec<PoolStats> = pool_data
            .into_iter()
            .enumerate()
            .map(|(idx, (pool, block_count))| {
                let share = if total_blocks > 0 {
                    block_count as f64 / total_blocks as f64
                } else {
                    0.0
                };
                PoolStats::new(pool, block_count, (idx + 1) as u32, share)
            })
            .collect();

        Ok(PoolsSummary {
            pools: pool_stats,
            block_count: total_blocks,
            last_estimated_hashrate,
            last_estimated_hashrate3d,
            last_estimated_hashrate1w,
        })
    }

    /// All supported pools as `PoolInfo`. Static list, no indexer reads, can't fail.
    pub fn all_pools(&self) -> Vec<PoolInfo> {
        pools().iter().map(PoolInfo::from).collect()
    }

    /// Per-pool detail: lifetime block count plus 24h and 1w windowed counts,
    /// each as a share of network blocks in the same window. The 24h share is
    /// also used to weight the current 1-day network hashrate into a per-pool
    /// `estimated_hashrate`. `total_reward` is the coinbase value of every block
    /// the pool mined.
    pub fn pool_detail(&self, slug: PoolSlug) -> Result<PoolDetail> {
        let _guard = self.read_publication()?;
        let plugins = self.plugins();
        let current_height = self.height();
        let end = current_height.to_usize();

        let pools_list = pools();
        let pool = pools_list.get(slug);

        let vecs = plugins.pools.by_slug.get(&slug).ok_or(Error::Internal(
            "pool slug present in static list but missing from pools",
        ))?;
        let cumulative = &vecs.blocks_mined.cumulative.height;

        let total_all: u64 = *cumulative.collect_one(current_height).data()?;

        let window_stats = |start: Height| -> Result<(u64, f64)> {
            let start = start.to_usize();
            let count_before = if start == 0 {
                0
            } else {
                *cumulative.collect_one(Height::from(start - 1)).data()?
            };
            let count = total_all.saturating_sub(count_before);
            let network_blocks = end
                .checked_sub(start)
                .ok_or(Error::Internal("Pool lookback exceeds published tip"))?
                + 1;
            Ok((count, count as f64 / network_blocks as f64))
        };
        let lookback = &plugins.blocks.lookback;
        let (total_24h, share_24h) =
            window_stats(lookback._24h.collect_one(current_height).data()?)?;
        let (total_1w, share_1w) = window_stats(lookback._1w.collect_one(current_height).data()?)?;
        let share_all = total_all as f64 / (end + 1) as f64;

        let network_hr = self.hashrate_at(current_height)?;
        let estimated_hashrate = (share_24h * network_hr as f64) as u128;

        let total_reward = vecs
            .rewards
            .cumulative
            .sats
            .height
            .collect_one(current_height)
            .data()?;

        Ok(PoolDetail {
            pool: PoolDetailInfo::from(pool),
            block_count: PoolBlockCounts {
                all: total_all,
                day: total_24h,
                week: total_1w,
            },
            block_share: PoolBlockShares {
                all: share_all,
                day: share_24h,
                week: share_1w,
            },
            estimated_hashrate,
            reported_hashrate: None,
            total_reward,
        })
    }

    /// Weekly-sampled hashrate series for a single pool over the full chain.
    /// Each point's hashrate is `network_hashrate(day) * pool_share_over_7d`,
    /// where the share is the pool's last-7-days block count divided by the
    /// network's last-7-days block count.
    pub fn pool_hashrate(&self, slug: PoolSlug) -> Result<Vec<PoolHashrateEntry>> {
        let _guard = self.read_publication()?;
        let pool_name = pools().get(slug).name;
        let shared = self.hashrate_shared_data(0)?;
        let pool_cum = self.pool_daily_cumulative(slug, shared.start_day, shared.end_day)?;
        drop(_guard);
        Ok(Self::hashrate_entries(&shared, &pool_cum, pool_name).collect())
    }

    /// Multi-pool weekly-sampled hashrate series over `time_period`. Walks
    /// the full chain when `time_period` is `None` or `Some(TimePeriod::All)`.
    /// For each known pool, emits one entry per weekly sample where the
    /// hashrate is `network_hashrate(day) * pool_share_over_7d`, tagged with
    /// `pool_name`. Entries from all pools are concatenated; the chart layer
    /// groups by pool name.
    pub fn pools_hashrate(
        &self,
        time_period: Option<TimePeriod>,
    ) -> Result<Vec<PoolHashrateEntry>> {
        let _guard = self.read_publication()?;
        let start_height = match time_period {
            Some(tp) => start_height(self, tp)?.to_usize(),
            None => 0,
        };

        let shared = self.hashrate_shared_data(start_height)?;
        let pools_list = pools();
        let mut entries = Vec::new();

        for pool in pools_list.iter() {
            let pool_cum =
                self.pool_daily_cumulative(pool.slug, shared.start_day, shared.end_day)?;
            entries.extend(Self::hashrate_entries(&shared, &pool_cum, pool.name));
        }

        Ok(entries)
    }

    /// Pre-loads the network-wide day1 series (network hashrate, per-day
    /// first heights) over `[start_day, end_day)`, where `start_day` is the
    /// day index of `start_height` and `end_day` is the day index of the
    /// current tip plus one (exclusive). Reused across pools so the network
    /// series is read only once per request.
    fn hashrate_shared_data(&self, start_height: usize) -> Result<HashrateSharedData> {
        let plugins = self.plugins();
        let current_height = self.height();
        let start_day = plugins
            .mappings
            .height
            .day1
            .collect_one_at(start_height)
            .data()?
            .to_usize();
        let end_day = plugins
            .mappings
            .height
            .day1
            .collect_one(current_height)
            .data()?
            .to_usize()
            + 1;
        let daily_hashrate = plugins
            .mining
            .hashrate
            .rate
            .block
            .day1
            .collect_range_at(start_day, end_day);
        let first_heights = plugins
            .mappings
            .day1
            .first_height
            .collect_range_at(start_day, end_day);
        let len = end_day
            .checked_sub(start_day)
            .ok_or(Error::Internal("Reversed pool hashrate window"))?;
        if daily_hashrate.len() != len || first_heights.len() != len {
            return Err(Error::Internal("Incomplete pool hashrate window"));
        }

        Ok(HashrateSharedData {
            start_day,
            end_day,
            daily_hashrate,
            first_heights,
        })
    }

    /// Reads the pool's daily-cumulative blocks-mined vec over the half-open
    /// day range `[start_day, end_day)`. Errors `Internal` if the slug has no
    /// series: both callers guarantee it is in the static pool list, so this
    /// only fires on a list/plugin mismatch, never on user input.
    fn pool_daily_cumulative(
        &self,
        slug: PoolSlug,
        start_day: usize,
        end_day: usize,
    ) -> Result<Vec<Option<Count>>> {
        let plugins = self.plugins();
        let cumulative = &plugins
            .pools
            .by_slug
            .get(&slug)
            .ok_or(Error::Internal(
                "pool slug present in static list but missing from pools",
            ))?
            .blocks_mined
            .cumulative
            .day1;
        let values = cumulative.collect_range_at(start_day, end_day);
        if end_day.checked_sub(start_day) != Some(values.len()) {
            return Err(Error::Internal("Incomplete pool cumulative window"));
        }
        Ok(values)
    }

    /// Per-pool hashrate-share entries from pre-loaded daily cumulative blocks
    /// plus the shared network series. Walks samples from `LOOKBACK_DAYS`
    /// onward in weekly strides; for each sample emits one entry with
    ///   pool_blocks  = pool_cum[i] - pool_cum[i - LOOKBACK_DAYS]
    ///   total_blocks = first_heights[i] - first_heights[i - LOOKBACK_DAYS]
    ///   share        = pool_blocks / total_blocks
    ///   avg_hashrate = daily_hashrate[i] * share
    /// Skips samples where either cumulative value is `None`, where
    /// `pool_blocks == 0`, where `total_blocks == 0`, or where the network
    /// hashrate for that day is unavailable. Source reads require complete
    /// matching windows before this computation.
    fn hashrate_entries<'a>(
        shared: &'a HashrateSharedData,
        pool_cum: &'a [Option<Count>],
        pool_name: &'static str,
    ) -> impl Iterator<Item = PoolHashrateEntry> + 'a {
        let total = pool_cum
            .len()
            .min(shared.first_heights.len())
            .min(shared.daily_hashrate.len());
        (LOOKBACK_DAYS..total)
            .step_by(SAMPLE_WEEKLY)
            .filter_map(move |i| {
                let cum_now = pool_cum[i]?;
                let cum_prev = pool_cum[i - LOOKBACK_DAYS]?;
                let hr = shared.daily_hashrate[i]?;
                let pool_blocks = (*cum_now).saturating_sub(*cum_prev);
                let total_blocks = shared.first_heights[i]
                    .to_usize()
                    .saturating_sub(shared.first_heights[i - LOOKBACK_DAYS].to_usize());
                if pool_blocks == 0 || total_blocks == 0 {
                    return None;
                }

                let share = pool_blocks as f64 / total_blocks as f64;
                Some(PoolHashrateEntry {
                    timestamp: Day1::from(shared.start_day + i).to_timestamp(),
                    avg_hashrate: (f64::from(hr) * share) as u128,
                    share,
                    pool_name: Cow::Borrowed(pool_name),
                })
            })
    }
}
