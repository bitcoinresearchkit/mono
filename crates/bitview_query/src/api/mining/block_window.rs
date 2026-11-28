use std::ops::{Add, Div};

use bitview_types::TimePeriod;
use brk_types::{Cents, Dollars, Height, Timestamp};
use vecdb::{ReadableVec, VecIndex, VecValue};

use super::{block_bucket::BlockBucket, period_start::start_height};
use crate::{Error, Query, Result};

/// Time-bucket divisor in seconds: blocks are grouped by `timestamp / div`.
/// `div = 1` groups only equal timestamps; coarser values down-sample long
/// windows so the response stays bounded.
pub(crate) fn time_div(period: TimePeriod) -> u32 {
    match period {
        TimePeriod::Day | TimePeriod::ThreeDays => 1,
        TimePeriod::Week => 300,
        TimePeriod::Month => 1800,
        TimePeriod::ThreeMonths => 7200,
        TimePeriod::SixMonths => 10800,
        TimePeriod::Year | TimePeriod::TwoYears => 28800,
        TimePeriod::ThreeYears | TimePeriod::FourYears => 43200,
        TimePeriod::All => 86400,
    }
}

/// Round-half-up integer division, matching MySQL's `CAST(AVG(...) AS INT)`.
pub const fn round_half_up(sum: u128, n: u128) -> u64 {
    ((sum + n / 2) / n) as u64
}

/// Mempool-compatible time-bucketed block window. Groups blocks by
/// `block.timestamp / div` (chronological by bucket, whatever the block order) and reads each
/// column into per-bucket means. Consecutive blocks mostly share a bucket (a day is about 144
/// blocks), so the window is held and read as runs of them, never block by block.
pub struct BlockWindow {
    start: usize,
    end: usize,
    /// The window in height order, as runs of blocks sharing a bucket.
    runs: Vec<Run>,
    pub buckets: Vec<BlockBucket>,
}

/// Consecutive blocks of one bucket: from the run before's end up to `end`.
struct Run {
    bucket: usize,
    /// The height after the last block.
    end: usize,
}

impl BlockWindow {
    /// Build a time-bucketed window over `[start_height(period), tip + 1)`.
    pub fn new(query: &Query, period: TimePeriod, tip: Height) -> Result<Self> {
        let start = start_height(query, period, tip)?.to_usize();
        let end = tip.to_usize() + 1;
        let div = time_div(period);
        let timestamps: Vec<Timestamp> = query
            .indexer()
            .vecs()
            .blocks
            .timestamp
            .collect_range_at(start, end);
        if timestamps.len() != end - start {
            return Err(Error::Internal("Incomplete mining timestamp window"));
        }

        // Each run's key and the offset after its last block.
        let mut keyed = Vec::new();
        let mut offset = 0;
        while offset < timestamps.len() {
            let key = *timestamps[offset] / div;
            // The bucket's seconds: a range test per block instead of a division.
            let seconds = key * div..(key + 1) * div;
            offset += timestamps[offset..]
                .iter()
                .position(|timestamp| !seconds.contains(&**timestamp))
                .unwrap_or(timestamps.len() - offset);
            keyed.push((key, offset));
        }
        let mut keys: Vec<u32> = keyed.iter().map(|&(key, _)| key).collect();
        keys.sort_unstable();
        keys.dedup();

        let mut sums = vec![(0u64, 0u64, 0u32); keys.len()];
        let mut runs = Vec::with_capacity(keyed.len());
        let mut from = 0;
        for (key, to) in keyed {
            let bucket = keys.binary_search(&key).unwrap();
            let (height, time, count) = &mut sums[bucket];
            // The heights' sum: their mean (first plus last, halved) times how many they are.
            *height += ((2 * start + from + to - 1) * (to - from) / 2) as u64;
            *time += timestamps[from..to]
                .iter()
                .map(|timestamp| u64::from(**timestamp))
                .sum::<u64>();
            *count += (to - from) as u32;
            runs.push(Run {
                bucket,
                end: start + to,
            });
            from = to;
        }

        Ok(Self {
            start,
            end,
            runs,
            buckets: sums
                .into_iter()
                .map(|(height, time, count)| BlockBucket {
                    avg_height: Height::from(round_half_up(height.into(), count.into())),
                    avg_timestamp: Timestamp::from(
                        round_half_up(time.into(), count.into()) as u32
                    ),
                    count,
                })
                .collect(),
        })
    }

    /// Folds a height-keyed column over the window into one accumulator per bucket, in height
    /// order, a run's values (or a read chunk's share of them) at a time. Requires the
    /// complete published range.
    fn fold<T: VecValue, A: Clone>(
        &self,
        vec: &impl ReadableVec<Height, T>,
        init: A,
        mut add: impl FnMut(&mut A, &[T]),
    ) -> Result<Vec<A>> {
        let mut sums = vec![init; self.buckets.len()];
        let mut runs = self.runs.iter().peekable();
        let mut at = self.start;
        vec.for_each_chunk_at(self.start, self.end, &mut |_, mut values| {
            while !values.is_empty()
                && let Some(&&Run { bucket, end }) = runs.peek()
            {
                let (now, rest) = values.split_at(values.len().min(end - at));
                add(&mut sums[bucket], now);
                at += now.len();
                values = rest;
                if at == end {
                    runs.next();
                }
            }
        });
        if at != self.end {
            return Err(Error::Internal("Incomplete mining value window"));
        }
        Ok(sums)
    }

    /// Each bucket's arithmetic mean of `map(value)`.
    pub fn means<T: VecValue, U>(
        &self,
        vec: &impl ReadableVec<Height, T>,
        map: impl Fn(T) -> U,
    ) -> Result<Vec<U>>
    where
        U: Copy + Default + Add<Output = U> + Div<usize, Output = U>,
    {
        let sums = self.fold(vec, U::default(), |sum, values| {
            *sum = values
                .iter()
                .fold(*sum, |sum, value| sum + map(value.clone()));
        })?;
        Ok(sums
            .into_iter()
            .zip(&self.buckets)
            .map(|(sum, bucket)| sum / bucket.count as usize)
            .collect())
    }

    /// Each bucket's round-half-up mean of a per-block quantity, from its running total through
    /// each block: one read per run, however long.
    pub fn means_from_totals<T>(&self, totals: &impl ReadableVec<Height, T>) -> Result<Vec<T>>
    where
        T: VecValue + From<u64>,
        u64: From<T>,
    {
        let incomplete = || Error::Internal("Incomplete mining running totals");
        let mut previous = match self.start.checked_sub(1) {
            Some(before) => u64::from(totals.collect_one_at(before).ok_or_else(incomplete)?),
            None => 0,
        };
        let lasts: Vec<usize> = self.runs.iter().map(|run| run.end - 1).collect();
        let read = totals.read_sorted_at(&lasts);
        if read.len() != lasts.len() {
            return Err(incomplete());
        }
        let mut sums = vec![0u128; self.buckets.len()];
        for (run, total) in self.runs.iter().zip(read) {
            let total = u64::from(total);
            sums[run.bucket] += u128::from(total - previous);
            previous = total;
        }
        Ok(sums
            .into_iter()
            .zip(&self.buckets)
            .map(|(sum, bucket)| T::from(round_half_up(sum, bucket.count.into())))
            .collect())
    }

    /// Each bucket's mean price: the price source's unavailable sentinel stays unavailable
    /// instead of converting to an integer (which panics); finite means keep cent rounding.
    pub fn mean_prices(
        &self,
        prices: &impl ReadableVec<Height, Cents>,
    ) -> Result<Vec<Option<Dollars>>> {
        let sums = self.fold(prices, Some(0u128), |sum, prices| {
            *sum = prices.iter().fold(*sum, |sum, price| {
                sum.zip(price.finite_inner())
                    .map(|(sum, cents)| sum + u128::from(cents))
            });
        })?;
        Ok(sums
            .into_iter()
            .zip(&self.buckets)
            .map(|(sum, bucket)| {
                sum.map(|sum| Dollars::from(Cents::from(round_half_up(sum, bucket.count.into()))))
            })
            .collect())
    }
}
