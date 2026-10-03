use brk_error::{Error, Result};
use brk_types::{Height, TimePeriod, Timestamp};
use rustc_hash::FxHashMap;
use vecdb::{ReadableVec, VecIndex, VecValue};

use super::{block_bucket::BlockBucket, period_start::start_height_at};
use crate::Query;

/// Time-bucket divisor in seconds: blocks are grouped by `timestamp / div`.
/// `div = 1` puts each block in its own bucket; coarser values down-sample
/// long windows so the response stays bounded.
fn time_div(period: TimePeriod) -> u32 {
    match period {
        TimePeriod::Day | TimePeriod::ThreeDays => 1,
        TimePeriod::Week => 300,
        TimePeriod::Month => 1800,
        TimePeriod::ThreeMonths => 7200,
        TimePeriod::SixMonths => 10800,
        TimePeriod::Year | TimePeriod::TwoYears => 28800,
        TimePeriod::ThreeYears => 43200,
        TimePeriod::All => 86400,
    }
}

fn bucket_offsets(timestamps: &[Timestamp], div: u32) -> Vec<(u32, Vec<usize>)> {
    let mut groups: FxHashMap<u32, Vec<usize>> = FxHashMap::default();
    for (i, timestamp) in timestamps.iter().enumerate() {
        groups.entry(**timestamp / div).or_default().push(i);
    }
    let mut groups: Vec<_> = groups.into_iter().collect();
    groups.sort_unstable_by_key(|(key, _)| *key);
    groups
}

/// Round-half-up integer division, matching MySQL's `CAST(AVG(...) AS INT)`.
pub const fn round_half_up(sum: u128, n: u128) -> u64 {
    ((sum + n / 2) / n) as u64
}

/// Mempool-compatible time-bucketed block window. Groups blocks by
/// `block.timestamp / div` and exposes arithmetic means per bucket.
pub struct BlockWindow {
    start: Height,
    end: Height,
    pub buckets: Vec<BlockBucket>,
}

impl BlockWindow {
    /// Build a time-bucketed window over `[start_height(period), tip + 1)`.
    /// Prefetches `blocks.timestamp` once, groups block indices by
    /// `ts / div(period)` (chronological), and stores per-bucket offsets
    /// into the prefetched slice. Downstream metric reads (`BlockWindow::read`)
    /// reuse the same `[start, end)` so each bucket's offsets index directly
    /// into the value vec without a second walk.
    pub fn new(query: &Query, period: TimePeriod) -> Result<Self> {
        Self::new_at(query, period, query.height())
    }

    pub fn new_at(query: &Query, period: TimePeriod, tip: Height) -> Result<Self> {
        let start = start_height_at(query, period, tip)?;
        let end = tip + 1usize;
        let timestamps: Vec<Timestamp> = query
            .indexer()
            .vecs()
            .blocks
            .timestamp
            .collect_range(start, end);
        Self::from_timestamps(start, end, period, &timestamps)
    }

    fn from_timestamps(
        start: Height,
        end: Height,
        period: TimePeriod,
        timestamps: &[Timestamp],
    ) -> Result<Self> {
        let div = time_div(period);
        let len = end
            .to_usize()
            .checked_sub(start.to_usize())
            .ok_or(Error::Internal("Reversed mining block window"))?;
        if timestamps.len() != len {
            return Err(Error::Internal("Incomplete mining timestamp window"));
        }

        let buckets = bucket_offsets(timestamps, div)
            .into_iter()
            .map(|(_, offsets)| {
                let n = offsets.len() as u128;
                let sum_h: u128 = offsets
                    .iter()
                    .map(|&i| u128::from(u64::from(start + i)))
                    .sum();
                let sum_ts: u128 = offsets
                    .iter()
                    .map(|&i| u128::from(u64::from(timestamps[i])))
                    .sum();
                BlockBucket::new(
                    Height::from(round_half_up(sum_h, n)),
                    Timestamp::from(round_half_up(sum_ts, n) as u32),
                    offsets,
                )
            })
            .collect();

        Ok(Self {
            start,
            end,
            buckets,
        })
    }

    /// Read a height-keyed vec over this window's `[start, end)` range.
    /// Require the complete published range before indexing its bucket offsets.
    pub fn read<V, T>(&self, vec: &V) -> Result<Vec<T>>
    where
        V: ReadableVec<Height, T>,
        T: VecValue,
    {
        let values = vec.collect_range(self.start, self.end);
        if values.len() != self.end.to_usize() - self.start.to_usize() {
            return Err(Error::Internal("Incomplete mining value window"));
        }
        Ok(values)
    }
}
