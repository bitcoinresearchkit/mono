use std::{borrow::Cow, cmp::Reverse};

use bitview_primitives::{Count, Day1, Hashrate, Pool, PoolSlug, pools};
use bitview_types::{
    PoolBlockCounts, PoolBlockShares, PoolDetail, PoolDetailInfo, PoolHashrateEntry, PoolInfo,
    PoolStats, PoolsSummary, TimePeriod,
};
use brk_types::Height;
use vecdb::{ReadableVec, VecIndex};

use super::start_height;
use crate::{Error, OptionData, Query, ReadPin, Result};

/// Weekly samples, each sharing out the 7 days since the previous one.
const SAMPLE_DAYS: usize = 7;

/// The network side of weekly pool-hashrate samples, shared by every pool.
struct HashrateSamples {
    /// Weeks' last days (Sundays), oldest first; the first only opens the second's window.
    days: Vec<usize>,
    daily_hashrate: Vec<Option<Hashrate>>,
    /// Each sample day's closing boundary: the first height after it (the tip's successor today).
    day_ends: Vec<Height>,
}

impl Query {
    /// Pools ranked by blocks mined over `time_period`'s timestamp lookback from the tip (`All`:
    /// from genesis), with their shares and coinbase-only blocks, plus the current, 3-day and
    /// 7-day network hashrates.
    pub fn mining_pools(&self, time_period: TimePeriod) -> Result<PoolsSummary> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin)?;
        let start = start_height(self, time_period, tip)?.to_usize();
        let pool_data = self.ranked_pools(start, tip)?;

        let empty_blocks = self.empty_blocks_by_pool(start, &pin)?;
        let last_estimated_hashrate = self.hashrate_at(tip)?;
        let last_estimated_hashrate3d = self.trailing_hashrate(tip, 3)?;
        let last_estimated_hashrate1w = self.trailing_hashrate(tip, 7)?;
        drop(pin);

        let total_blocks: u64 = pool_data.iter().map(|(_, count)| count).sum();

        let pool_stats: Vec<PoolStats> = pool_data
            .into_iter()
            .enumerate()
            .map(|(idx, (pool, block_count))| {
                let share = block_count as f64 / total_blocks as f64;
                let empty_blocks = empty_blocks[pool.slug as usize];
                PoolStats::new(pool, block_count, empty_blocks, (idx + 1) as u32, share)
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

    /// The pools that mined blocks from `start` through `tip`, with their counts, most first.
    fn ranked_pools(&self, start: usize, tip: Height) -> Result<Vec<(&'static Pool, u64)>> {
        let pools = pools();
        let mut ranked = Vec::new();
        // Range count = cumulative(end) - cumulative(start - 1).
        for (pool_id, pool) in self.plugins().pools.by_slug.iter() {
            let cumulative = &pool.block_count.cumulative.height;
            let count_at_end: u64 = *cumulative.collect_one(tip).data()?;
            let count_at_start: u64 = if start == 0 {
                0
            } else {
                *cumulative.collect_one(Height::from(start - 1)).data()?
            };
            let block_count = count_at_end - count_at_start;
            if block_count > 0 {
                ranked.push((pools.get(*pool_id), block_count));
            }
        }
        ranked.sort_by_key(|p| Reverse(p.1));
        Ok(ranked)
    }

    /// Each pool's coinbase-only blocks from `start` through the pin's tip, by slug: one
    /// sequential pass over the window's first-transaction boundaries and pools.
    fn empty_blocks_by_pool(&self, start: usize, pin: &ReadPin) -> Result<[u64; 256]> {
        let lengths = pin.lengths();
        let end = lengths.height.to_usize();
        let first_tx_index = &self.indexer().vecs().transactions.first_tx_index;
        let mut slugs = self
            .plugins()
            .pools
            .block_pool
            .collect_range_at(start, end)
            .into_iter();
        let mut empty = [0u64; 256];
        let mut tally = |tx_count: usize| {
            let slug = slugs.next().expect("a pool per block");
            if tx_count == 1 {
                empty[slug as usize] += 1;
            }
        };
        let first = first_tx_index.collect_one_at(start).data()?;
        let last = first_tx_index.fold_range_at(start + 1, end, first, |previous, first| {
            tally(usize::from(first) - usize::from(previous));
            first
        });
        tally(usize::from(lengths.tx_index) - usize::from(last));
        Ok(empty)
    }

    /// The static list of supported pools.
    pub fn all_pools(&self) -> Vec<PoolInfo> {
        pools().iter().map(PoolInfo::from).collect()
    }

    /// Per-pool detail: lifetime block count plus 24h and 1w windowed counts,
    /// each as a share of network blocks in the same window. The 24h share is
    /// also used to weight the current 1-day network hashrate into a per-pool
    /// `estimated_hashrate`. `total_reward` is the coinbase value of every block
    /// the pool mined.
    pub fn pool_detail(&self, slug: PoolSlug) -> Result<PoolDetail> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin)?;
        let plugins = self.plugins();
        let end = tip.to_usize();

        let pools_list = pools();
        let pool = pools_list.get(slug);

        let vecs = plugins.pools.by_slug.get(&slug).ok_or(Error::Internal(
            "pool slug present in static list but missing from pools",
        ))?;
        let cumulative = &vecs.block_count.cumulative.height;

        let total_all: u64 = *cumulative.collect_one(tip).data()?;

        let window_stats = |start: Height| -> Result<(u64, f64)> {
            let start = start.to_usize();
            let count_before = if start == 0 {
                0
            } else {
                *cumulative.collect_one(Height::from(start - 1)).data()?
            };
            let count = total_all - count_before;
            let network_blocks = end
                .checked_sub(start)
                .ok_or(Error::Internal("Pool lookback exceeds published tip"))?
                + 1;
            Ok((count, count as f64 / network_blocks as f64))
        };
        let lookback = &plugins.mappings.lookback;
        let (total_24h, share_24h) = window_stats(lookback._24h.collect_one(tip).data()?)?;
        let (total_1w, share_1w) = window_stats(lookback._1w.collect_one(tip).data()?)?;
        let share_all = total_all as f64 / (end + 1) as f64;

        let network_hr = self.hashrate_at(tip)?;
        let estimated_hashrate = (share_24h * network_hr as f64) as u128;

        let total_reward = vecs
            .coinbase
            .cumulative
            .sats
            .height
            .collect_one(tip)
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
            avg_block_health: None,
            total_reward,
        })
    }

    /// Weekly-sampled hashrate series for a single pool over the full chain.
    /// Each point's hashrate is `network_hashrate(day) * pool_share_over_7d`,
    /// where the share is the pool's last-7-days block count divided by the
    /// network's last-7-days block count.
    pub fn pool_hashrate(&self, slug: PoolSlug) -> Result<Vec<PoolHashrateEntry>> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin)?;
        let pool_name = pools().get(slug).name;
        let samples = self.hashrate_samples(0, tip)?;
        let pool_cum = self.pool_sampled_cumulative(slug, &samples.days)?;
        drop(pin);
        Ok(Self::hashrate_entries(&samples, &pool_cum, pool_name).collect())
    }

    /// [`Self::pool_hashrate`]'s weekly samples over `time_period` (the full chain for `None` or
    /// `All`) for the pools that mined in the last week, by time and then their weekly rank, as
    /// mempool.space lists them.
    pub fn pools_hashrate(
        &self,
        time_period: Option<TimePeriod>,
    ) -> Result<Vec<PoolHashrateEntry>> {
        let pin = self.pin_safe_lengths()?;
        let tip = Query::tip_height(&pin)?;
        let start = match time_period {
            Some(tp) => start_height(self, tp, tip)?.to_usize(),
            None => 0,
        };

        let samples = self.hashrate_samples(start, tip)?;
        // The pools that mined in the last week, as mempool.space charts them.
        let week = start_height(self, TimePeriod::Week, tip)?.to_usize();
        let mut entries = Vec::new();
        for (pool, _) in self.ranked_pools(week, tip)? {
            let pool_cum = self.pool_sampled_cumulative(pool.slug, &samples.days)?;
            entries.extend(Self::hashrate_entries(&samples, &pool_cum, pool.name));
        }
        // By time, then the pools' weekly rank (the sort is stable).
        entries.sort_by_key(|entry| entry.timestamp);
        Ok(entries)
    }

    /// The network side of the weekly samples whose weeks end at or after `start_height`'s day,
    /// through the last completed one. Weeks end on Mondays 00:00 UTC, as mempool.space's: each
    /// sample day is a week's last day, a Sunday before the tip's (still open) day, and the first
    /// only opens the second's window. Reused across pools so the network series is read once per
    /// request.
    fn hashrate_samples(&self, start_height: usize, tip: Height) -> Result<HashrateSamples> {
        let plugins = self.plugins();
        let day_of = &plugins.mappings.height.day1;
        let start_day = day_of.collect_one_at(start_height).data()?.to_usize();
        let last_day = day_of.collect_one(tip).data()?.to_usize();
        let is_sunday = |day: usize| (*Day1::from(day).to_timestamp() / 86_400 + 4) % 7 == 0;
        let last_sunday = (0..last_day).rev().find(|&day| is_sunday(day));
        let days: Vec<usize> = match last_sunday {
            Some(last) => {
                let first = (start_day.max(SAMPLE_DAYS) - SAMPLE_DAYS..=last)
                    .find(|&day| is_sunday(day))
                    .unwrap_or(last);
                (first..=last).step_by(SAMPLE_DAYS).collect()
            }
            None => Vec::new(),
        };
        let daily_hashrate = plugins
            .mining
            .hash
            .hashrate
            .value
            .day1
            .read_sorted_at(&days);
        let first_height = &plugins.mappings.day1.first_height;
        let day_ends = days
            .iter()
            .map(|&day| first_height.collect_one_at(day + 1).data())
            .collect::<Result<Vec<_>>>()?;
        if daily_hashrate.len() != days.len() {
            return Err(Error::Internal("Incomplete pool hashrate window"));
        }

        Ok(HashrateSamples {
            days,
            daily_hashrate,
            day_ends,
        })
    }

    /// The pool's cumulative blocks mined through each sample day. Errors `Internal` if the
    /// slug has no series: both callers guarantee it is in the static pool list, so this only
    /// fires on a list/plugin mismatch, never on user input.
    fn pool_sampled_cumulative(
        &self,
        slug: PoolSlug,
        days: &[usize],
    ) -> Result<Vec<Option<Count>>> {
        let values = self
            .plugins()
            .pools
            .by_slug
            .get(&slug)
            .ok_or(Error::Internal(
                "pool slug present in static list but missing from pools",
            ))?
            .block_count
            .cumulative
            .day1
            .read_sorted_at(days);
        if values.len() != days.len() {
            return Err(Error::Internal("Incomplete pool cumulative window"));
        }
        Ok(values)
    }

    /// Per-pool hashrate-share entries, one per sample day `days[k]` after the first:
    ///   pool_blocks  = pool_cum[k] - pool_cum[k - 1]
    ///   total_blocks = day_ends[k] - day_ends[k - 1]
    ///   share        = pool_blocks / total_blocks
    ///   avg_hashrate = daily_hashrate[k] * share
    /// Both counts cover the same closing-day window. Skips samples where
    /// either cumulative value is `None`, where `pool_blocks == 0`, where
    /// `total_blocks == 0`, or where the network hashrate for that day is
    /// unavailable.
    fn hashrate_entries<'a>(
        samples: &'a HashrateSamples,
        pool_cum: &'a [Option<Count>],
        pool_name: &'static str,
    ) -> impl Iterator<Item = PoolHashrateEntry> + 'a {
        (1..samples.days.len()).filter_map(move |k| {
            let cum_now = pool_cum[k]?;
            let cum_prev = pool_cum[k - 1]?;
            let hr = samples.daily_hashrate[k]?;
            let pool_blocks = *cum_now - *cum_prev;
            let total_blocks = samples.day_ends[k].to_usize() - samples.day_ends[k - 1].to_usize();
            if pool_blocks == 0 || total_blocks == 0 {
                return None;
            }

            let share = pool_blocks as f64 / total_blocks as f64;
            Some(PoolHashrateEntry {
                // The week's end: the Monday after its last day.
                timestamp: Day1::from(samples.days[k] + 1).to_timestamp(),
                avg_hashrate: (f64::from(hr) * share) as u128,
                share,
                pool_name: Cow::Borrowed(pool_name),
            })
        })
    }
}
