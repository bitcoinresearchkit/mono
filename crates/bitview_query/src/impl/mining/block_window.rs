use std::ops::{Add, Div};

use bitview_types::TimePeriod;
use brk_types::{Cents, Dollars, Height, Timestamp};
use vecdb::{ReadableVec, VecIndex, VecValue};

use super::{block_bucket::BlockBucket, period_start::start_height};
use crate::{Error, Query, Result};

/// Time-bucket divisor in seconds: blocks are grouped by `timestamp / div`.
/// `div = 1` groups only equal timestamps; coarser values down-sample long
/// windows so the response stays bounded.
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

/// Round-half-up integer division, matching MySQL's `CAST(AVG(...) AS INT)`.
pub const fn round_half_up(sum: u128, n: u128) -> u64 {
    ((sum + n / 2) / n) as u64
}

/// Mempool-compatible time-bucketed block window. Groups blocks by
/// `block.timestamp / div` (chronological by bucket, whatever the block order) and folds
/// each column into per-bucket means while reading it.
pub struct BlockWindow {
    start: Height,
    end: Height,
    /// Each block's bucket, by offset from `start`.
    bucket_of: Vec<u32>,
    pub buckets: Vec<BlockBucket>,
}

impl BlockWindow {
    /// Build a time-bucketed window over `[start_height(period), tip + 1)`.
    pub fn new(query: &Query, period: TimePeriod, tip: Height) -> Result<Self> {
        let start = start_height(query, period, tip)?;
        let end = tip + 1usize;
        let div = time_div(period);
        let timestamps: Vec<Timestamp> = query
            .indexer()
            .vecs()
            .blocks
            .timestamp
            .collect_range(start, end);
        if timestamps.len() != end.to_usize() - start.to_usize() {
            return Err(Error::Internal("Incomplete mining timestamp window"));
        }

        let mut keys: Vec<u32> = timestamps
            .iter()
            .map(|timestamp| **timestamp / div)
            .collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        let mut sums = vec![(0u128, 0u128, 0u32); sorted.len()];
        for (offset, (key, timestamp)) in keys.iter_mut().zip(&timestamps).enumerate() {
            *key = sorted.binary_search(key).unwrap() as u32;
            let (height, time, count) = &mut sums[*key as usize];
            *height += (start.to_usize() + offset) as u128;
            *time += u128::from(**timestamp);
            *count += 1;
        }

        Ok(Self {
            start,
            end,
            bucket_of: keys,
            buckets: sums
                .into_iter()
                .map(|(height, time, count)| BlockBucket {
                    avg_height: Height::from(round_half_up(height, count.into())),
                    avg_timestamp: Timestamp::from(round_half_up(time, count.into()) as u32),
                    count,
                })
                .collect(),
        })
    }

    /// Folds a height-keyed column over the window into one accumulator per bucket. Requires
    /// the complete published range.
    fn fold<T: VecValue, A: Clone>(
        &self,
        vec: &impl ReadableVec<Height, T>,
        init: A,
        mut add: impl FnMut(&mut A, T),
    ) -> Result<Vec<A>> {
        let mut sums = vec![init; self.buckets.len()];
        let mut offset = 0;
        vec.for_each_range_dyn(self.start, self.end, &mut |value| {
            add(&mut sums[self.bucket_of[offset] as usize], value);
            offset += 1;
        });
        if offset != self.bucket_of.len() {
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
        let sums = self.fold(vec, U::default(), |sum, value| *sum = *sum + map(value))?;
        Ok(sums
            .into_iter()
            .zip(&self.buckets)
            .map(|(sum, bucket)| sum / bucket.count as usize)
            .collect())
    }

    /// Each bucket's round-half-up mean, for integer wrapper types convertible through `u64`.
    pub fn means_rounded<T: VecValue, U>(
        &self,
        vec: &impl ReadableVec<Height, T>,
        map: impl Fn(T) -> U,
    ) -> Result<Vec<U>>
    where
        U: From<u64>,
        u64: From<U>,
    {
        let sums = self.fold(vec, 0u128, |sum, value| {
            *sum += u128::from(u64::from(map(value)))
        })?;
        Ok(sums
            .into_iter()
            .zip(&self.buckets)
            .map(|(sum, bucket)| U::from(round_half_up(sum, bucket.count.into())))
            .collect())
    }

    /// Each bucket's mean price: the price source's unavailable sentinel stays unavailable
    /// instead of converting to an integer (which panics); finite means keep cent rounding.
    pub fn mean_prices(&self, prices: &impl ReadableVec<Height, Cents>) -> Result<Vec<Dollars>> {
        let sums = self.fold(prices, Some(0u128), |sum, price| {
            *sum = sum
                .zip(price.finite_inner())
                .map(|(sum, cents)| sum + u128::from(cents));
        })?;
        Ok(sums
            .into_iter()
            .zip(&self.buckets)
            .map(|(sum, bucket)| {
                sum.map_or(Dollars::NAN, |sum| {
                    Dollars::from(Cents::from(round_half_up(sum, bucket.count.into())))
                })
            })
            .collect())
    }
}
