use std::{borrow::Cow, cmp::Reverse};

use brk_error::{Error, OptionData};
use brk_types::{
    BlockInfoV1, Day1, Height, Pool, PoolBlockCounts, PoolBlockShares, PoolDetail, PoolDetailInfo,
    PoolHashrateEntry, PoolInfo, PoolSlug, PoolStats, PoolsSummary, StoredF64, StoredU64,
    TimePeriod, pools,
};
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::Query;

/// 7-day lookback for share computation.
const LOOKBACK_DAYS: usize = 7;
/// Weekly sample interval (~604800s).
const SAMPLE_WEEKLY: usize = 7;

/// Pre-read shared data for hashrate computation.
struct HashrateSharedData {
    start_day: usize,
    end_day: usize,
    daily_hashrate: Vec<Option<StoredF64>>,
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
    pub fn mining_pools(&self, time_period: TimePeriod) -> brk_error::Result<PoolsSummary> {
        let computer = self.computer();
        let current_height = self.height();

        if computer.pools.pool.len() == 0 {
            return Ok(PoolsSummary {
                pools: vec![],
                block_count: 0,
                last_estimated_hashrate: 0,
                last_estimated_hashrate3d: 0,
                last_estimated_hashrate1w: 0,
            });
        }

        let start = super::start_height(self, time_period)?.to_usize();
        let lookback = &computer.blocks.lookback;

        let pools = pools();
        let mut pool_data: Vec<(&'static Pool, u64)> = Vec::new();

        // Range count = cumulative(end) - cumulative(start - 1).
        for (pool_id, cumulative) in computer
            .pools
            .major
            .iter()
            .map(|(id, v)| (id, &v.blocks_mined.cumulative.height))
            .chain(
                computer
                    .pools
                    .minor
                    .iter()
                    .map(|(id, v)| (id, &v.blocks_mined.cumulative.height)),
            )
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

        let last_estimated_hashrate = super::hashrate_at(self, current_height)?;
        let last_estimated_hashrate3d =
            super::hashrate_at(self, lookback._3d.collect_one(current_height).data()?)?;
        let last_estimated_hashrate1w =
            super::hashrate_at(self, lookback._1w.collect_one(current_height).data()?)?;

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
    /// `estimated_hashrate`. `total_reward` is `Some` only for major pools
    /// (minor pools don't track per-pool reward sums); under stamp lag on a
    /// major pool's reward vec this errors rather than silently reporting
    /// `None`.
    pub fn pool_detail(&self, slug: PoolSlug) -> brk_error::Result<PoolDetail> {
        let computer = self.computer();
        let current_height = self.height();
        let end = current_height.to_usize();

        let pools_list = pools();
        let pool = pools_list.get(slug);

        let cumulative = computer
            .pools
            .major
            .get(&slug)
            .map(|v| &v.blocks_mined.cumulative.height)
            .or_else(|| {
                computer
                    .pools
                    .minor
                    .get(&slug)
                    .map(|v| &v.blocks_mined.cumulative.height)
            })
            .ok_or_else(|| {
                Error::Internal(
                    "pool slug present in static list but missing from major/minor maps",
                )
            })?;

        let total_all: u64 = *cumulative.collect_one(current_height).data()?;

        let lookback = &computer.blocks.lookback;
        let start_24h = lookback._24h.collect_one(current_height).data()?.to_usize();
        let count_before_24h: u64 = if start_24h == 0 {
            0
        } else {
            *cumulative.collect_one(Height::from(start_24h - 1)).data()?
        };
        let total_24h = total_all.saturating_sub(count_before_24h);

        let start_1w = lookback._1w.collect_one(current_height).data()?.to_usize();
        let count_before_1w: u64 = if start_1w == 0 {
            0
        } else {
            *cumulative.collect_one(Height::from(start_1w - 1)).data()?
        };
        let total_1w = total_all.saturating_sub(count_before_1w);

        let network_blocks_all = (end + 1) as u64;
        let network_blocks_24h = (end - start_24h + 1) as u64;
        let network_blocks_1w = (end - start_1w + 1) as u64;

        let share_all = if network_blocks_all > 0 {
            total_all as f64 / network_blocks_all as f64
        } else {
            0.0
        };
        let share_24h = if network_blocks_24h > 0 {
            total_24h as f64 / network_blocks_24h as f64
        } else {
            0.0
        };
        let share_1w = if network_blocks_1w > 0 {
            total_1w as f64 / network_blocks_1w as f64
        } else {
            0.0
        };

        let network_hr = super::hashrate_at(self, current_height)?;
        let estimated_hashrate = (share_24h * network_hr as f64) as u128;

        let total_reward = if let Some(major) = computer.pools.major.get(&slug) {
            Some(
                major
                    .rewards
                    .cumulative
                    .sats
                    .height
                    .collect_one(current_height)
                    .data()?,
            )
        } else {
            None
        };

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

    /// Page of blocks mined by `slug`, in descending height order, capped at
    /// `limit`. `before_height` is the inclusive upper bound to paginate from
    /// (defaults to tip). Returns an empty `Vec` if the pool has no recorded
    /// blocks. Heights come from a sorted-ascending per-pool index, so the
    /// page is computed via `partition_point` then reversed; consecutive
    /// runs are merged into a single bulk read of `blocks_v1_range`.
    pub fn pool_blocks(
        &self,
        slug: PoolSlug,
        before_height: Option<Height>,
        limit: usize,
    ) -> brk_error::Result<Vec<BlockInfoV1>> {
        let computer = self.computer();
        let tip = self.height().to_usize();
        let upper = before_height.map(|h| h.to_usize()).unwrap_or(tip);
        let end = upper.min(tip);

        let heights: Vec<usize> = computer
            .pools
            .pool_heights
            .latest_heights(slug, Height::from(end), limit)
            .into_iter()
            .map(|height| height.to_usize())
            .collect();

        // Group consecutive descending heights into ranges for batch reads.
        let mut blocks = Vec::with_capacity(heights.len());
        let mut i = 0;
        while i < heights.len() {
            let hi = heights[i];
            while i + 1 < heights.len() && heights[i + 1] + 1 == heights[i] {
                i += 1;
            }
            let mut v = crate::r#impl::block::blocks_v1_range(self, heights[i], hi + 1)?;
            blocks.append(&mut v);
            i += 1;
        }

        Ok(blocks)
    }

    /// Weekly-sampled hashrate series for a single pool over the full chain.
    /// Each point's hashrate is `network_hashrate(day) * pool_share_over_7d`,
    /// where the share is the pool's last-7-days block count divided by the
    /// network's last-7-days block count.
    pub fn pool_hashrate(&self, slug: PoolSlug) -> brk_error::Result<Vec<PoolHashrateEntry>> {
        let pool_name = pools().get(slug).name;
        let shared = self.hashrate_shared_data(0)?;
        let pool_cum = self.pool_daily_cumulative(slug, shared.start_day, shared.end_day)?;
        Ok(Self::compute_hashrate_entries(
            &shared,
            &pool_cum,
            pool_name,
            SAMPLE_WEEKLY,
        ))
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
    ) -> brk_error::Result<Vec<PoolHashrateEntry>> {
        let start_height = match time_period {
            Some(tp) => super::start_height(self, tp)?.to_usize(),
            None => 0,
        };

        let shared = self.hashrate_shared_data(start_height)?;
        let pools_list = pools();
        let mut entries = Vec::new();

        for pool in pools_list.iter() {
            let pool_cum =
                self.pool_daily_cumulative(pool.slug, shared.start_day, shared.end_day)?;
            entries.extend(Self::compute_hashrate_entries(
                &shared,
                &pool_cum,
                pool.name,
                SAMPLE_WEEKLY,
            ));
        }

        Ok(entries)
    }

    /// Pre-loads the network-wide day1 series (network hashrate, per-day
    /// first heights) over `[start_day, end_day)`, where `start_day` is the
    /// day index of `start_height` and `end_day` is the day index of the
    /// current tip plus one (exclusive). Reused across pools so the network
    /// series is read only once per request.
    fn hashrate_shared_data(&self, start_height: usize) -> brk_error::Result<HashrateSharedData> {
        let computer = self.computer();
        let current_height = self.height();
        let start_day = computer
            .indexes
            .height
            .day1
            .collect_one_at(start_height)
            .data()?
            .to_usize();
        let end_day = computer
            .indexes
            .height
            .day1
            .collect_one(current_height)
            .data()?
            .to_usize()
            + 1;
        let daily_hashrate = computer
            .mining
            .hashrate
            .rate
            .base
            .day1
            .collect_range_at(start_day, end_day);
        let first_heights = computer
            .indexes
            .day1
            .first_height
            .collect_range_at(start_day, end_day);

        Ok(HashrateSharedData {
            start_day,
            end_day,
            daily_hashrate,
            first_heights,
        })
    }

    /// Reads the pool's daily-cumulative blocks-mined vec over the half-open
    /// day range `[start_day, end_day)`. Major pools nest under `.base`
    /// (additional derived computations), minor pools don't, so the slug is
    /// looked up in both maps. Errors `Internal` if the slug is in neither
    /// map: this can only fire on a static-pool-list / indexer-map mismatch
    /// since both callers guarantee the slug is in the static list, so the
    /// route layer never reaches a user-driven not-found path here.
    fn pool_daily_cumulative(
        &self,
        slug: PoolSlug,
        start_day: usize,
        end_day: usize,
    ) -> brk_error::Result<Vec<Option<StoredU64>>> {
        let computer = self.computer();
        computer
            .pools
            .major
            .get(&slug)
            .map(|v| {
                v.base
                    .blocks_mined
                    .cumulative
                    .day1
                    .collect_range_at(start_day, end_day)
            })
            .or_else(|| {
                computer.pools.minor.get(&slug).map(|v| {
                    v.blocks_mined
                        .cumulative
                        .day1
                        .collect_range_at(start_day, end_day)
                })
            })
            .ok_or_else(|| {
                Error::Internal(
                    "pool slug present in static list but missing from major/minor maps",
                )
            })
    }

    /// Per-pool hashrate-share entries from pre-loaded daily cumulative blocks
    /// plus the shared network series. Walks samples from `LOOKBACK_DAYS`
    /// onward in `sample_days` strides; for each sample emits one entry with
    ///   pool_blocks  = pool_cum[i] - pool_cum[i - LOOKBACK_DAYS]
    ///   total_blocks = first_heights[i] - first_heights[i - LOOKBACK_DAYS]
    ///   share        = pool_blocks / total_blocks
    ///   avg_hashrate = daily_hashrate[i] * share
    /// Skips samples where either cumulative value is `None`, where
    /// `pool_blocks == 0`, where `total_blocks == 0`, or where the network
    /// hashrate for that day is unavailable. The iteration is bounded by
    /// the shortest of `pool_cum`, `shared.first_heights`, and
    /// `shared.daily_hashrate` so per-vec stamp-lag truncation from
    /// `collect_range_at` degrades the chart's tail rather than panicking
    /// on out-of-bounds indexing. `LOOKBACK_DAYS` (rolling window) and
    /// `sample_days` (point spacing) are independent.
    fn compute_hashrate_entries(
        shared: &HashrateSharedData,
        pool_cum: &[Option<StoredU64>],
        pool_name: &'static str,
        sample_days: usize,
    ) -> Vec<PoolHashrateEntry> {
        let total = pool_cum
            .len()
            .min(shared.first_heights.len())
            .min(shared.daily_hashrate.len());
        if total <= LOOKBACK_DAYS {
            return vec![];
        }

        let mut entries = Vec::new();
        let mut i = LOOKBACK_DAYS;
        while i < total {
            if let (Some(cum_now), Some(cum_prev)) = (pool_cum[i], pool_cum[i - LOOKBACK_DAYS]) {
                let pool_blocks = (*cum_now).saturating_sub(*cum_prev);
                if pool_blocks > 0 {
                    let h_now = shared.first_heights[i].to_usize();
                    let h_prev = shared.first_heights[i - LOOKBACK_DAYS].to_usize();
                    let total_blocks = h_now.saturating_sub(h_prev);

                    if total_blocks > 0
                        && let Some(hr) = shared.daily_hashrate[i].as_ref()
                    {
                        let network_hr = **hr;
                        let share = pool_blocks as f64 / total_blocks as f64;
                        let day = Day1::from(shared.start_day + i);
                        entries.push(PoolHashrateEntry {
                            timestamp: day.to_timestamp(),
                            avg_hashrate: (network_hr * share) as u128,
                            share,
                            pool_name: Cow::Borrowed(pool_name),
                        });
                    }
                }
            }
            i += sample_days;
        }

        entries
    }
}
