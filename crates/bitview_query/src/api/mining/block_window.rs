use std::ops::{Add, Div, Range};

use bitview_types::{TimePeriod, TimestampRange};
use brk_types::{Cents, Dollars, Height, Timestamp};
use vecdb::{ReadableVec, VecValue};

use super::{block_bucket::BlockBucket, period_start::start_height};
use crate::{
    OptionData, Query, Result,
    api::indexed_bound::{IndexedBound, indexed_bound},
};

/// Time-bucket divisor in seconds for the existing named periods.
fn time_div(period: TimePeriod) -> u32 {
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

/// Upstream's strict `>` thresholds over the requested (not clamped) span.
fn span_div(span: u32) -> u32 {
    const DAY: u32 = 86_400;
    match span {
        s if s > 4 * 365 * DAY => 86_400,
        s if s > 2 * 365 * DAY => 43_200, // also >3y
        s if s > 6 * 30 * DAY => 28_800, // also >1y
        s if s > 3 * 30 * DAY => 10_800,
        s if s > 30 * DAY => 7_200,
        s if s > 7 * DAY => 1_800,
        s if s > 3 * DAY => 300,
        _ => 1, // also >24h
    }
}

/// Conservative height bounds; only the subsequent raw-timestamp filter is exact.
/// Prefix maximum excludes earlier heights. The first MTP >= `to` excludes its
/// descendants, whose raw times must exceed their predecessor's MTP. Keep the
/// crossing block itself, including when its raw time equals `to`.
fn timestamp_candidates(
    len: usize,
    range: TimestampRange,
    maximum: impl FnMut(usize) -> Result<Timestamp>,
    median_time: impl FnMut(usize) -> Result<Timestamp>,
) -> Result<Range<usize>> {
    let begin = indexed_bound(len, range.from(), IndexedBound::AtLeast, maximum)?;
    if begin == len {
        return Ok(len..len);
    }
    let crossing = indexed_bound(len, range.to(), IndexedBound::AtLeast, median_time)?;
    let end = if crossing == len { len } else { crossing + 1 };
    Ok(begin..end.max(begin))
}

/// Round-half-up integer division, matching the intended positive SQL integer mean.
pub const fn round_half_up(sum: u128, n: u128) -> u64 {
    ((sum + n / 2) / n) as u64
}

/// Selected heights and their time buckets from one pinned chain view. Named
/// periods select every height in their lookback; explicit ranges filter raw time.
/// Selection is request-local, not a retained timestamp or aggregate index.
pub struct BlockWindow {
    /// Height order, with compact bucket indices. Gaps are never folded as rows.
    selected: Vec<(Height, u32)>,
    pub buckets: Vec<BlockBucket>,
}

impl BlockWindow {
    pub fn new(query: &Query, period: TimePeriod, tip: Height) -> Result<Self> {
        let start = usize::from(start_height(query, period, tip)?);
        Self::read(query, start..usize::from(tip) + 1, time_div(period), |_| true)
    }

    /// Caller retains the same pin through selection and every column reduction.
    pub fn between(query: &Query, range: TimestampRange, tip: Height) -> Result<Self> {
        let blocks = &query.indexer().vecs().blocks;
        let maximum = &query.plugins().mappings.timestamp.monotonic;
        let candidates = timestamp_candidates(
            usize::from(tip) + 1,
            range,
            |height| maximum.collect_one_at(height).data(),
            |height| blocks.median_time.collect_one_at(height).data(),
        )?;
        Self::read(query, candidates, span_div(range.span()), |time| range.contains(time))
    }

    fn read(
        query: &Query,
        heights: Range<usize>,
        div: u32,
        include: impl Fn(Timestamp) -> bool,
    ) -> Result<Self> {
        let mut timestamps = query.indexer().vecs().blocks.timestamp
            .bounded_cursor_at(heights.start, heights.end);
        Self::select(heights, div, include, |height| timestamps.get(height).data())
    }

    /// Group selected timestamps using the existing sorted-key reducer. The selected
    /// vector first carries raw times, then compact bucket indices for column folds.
    fn select(
        heights: Range<usize>,
        div: u32,
        include: impl Fn(Timestamp) -> bool,
        mut read: impl FnMut(usize) -> Result<Timestamp>,
    ) -> Result<Self> {
        let mut selected = Vec::new();
        for height in heights {
            let timestamp = read(height)?;
            if include(timestamp) {
                selected.push((Height::from(height), *timestamp));
            }
        }
        let mut keys: Vec<_> = selected.iter().map(|(_, time)| time / div).collect();
        keys.sort_unstable();
        keys.dedup();
        let mut sums = vec![(0u128, 0u128, 0u32); keys.len()];
        for (height, time_or_bucket) in &mut selected {
            let time = *time_or_bucket;
            *time_or_bucket = keys.binary_search(&(time / div)).unwrap() as u32;
            let (height_sum, time_sum, count) = &mut sums[*time_or_bucket as usize];
            *height_sum += u128::from(u64::from(*height));
            *time_sum += u128::from(time);
            *count += 1;
        }
        let buckets = sums.into_iter()
            .map(|(height, time, count)| BlockBucket {
                avg_height: Height::from(round_half_up(height, count.into())),
                avg_timestamp: Timestamp::from(round_half_up(time, count.into()) as u32),
                count,
            })
            .collect();
        Ok(Self { selected, buckets })
    }

    /// Read only the selected heights, retaining decoded chunks across gaps.
    fn fold<T: VecValue, A: Clone>(
        &self,
        vec: &impl ReadableVec<Height, T>,
        init: A,
        add: impl FnMut(&mut A, T),
    ) -> Result<Vec<A>> {
        let start = self.selected.first().map_or(0, |(height, _)| usize::from(*height));
        let end = self.selected.last().map_or(start, |(height, _)| usize::from(*height) + 1);
        let mut cursor = vec.bounded_cursor_at(start, end);
        self.fold_with(init, add, |height| cursor.get(usize::from(height)).data())
    }

    fn fold_with<T, A: Clone>(
        &self,
        init: A,
        mut add: impl FnMut(&mut A, T),
        mut read: impl FnMut(Height) -> Result<T>,
    ) -> Result<Vec<A>> {
        let mut sums = vec![init; self.buckets.len()];
        for &(height, bucket) in &self.selected {
            add(&mut sums[bucket as usize], read(height)?);
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

// The bounded selection is checked against an independent full-column oracle.
#[cfg(test)]
#[path = "block_window_tests.rs"]
mod tests;
